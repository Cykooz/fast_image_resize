use std::arch::x86_64::*;

use crate::convolution::{Coefficients, optimisations};
use crate::pixels::U8x2;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8x2>,
    dst_image: &mut ImageViewMut<U8x2>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows(
                src_rows,
                dst_rows,
                &coefficients_chunks,
                &normalizer,
            );
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_one_row(
                src_image.get_row(yy + offset).unwrap(),
                dst_image.get_row_mut(yy).unwrap(),
                &coefficients_chunks,
                &normalizer,
            );
        }
        yy += 1;
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
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U8x2]; 4],
    dst_rows: [&mut &mut [U8x2]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let precision = normalizer.precision();
    let initial = _mm256_set1_epi32(1 << (precision - 2));

    /*
        |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
        |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

        Shuffle components with converting from u8 into i16:

        A: |-1 07| |-1 05| |-1 03| |-1 01|
        L: |-1 06| |-1 04| |-1 02| |-1 00|
    */
    #[rustfmt::skip]
    let sh1 = _mm256_set_epi8(
        -1, 7, -1, 5, -1, 3, -1, 1, -1, 6, -1, 4, -1, 2, -1, 0,
        -1, 7, -1, 5, -1, 3, -1, 1, -1, 6, -1, 4, -1, 2, -1, 0,
    );
    /*
        A: |-1 15| |-1 13| |-1 11| |-1 09|
        L: |-1 14| |-1 12| |-1 10| |-1 08|
    */
    #[rustfmt::skip]
    let sh2 = _mm256_set_epi8(
        -1, 15, -1, 13, -1, 11, -1, 9, -1, 14, -1, 12, -1, 10, -1, 8,
        -1, 15, -1, 13, -1, 11, -1, 9, -1, 14, -1, 12, -1, 10, -1, 8,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x = coeffs_chunk.start as usize;

        let mut sss0 = initial;
        let mut sss1 = initial;
        let coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        let reminder = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let mmk0 = simd_utils::ptr_i16_to_256set1_epi64x(k, 0);
            let mmk1 = simd_utils::ptr_i16_to_256set1_epi64x(k, 4);

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadu_si128(src_rows[0], x)),
                simd_utils::loadu_si128(src_rows[1], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk0));
            let pix = _mm256_shuffle_epi8(source, sh2);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk1));

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadu_si128(src_rows[2], x)),
                simd_utils::loadu_si128(src_rows[3], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk0));
            let pix = _mm256_shuffle_epi8(source, sh2);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk1));

            x += 8;
        }

        let coeffs_by_4 = reminder.chunks_exact(4);
        let reminder = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let mmk = simd_utils::ptr_i16_to_256set1_epi64x(k, 0);

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi64(src_rows[0], x)),
                simd_utils::loadl_epi64(src_rows[1], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi64(src_rows[2], x)),
                simd_utils::loadl_epi64(src_rows[3], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            x += 4;
        }

        let coeffs_by_2 = reminder.chunks_exact(2);
        let reminder = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let mmk = simd_utils::ptr_i16_to_256set1_epi32(k, 0);

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi32(src_rows[0], x)),
                simd_utils::loadl_epi32(src_rows[1], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi32(src_rows[2], x)),
                simd_utils::loadl_epi32(src_rows[3], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            x += 2;
        }

        if let Some(&k) = reminder.first() {
            // [16] xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0
            let mmk = _mm256_set1_epi32(k as i32);

            // [16] xx a0 xx b0 xx g0 xx r0 xx a0 xx b0 xx g0 xx r0
            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi16(src_rows[0], x)),
                simd_utils::loadl_epi16(src_rows[1], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi16(src_rows[2], x)),
                simd_utils::loadl_epi16(src_rows[3], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));
        }

        let lo128 = _mm256_extracti128_si256::<0>(sss0);
        let hi128 = _mm256_extracti128_si256::<1>(sss0);
        set_dst_pixel(lo128, dst_rows[0], dst_x, normalizer);
        set_dst_pixel(hi128, dst_rows[1], dst_x, normalizer);

        let lo128 = _mm256_extracti128_si256::<0>(sss1);
        let hi128 = _mm256_extracti128_si256::<1>(sss1);
        set_dst_pixel(lo128, dst_rows[2], dst_x, normalizer);
        set_dst_pixel(hi128, dst_rows[3], dst_x, normalizer);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn set_dst_pixel(
    raw: __m128i,
    d_row: &mut &mut [U8x2],
    dst_x: usize,
    normalizer: &optimisations::Normalizer16,
) {
    let l32x2 = _mm_extract_epi64::<0>(raw);
    let a32x2 = _mm_extract_epi64::<1>(raw);
    let l32 = ((l32x2 >> 32) as i32).saturating_add((l32x2 & 0xffffffff) as i32);
    let a32 = ((a32x2 >> 32) as i32).saturating_add((a32x2 & 0xffffffff) as i32);
    let l8 = normalizer.clip(l32);
    let a8 = normalizer.clip(a32);
    d_row.get_unchecked_mut(dst_x).0 = u16::from_le_bytes([l8, a8]);
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_one_row(
    src_row: &[U8x2],
    dst_row: &mut [U8x2],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let precision = normalizer.precision();
    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

       Scale first four pixels into i16:

       A: |-1 07| |-1 05|
       L: |-1 06| |-1 04|
       A: |-1 03| |-1 01|
       L: |-1 02| |-1 00|
    */
    #[rustfmt::skip]
    let pix_sh1 = _mm256_set_epi8(
        -1, 7, -1, 5, -1, 6, -1, 4, -1, 3, -1, 1, -1, 2, -1, 0,
        -1, 7, -1, 5, -1, 6, -1, 4, -1, 3, -1, 1, -1, 2, -1, 0,
    );
    /*
       |C0   | |C1   | |C2   | |C3   | |C4   | |C5   | |C6   | |C7   |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

       Duplicate first four coefficients for A and L components of pixels:

       CA: |07 06| |05 04|
       CL: |07 06| |05 04|
       CA: |03 02| |01 00|
       CL: |03 02| |01 00|
    */
    #[rustfmt::skip]
    let coeff_sh1 = _mm256_set_epi8(
        7, 6, 5, 4, 7, 6, 5, 4, 3, 2, 1, 0, 3,2, 1, 0,
        7, 6, 5, 4, 7, 6, 5, 4, 3, 2, 1, 0, 3,2, 1, 0,
    );

    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

       Scale second four pixels into i16:

       A: |-1 15| |-1 13|
       L: |-1 14| |-1 12|
       A: |-1 11| |-1 09|
       L: |-1 10| |-1 08|
    */
    #[rustfmt::skip]
    let pix_sh2 = _mm256_set_epi8(
        -1, 15, -1, 13, -1, 14, -1, 12, -1, 11, -1, 9, -1, 10, -1, 8,
        -1, 15, -1, 13, -1, 14, -1, 12, -1, 11, -1, 9, -1, 10, -1, 8,    
    );
    /*
       |C0   | |C1   | |C2   | |C3   | |C4   | |C5   | |C6   | |C7   |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

       Duplicate second four coefficients for A and L components of pixels:

       CA: |15 14| |13 12|
       CL: |15 14| |13 12|
       CA: |11 10| |09 08|
       CL: |11 10| |09 08|
    */
    #[rustfmt::skip]
    let coeff_sh2 = _mm256_set_epi8(
        15, 14, 13, 12, 15, 14, 13, 12, 11, 10, 9, 8, 11, 10, 9, 8,
        15, 14, 13, 12, 15, 14, 13, 12, 11, 10, 9, 8, 11, 10, 9, 8,
    );

    /*
        Scale to i16 first four pixels in first half of register,
        and second four pixels in second half of register.
    */
    #[rustfmt::skip]
    let pix_sh3 = _mm256_set_epi8(
        -1, 15, -1, 13, -1, 14, -1, 12, -1, 11, -1, 9, -1, 10, -1, 8,
        -1, 7, -1, 5, -1, 6, -1, 4, -1, 3, -1, 1, -1, 2, -1, 0,
    );
    /*
        Duplicate first four coefficients in first half of register,
        and second four coefficients in second half of register.
    */
    #[rustfmt::skip]
    let coeff_sh3 = _mm256_set_epi8(
        15, 14, 13, 12, 15, 14, 13, 12, 11, 10, 9, 8, 11, 10, 9, 8,
        7, 6, 5, 4, 7, 6, 5, 4, 3, 2, 1, 0, 3,2, 1, 0,
    );

    /*
       |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

       Scale four pixels into i16:

       A: |-1 07| |-1 05|
       L: |-1 06| |-1 04|
       A: |-1 03| |-1 01|
       L: |-1 02| |-1 00|
    */
    let pix_sh4 = _mm_set_epi8(-1, 7, -1, 5, -1, 6, -1, 4, -1, 3, -1, 1, -1, 2, -1, 0);

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x = coeffs_chunk.start as usize;
        let mut coeffs = coeffs_chunk.values;

        let mut sss = if coeffs.len() < 16 {
            // Lower part will be added to higher, use only half of the error
            _mm_set1_epi32(1 << (precision - 2))
        } else {
            // Lower part will be added to higher twice, use only quarter of the error
            let mut sss256 = _mm256_set1_epi32(1 << (precision - 3));

            let coeffs_by_16 = coeffs.chunks_exact(16);
            let reminder = coeffs_by_16.remainder();

            for k in coeffs_by_16 {
                let ksource = simd_utils::loadu_si256(k, 0);
                let source = simd_utils::loadu_si256(src_row, x);

                let pix = _mm256_shuffle_epi8(source, pix_sh1);
                let mmk = _mm256_shuffle_epi8(ksource, coeff_sh1);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                let pix = _mm256_shuffle_epi8(source, pix_sh2);
                let mmk = _mm256_shuffle_epi8(ksource, coeff_sh2);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                x += 16;
            }

            let coeffs_by_8 = reminder.chunks_exact(8);
            coeffs = coeffs_by_8.remainder();

            for k in coeffs_by_8 {
                let tmp = simd_utils::loadu_si128(k, 0);
                let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                let tmp = simd_utils::loadu_si128(src_row, x);
                let source = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                let pix = _mm256_shuffle_epi8(source, pix_sh3);
                let mmk = _mm256_shuffle_epi8(ksource, coeff_sh3);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                x += 8;
            }

            _mm_add_epi32(
                _mm256_extracti128_si256::<0>(sss256),
                _mm256_extracti128_si256::<1>(sss256),
            )
        };

        let coeffs_by_4 = coeffs.chunks_exact(4);
        let reminder1 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let mmk = _mm_set_epi16(k[3], k[2], k[3], k[2], k[1], k[0], k[1], k[0]);
            let source = simd_utils::loadl_epi64(src_row, x);
            let pix = _mm_shuffle_epi8(source, pix_sh4);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            x += 4
        }

        if !reminder1.is_empty() {
            let mut pixels: [i16; 6] = [0; 6];
            let mut coeffs: [i16; 3] = [0; 3];
            for (i, &coeff) in reminder1.iter().enumerate() {
                coeffs[i] = coeff;
                let pixel: [u8; 2] = (*src_row.get_unchecked(x)).0.to_le_bytes();
                pixels[i * 2] = pixel[0] as i16;
                pixels[i * 2 + 1] = pixel[1] as i16;
                x += 1;
            }

            let pix = _mm_set_epi16(
                0, pixels[5], 0, pixels[4], pixels[3], pixels[1], pixels[2], pixels[0],
            );
            let mmk = _mm_set_epi16(
                0, coeffs[2], 0, coeffs[2], coeffs[1], coeffs[0], coeffs[1], coeffs[0],
            );

            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));
        }

        let lo = _mm_extract_epi64::<0>(sss);
        let hi = _mm_extract_epi64::<1>(sss);

        let a32 = ((lo >> 32) as i32).saturating_add((hi >> 32) as i32);
        let l32 = ((lo & 0xffffffff) as i32).saturating_add((hi & 0xffffffff) as i32);
        let a8 = normalizer.clip(a32);
        let l8 = normalizer.clip(l32);
        dst_row.get_unchecked_mut(dst_x).0 = u16::from_le_bytes([l8, a8]);
    }
}
