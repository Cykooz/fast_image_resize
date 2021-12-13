use std::arch::x86_64::*;
use std::intrinsics::transmute;

use crate::convolution::optimisations::CoefficientsI16Chunk;
use crate::convolution::{optimisations, Bound, Coefficients};
use crate::image_view::{FourRows, FourRowsMut, TypedImageView, TypedImageViewMut};
use crate::pixels::{Pixel, U8x3};
use crate::simd_utils;

#[inline]
pub(crate) fn horiz_convolution(
    src_image: TypedImageView<U8x3>,
    mut dst_image: TypedImageViewMut<U8x3>,
    offset: u32,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds_per_pixel) =
        (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard::new(values);
    let precision = normalizer_guard.precision();
    let coefficients_chunks =
        normalizer_guard.normalized_i16_chunks(window_size, &bounds_per_pixel);
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_8u4x(src_rows, dst_rows, &coefficients_chunks, precision);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_8u(
                src_image.get_row(yy + offset).unwrap(),
                dst_image.get_row_mut(yy).unwrap(),
                &coefficients_chunks,
                precision,
            );
        }
        yy += 1;
    }
}

#[inline]
pub(crate) fn vert_convolution(
    src_image: TypedImageView<U8x3>,
    mut dst_image: TypedImageViewMut<U8x3>,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard::new(values);
    let precision = normalizer_guard.precision();
    let coeffs_i16 = normalizer_guard.normalized_i16();
    let coeffs_chunks = coeffs_i16.chunks(window_size);

    let dst_rows = dst_image.iter_rows_mut();
    for ((&bound, k), dst_row) in bounds.iter().zip(coeffs_chunks).zip(dst_rows) {
        unsafe {
            vert_convolution_8u(&src_image, dst_row, k, bound, precision);
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
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_8u4x(
    src_rows: FourRows<U8x3>,
    dst_rows: FourRowsMut<U8x3>,
    coefficients_chunks: &[CoefficientsI16Chunk],
    precision: u8,
) {
    let (s_row0, s_row1, s_row2, s_row3) = src_rows;
    let (d_row0, d_row1, d_row2, d_row3) = dst_rows;
    let zero = _mm256_setzero_si256();
    let initial = _mm256_set1_epi32(1 << (precision - 1));
    let src_width = s_row0.len();

    /*
        |R  G  B | |R  G  B | |R  G  B | |R  G  B | |R  G  B | |R |
        |00 01 02| |03 04 05| |06 07 08| |09 10 11| |12 13 14| |15|

        Ignore 12-15 bytes in each half of 32-bytes register and
        shuffle other components with converting from u8 into i16:

        x: |-1 -1| |-1 -1|
        B: |-1 05| |-1 02|
        G: |-1 04| |-1 01|
        R: |-1 03| |-1 00|
    */
    #[rustfmt::skip]
    let sh1 = _mm256_set_epi8(
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
    );
    /*
        x: |-1 -1| |-1 -1|
        B: |-1 11| |-1 08|
        G: |-1 10| |-1 07|
        R: |-1 09| |-1 06|
    */
    #[rustfmt::skip]
    let sh2 = _mm256_set_epi8(
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let x_start = coeffs_chunk.start as usize;
        let mut x = x_start;

        let mut sss0 = initial;
        let mut sss1 = initial;
        let mut coeffs = coeffs_chunk.values;

        // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 byte
        let max_x = src_width.saturating_sub(5);
        if x < max_x {
            let coeffs_by_4 = coeffs.chunks_exact(4);

            for k in coeffs_by_4 {
                let mmk0 = simd_utils::ptr_i16_to_256set1_epi32(k, 0);
                let mmk1 = simd_utils::ptr_i16_to_256set1_epi32(k, 2);

                let source = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadu_si128(s_row0, x)),
                    simd_utils::loadu_si128(s_row1, x),
                );
                let pix = _mm256_shuffle_epi8(source, sh1);
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk0));
                let pix = _mm256_shuffle_epi8(source, sh2);
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk1));

                let source = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadu_si128(s_row2, x)),
                    simd_utils::loadu_si128(s_row3, x),
                );
                let pix = _mm256_shuffle_epi8(source, sh1);
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk0));
                let pix = _mm256_shuffle_epi8(source, sh2);
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk1));

                x += 4;
                if x >= max_x {
                    break;
                }
            }
        }

        // (8 bytes) / (3 bytes per pixel) = 2 whole pixels + 2 bytes
        let max_x = src_width.saturating_sub(2);
        if x < max_x {
            let coeffs_by_2 = coeffs[x - x_start..].chunks_exact(2);

            for k in coeffs_by_2 {
                let mmk = simd_utils::ptr_i16_to_256set1_epi32(k, 0);

                let source = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadl_epi64(s_row0, x)),
                    simd_utils::loadl_epi64(s_row1, x),
                );
                let pix = _mm256_shuffle_epi8(source, sh1);
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

                let source = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadl_epi64(s_row2, x)),
                    simd_utils::loadl_epi64(s_row3, x),
                );
                let pix = _mm256_shuffle_epi8(source, sh1);
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

                x += 2;
                if x >= max_x {
                    break;
                }
            }
        }

        coeffs = coeffs.split_at(x - x_start).1;
        for &k in coeffs {
            // [16] xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0
            let mmk = _mm256_set1_epi32(k as i32);

            // [16] xx a0 xx b0 xx g0 xx r0 xx a0 xx b0 xx g0 xx r0
            let pix = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::mm_cvtepu8_epi32_u8x3(s_row0, x)),
                simd_utils::mm_cvtepu8_epi32_u8x3(s_row1, x),
            );
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

            let pix = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::mm_cvtepu8_epi32_u8x3(s_row2, x)),
                simd_utils::mm_cvtepu8_epi32_u8x3(s_row3, x),
            );
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            x += 1;
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = _mm256_srai_epi32::<$imm8>(sss0);
                sss1 = _mm256_srai_epi32::<$imm8>(sss1);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = _mm256_packs_epi32(sss0, zero);
        sss1 = _mm256_packs_epi32(sss1, zero);
        sss0 = _mm256_packus_epi16(sss0, zero);
        sss1 = _mm256_packus_epi16(sss1, zero);

        let pixel: u32 = transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<0>(sss0)));
        let bytes = pixel.to_le_bytes();
        d_row0.get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];

        let pixel: u32 = transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<1>(sss0)));
        let bytes = pixel.to_le_bytes();
        d_row1.get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];

        let pixel: u32 = transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<0>(sss1)));
        let bytes = pixel.to_le_bytes();
        d_row2.get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];

        let pixel: u32 = transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<1>(sss1)));
        let bytes = pixel.to_le_bytes();
        d_row3.get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_8u(
    src_row: &[U8x3],
    dst_row: &mut [U8x3],
    coefficients_chunks: &[CoefficientsI16Chunk],
    precision: u8,
) {
    #[rustfmt::skip]
    let sh1 = _mm256_set_epi8(
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
    );
    #[rustfmt::skip]
    let sh2 = _mm256_set_epi8(
        11, 10, 9, 8, 11, 10, 9, 8, 11, 10, 9, 8, 11, 10, 9, 8,
        3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0,
    );
    #[rustfmt::skip]
    let sh3 = _mm256_set_epi8(
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
    );
    #[rustfmt::skip]
    let sh4 = _mm256_set_epi8(
        15, 14, 13, 12, 15, 14, 13, 12, 15, 14, 13, 12, 15, 14, 13, 12,
        7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4,
    );
    #[rustfmt::skip]
    let sh5 = _mm256_set_epi8(
        -1, -1, -1, -1, -1, 11, -1, 8, -1, 10, -1, 7, -1, 9, -1, 6,
        -1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0,
    );
    #[rustfmt::skip]
    let sh6 = _mm256_set_epi8(
        7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4,
        3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0,
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
    let sh7 = _mm_set_epi8(-1, -1, -1, -1, -1, 5, -1, 2, -1, 4, -1, 1, -1, 3, -1, 0);
    let src_width = src_row.len();

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let x_start = coeffs_chunk.start as usize;
        let mut x = x_start;
        let mut coeffs = coeffs_chunk.values;

        // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 bytes
        // 4 + 5 = 9
        let max_x = src_width.saturating_sub(9);

        // (32 bytes) / (3 bytes per pixel) = 10 whole pixels + 2 bytes
        let mut sss = if coeffs.len() < 8 || x >= max_x {
            _mm_set1_epi32(1 << (precision - 1))
        } else {
            // Lower part will be added to higher, use only half of the error
            let mut sss256 = _mm256_set1_epi32(1 << (precision - 2));

            let coeffs_by_8 = coeffs.chunks_exact(8);
            for k in coeffs_by_8 {
                let tmp = simd_utils::loadu_si128(k, 0);
                let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                let s_upper = simd_utils::loadu_si128(src_row, x);
                let s_lower = simd_utils::loadu_si128(src_row, x + 4);
                let source = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(s_upper), s_lower);

                let pix = _mm256_shuffle_epi8(source, sh1);
                let mmk = _mm256_shuffle_epi8(ksource, sh2);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                let pix = _mm256_shuffle_epi8(source, sh3);
                let mmk = _mm256_shuffle_epi8(ksource, sh4);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                x += 8;
                if x >= max_x {
                    break;
                }
            }

            // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 bytes
            let max_x = src_width.saturating_sub(5);
            if x < max_x {
                let coeffs_by_4 = coeffs[x - x_start..].chunks_exact(4);

                for k in coeffs_by_4 {
                    let tmp = simd_utils::loadl_epi64(k, 0);
                    let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                    let tmp = simd_utils::loadu_si128(src_row, x);
                    let source = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                    let pix = _mm256_shuffle_epi8(source, sh5);
                    let mmk = _mm256_shuffle_epi8(ksource, sh6);
                    sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                    x += 4;
                    if x >= max_x {
                        break;
                    }
                }
            }

            _mm_add_epi32(
                _mm256_extracti128_si256::<0>(sss256),
                _mm256_extracti128_si256::<1>(sss256),
            )
        };

        // (8 bytes) / (3 bytes per pixel) = 2 whole pixels + 2 bytes
        let max_x = src_width.saturating_sub(2);
        if x < max_x {
            let coeffs_by_2 = coeffs[x - x_start..].chunks_exact(2);

            for k in coeffs_by_2 {
                let mmk = simd_utils::ptr_i16_to_set1_epi32(k, 0);
                let source = simd_utils::loadl_epi64(src_row, x);
                let pix = _mm_shuffle_epi8(source, sh7);
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

        macro_rules! call {
            ($imm8:expr) => {{
                sss = _mm_srai_epi32::<$imm8>(sss);
            }};
        }
        constify_imm8!(precision, call);

        sss = _mm_packs_epi32(sss, sss);
        let pixel: u32 = transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));
        let bytes = pixel.to_le_bytes();
        dst_row.get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn vert_convolution_8u(
    src_img: &TypedImageView<U8x3>,
    dst_row: &mut [U8x3],
    coeffs: &[i16],
    bound: Bound,
    precision: u8,
) {
    let src_width = src_img.width().get() as usize;
    let y_start = bound.start;
    let y_size = bound.size;

    let initial = _mm_set1_epi32(1 << (precision - 1));
    let initial_256 = _mm256_set1_epi32(1 << (precision - 1));

    let mut x_in_bytes: usize = 0;
    let width_in_bytes = src_width * U8x3::size();
    let dst_ptr_u8 = dst_row.as_mut_ptr() as *mut u8;

    while x_in_bytes < width_in_bytes.saturating_sub(31) {
        let mut sss0 = initial_256;
        let mut sss1 = initial_256;
        let mut sss2 = initial_256;
        let mut sss3 = initial_256;

        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_256set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadu_si256_raw(s_row1, x_in_bytes); // top line
            let source2 = simd_utils::loadu_si256_raw(s_row2, x_in_bytes); // bottom line

            let source = _mm256_unpacklo_epi8(source1, source2);
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_unpackhi_epi8(source1, source2);
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(pix, mmk));

            y += 2;
        }

        if let Some(s_row) = src_img.get_row(y_start + y) {
            let mmk = _mm256_set1_epi32(coeffs[y as usize] as i32);

            let source1 = simd_utils::loadu_si256_raw(s_row, x_in_bytes); // top line
            let source2 = _mm256_setzero_si256(); // bottom line is empty

            let source = _mm256_unpacklo_epi8(source1, source2);
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_unpackhi_epi8(source1, _mm256_setzero_si256());
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = _mm256_srai_epi32::<$imm8>(sss0);
                sss1 = _mm256_srai_epi32::<$imm8>(sss1);
                sss2 = _mm256_srai_epi32::<$imm8>(sss2);
                sss3 = _mm256_srai_epi32::<$imm8>(sss3);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = _mm256_packs_epi32(sss0, sss1);
        sss2 = _mm256_packs_epi32(sss2, sss3);
        sss0 = _mm256_packus_epi16(sss0, sss2);

        let dst_ptr = dst_ptr_u8.add(x_in_bytes) as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, sss0);

        x_in_bytes += 32;
    }

    while x_in_bytes < width_in_bytes.saturating_sub(7) {
        let mut sss0 = initial; // left row
        let mut sss1 = initial; // right row
        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadl_epi64_raw(s_row1, x_in_bytes); // top line
            let source2 = simd_utils::loadl_epi64_raw(s_row2, x_in_bytes); // bottom line

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

            y += 2;
        }

        if let Some(s_row) = src_img.get_row(y_start + y) {
            let mmk = _mm_set1_epi32(*coeffs.get_unchecked(y as usize) as i32);

            let source1 = simd_utils::loadl_epi64_raw(s_row, x_in_bytes); // top line
            let source2 = _mm_setzero_si128(); // bottom line is empty

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = _mm_srai_epi32::<$imm8>(sss0);
                sss1 = _mm_srai_epi32::<$imm8>(sss1);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = _mm_packs_epi32(sss0, sss1);
        sss0 = _mm_packus_epi16(sss0, sss0);

        let dst_ptr = dst_ptr_u8.add(x_in_bytes) as *mut __m128i;
        _mm_storel_epi64(dst_ptr, sss0);

        x_in_bytes += 8;
    }

    while x_in_bytes < width_in_bytes.saturating_sub(3) {
        let mut sss = initial;
        let mut y: u32 = 0;
        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
            // Load two coefficients at once
            let two_coeffs = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let row1 = simd_utils::mm_cvtsi32_si128_from_raw(s_row1, x_in_bytes); // top line
            let row2 = simd_utils::mm_cvtsi32_si128_from_raw(s_row2, x_in_bytes); // bottom line

            let pixels_u8 = _mm_unpacklo_epi8(row1, row2);
            let pixels_i16 = _mm_unpacklo_epi8(pixels_u8, _mm_setzero_si128());
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pixels_i16, two_coeffs));

            y += 2;
        }

        if let Some(s_row) = src_img.get_row(y_start + y) {
            let pix = simd_utils::mm_cvtepu8_epi32_from_raw(s_row, x_in_bytes);
            let mmk = _mm_set1_epi32(*coeffs.get_unchecked(y as usize) as i32);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss = _mm_srai_epi32::<$imm8>(sss);
            }};
        }
        constify_imm8!(precision, call);

        sss = _mm_packs_epi32(sss, sss);
        let dst_ptr_i32 = dst_ptr_u8.add(x_in_bytes) as *mut i32;
        *dst_ptr_i32 = _mm_cvtsi128_si32(_mm_packus_epi16(sss, sss));

        x_in_bytes += 4;
    }

    if x_in_bytes < width_in_bytes {
        let dst_u8 =
            std::slice::from_raw_parts_mut(dst_ptr_u8.add(x_in_bytes), width_in_bytes - x_in_bytes);

        for dst_pixel in dst_u8 {
            let mut ss0 = 1 << (precision - 1);
            for (dy, &k) in coeffs.iter().take(y_size as usize).enumerate() {
                if let Some(src_row) = src_img.get_row(y_start + dy as u32) {
                    let src_ptr = src_row.as_ptr() as *const u8;
                    let src_component = *src_ptr.add(x_in_bytes);
                    ss0 += src_component as i32 * (k as i32);
                }
            }
            *dst_pixel = optimisations::clip8(ss0, precision);
            x_in_bytes += 1;
        }
    }
}
