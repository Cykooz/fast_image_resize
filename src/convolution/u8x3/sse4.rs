use std::arch::x86_64::*;

use crate::convolution::optimisations::Normalizer16;
use crate::pixels::U8x3;
use crate::{simd_utils, ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U8x3>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x3>,
    offset: u32,
    normalizer: &Normalizer16,
) {
    let precision = normalizer.precision();

    macro_rules! call {
        ($imm8:expr) => {{
            horiz_convolution_p::<$imm8>(src_view, dst_view, offset, normalizer);
        }};
    }
    constify_imm8!(precision, call);
}

fn horiz_convolution_p<const PRECISION: i32>(
    src_view: &impl ImageView<Pixel = U8x3>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x3>,
    offset: u32,
    normalizer: &Normalizer16,
) {
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows::<PRECISION>(src_rows, dst_rows, normalizer);
        }
    }

    let yy = dst_height - dst_height % 4;
    let src_rows = src_view.iter_rows(yy + offset);
    let dst_rows = dst_view.iter_rows_mut(yy);
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            horiz_convolution_one_row::<PRECISION>(src_row, dst_row, normalizer);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_four_rows<const PRECISION: i32>(
    src_rows: [&[U8x3]; 4],
    dst_rows: [&mut [U8x3]; 4],
    normalizer: &Normalizer16,
) {
    let zero = _mm_setzero_si128();
    let initial = _mm_set1_epi32(1 << (PRECISION - 1));
    let src_width = src_rows[0].len();

    /*
        |R  G  B | |R  G  B | |R  G  B | |R  G  B | |R  G  B | |R |
        |00 01 02| |03 04 05| |06 07 08| |09 10 11| |12 13 14| |15|

        Ignore 12-15 bytes in register and
        shuffle other components with converting from u8 into i16:

        x: |-1 -1| |-1 -1|
        B: |-1 05| |-1 02|
        G: |-1 04| |-1 01|
        R: |-1 03| |-1 00|
    */
    #[rustfmt::skip]
    let sh_lo = _mm_set_epi8(
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
    );
    /*
        x: |-1 -1| |-1 -1|
        B: |-1 11| |-1 08|
        G: |-1 10| |-1 07|
        R: |-1 09| |-1 06|
    */
    #[rustfmt::skip]
    let sh_hi = _mm_set_epi8(
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
    );

    for (dst_x, chunk) in normalizer.chunks().iter().enumerate() {
        let x_start = chunk.start as usize;
        let mut x = x_start;

        let mut sss_a = [initial; 4];
        let mut coeffs = chunk.values();

        // The next block of code will load source pixels by 16 bytes per time.
        // We must guarantee that this process won't go beyond
        // the one row of the image.
        // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 byte
        let max_x = src_width.saturating_sub(5);
        if x < max_x {
            let coeffs_by_4 = coeffs.chunks_exact(4);

            for k in coeffs_by_4 {
                let mmk0 = simd_utils::mm_load_and_clone_i16x2(k);
                let mmk1 = simd_utils::mm_load_and_clone_i16x2(&k[2..]);
                for i in 0..4 {
                    let source = simd_utils::loadu_si128(src_rows[i], x);
                    let pix = _mm_shuffle_epi8(source, sh_lo);
                    let mut sss = sss_a[i];
                    sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk0));
                    let pix = _mm_shuffle_epi8(source, sh_hi);
                    sss_a[i] = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk1));
                }

                x += 4;
                if x >= max_x {
                    break;
                }
            }
        }

        // The next block of code will load source pixels by 8 bytes per time.
        // We must guarantee that this process won't go beyond
        // the one row of the image.
        // (8 bytes) / (3 bytes per pixel) = 2 whole pixels + 2 bytes
        let max_x = src_width.saturating_sub(2);
        if x < max_x {
            let coeffs_by_2 = coeffs[x - x_start..].chunks_exact(2);

            for k in coeffs_by_2 {
                let mmk = simd_utils::mm_load_and_clone_i16x2(k);

                for i in 0..4 {
                    let source = simd_utils::loadl_epi64(src_rows[i], x);
                    let pix = _mm_shuffle_epi8(source, sh_lo);
                    sss_a[i] = _mm_add_epi32(sss_a[i], _mm_madd_epi16(pix, mmk));
                }

                x += 2;
                if x >= max_x {
                    break;
                }
            }
        }

        coeffs = coeffs.split_at(x - x_start).1;
        for &k in coeffs {
            let mmk = _mm_set1_epi32(k as i32);
            for i in 0..4 {
                let pix = simd_utils::mm_cvtepu8_epi32_u8x3(src_rows[i], x);
                sss_a[i] = _mm_add_epi32(sss_a[i], _mm_madd_epi16(pix, mmk));
            }

            x += 1;
        }

        sss_a[0] = _mm_srai_epi32::<PRECISION>(sss_a[0]);
        sss_a[1] = _mm_srai_epi32::<PRECISION>(sss_a[1]);
        sss_a[2] = _mm_srai_epi32::<PRECISION>(sss_a[2]);
        sss_a[3] = _mm_srai_epi32::<PRECISION>(sss_a[3]);

        for i in 0..4 {
            let sss = _mm_packs_epi32(sss_a[i], zero);
            let pixel: u32 = i32::cast_unsigned(_mm_cvtsi128_si32(_mm_packus_epi16(sss, zero)));
            let bytes = pixel.to_le_bytes();
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = [bytes[0], bytes[1], bytes[2]];
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_one_row<const PRECISION: i32>(
    src_row: &[U8x3],
    dst_row: &mut [U8x3],
    normalizer: &Normalizer16,
) {
    #[rustfmt::skip]
    let pix_sh1 = _mm_set_epi8(
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
    );
    #[rustfmt::skip]
    let coef_sh1 = _mm_set_epi8(
        3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0,
    );
    #[rustfmt::skip]
    let pix_sh2 = _mm_set_epi8(
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
    );
    #[rustfmt::skip]
    let coef_sh2 = _mm_set_epi8(
        7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4,
    );
    /*
        Load 8 bytes from memory into low half of 16-bytes register:
        |R  G  B | |R  G  B | |R  G |
        |00 01 02| |03 04 05| |06 07| 08 09 10 11 12 13 14 15

        Ignore 06-16 bytes in 16-bytes register and
        shuffle other components with converting from u8 into i16:

        x: |-1 -1| |-1 -1|
        B: |-1 05| |-1 02|
        G: |-1 04| |-1 01|
        R: |-1 03| |-1 00|
    */
    let src_width = src_row.len();

    for (dst_x, chunk) in normalizer.chunks().iter().enumerate() {
        let x_start = chunk.start as usize;
        let mut x = x_start;

        let mut coeffs = chunk.values();
        let mut sss = _mm_set1_epi32(1 << (PRECISION - 1));

        // The next block of code will load source pixels by 16 bytes per time.
        // We must guarantee that this process won't go beyond
        // the one row of the image.
        // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 byte
        let max_x = src_width.saturating_sub(5);
        if x < max_x {
            let coeffs_by_4 = coeffs.chunks_exact(4);
            for k in coeffs_by_4 {
                let ksource = simd_utils::loadl_epi64(k, 0);
                let source = simd_utils::loadu_si128(src_row, x);

                let pix = _mm_shuffle_epi8(source, pix_sh1);
                let mmk = _mm_shuffle_epi8(ksource, coef_sh1);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                let pix = _mm_shuffle_epi8(source, pix_sh2);
                let mmk = _mm_shuffle_epi8(ksource, coef_sh2);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                x += 4;
                if x >= max_x {
                    break;
                }
            }
        }

        // The next block of code will load source pixels by 8 bytes per time.
        // We must guarantee that this process won't go beyond
        // the one row of the image.
        // (8 bytes) / (3 bytes per pixel) = 2 whole pixels + 2 bytes
        let max_x = src_width.saturating_sub(2);
        if x < max_x {
            let coeffs_by_2 = coeffs[x - x_start..].chunks_exact(2);

            for k in coeffs_by_2 {
                let mmk = simd_utils::mm_load_and_clone_i16x2(k);
                let source = simd_utils::loadl_epi64(src_row, x);
                let pix = _mm_shuffle_epi8(source, pix_sh1);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                x += 2;
                if x >= max_x {
                    break;
                }
            }
        }

        coeffs = coeffs.split_at(x - x_start).1;
        for &k in coeffs {
            let pix = simd_utils::mm_cvtepu8_epi32_u8x3(src_row, x);
            let mmk = _mm_set1_epi32(k as i32);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));
            x += 1;
        }

        sss = _mm_srai_epi32::<PRECISION>(sss);
        sss = _mm_packs_epi32(sss, sss);
        let pixel: u32 = i32::cast_unsigned(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));
        let bytes = pixel.to_le_bytes();
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0 = [bytes[0], bytes[1], bytes[2]];
    }
}
