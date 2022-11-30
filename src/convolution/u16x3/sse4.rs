use std::arch::x86_64::*;

use crate::convolution::optimisations::CoefficientsI32Chunk;
use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U16x3;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U16x3>,
    dst_image: &mut ImageViewMut<U16x3>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_8u4x(src_rows, dst_rows, &coefficients_chunks, &normalizer);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_8u(
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
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_8u4x(
    src_rows: [&[U16x3]; 4],
    dst_rows: [&mut &mut [U16x3]; 4],
    coefficients_chunks: &[CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut rg_buf = [0i64; 2];
    let mut bb_buf = [0i64; 2];

    /*
        |R    G    B   | |R    G    B   | |R    G   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415|

        Shuffle to extract RG components of first pixel as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract RG components of second pixel as i64:
        -1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6

        Shuffle to extract B components of two pixels as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4

    */

    let rg0_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0);
    let rg1_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6);
    let bb_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4);

    let width = src_rows[0].len();

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sum = [_mm_set1_epi8(0); 4];
        let mut bb_sum = [_mm_set1_epi8(0); 4];

        let mut coeffs = coeffs_chunk.values;
        let end_x = x + coeffs.len();

        if width - end_x >= 1 {
            let coeffs_by_2 = coeffs.chunks_exact(2);
            coeffs = coeffs_by_2.remainder();

            for k in coeffs_by_2 {
                let coeff0_i64x2 = _mm_set1_epi64x(k[0] as i64);
                let coeff1_i64x2 = _mm_set1_epi64x(k[1] as i64);
                let coeff_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);

                for i in 0..4 {
                    let source = simd_utils::loadu_si128(src_rows[i], x);

                    let rg0_i64x2 = _mm_shuffle_epi8(source, rg0_shuffle);
                    rg_sum[i] = _mm_add_epi64(rg_sum[i], _mm_mul_epi32(rg0_i64x2, coeff0_i64x2));

                    let rg1_i64x2 = _mm_shuffle_epi8(source, rg1_shuffle);
                    rg_sum[i] = _mm_add_epi64(rg_sum[i], _mm_mul_epi32(rg1_i64x2, coeff1_i64x2));

                    let bb_i64x2 = _mm_shuffle_epi8(source, bb_shuffle);
                    bb_sum[i] = _mm_add_epi64(bb_sum[i], _mm_mul_epi32(bb_i64x2, coeff_i64x2));
                }
                x += 2;
            }
        }

        for &k in coeffs {
            let coeff_i64x2 = _mm_set1_epi64x(k as i64);

            for i in 0..4 {
                let &pixel = src_rows[i].get_unchecked(x);
                let rg_i64x2 = _mm_set_epi64x(pixel.0[1] as i64, pixel.0[0] as i64);
                rg_sum[i] = _mm_add_epi64(rg_sum[i], _mm_mul_epi32(rg_i64x2, coeff_i64x2));
                let bb_i64x2 = _mm_set_epi64x(0, pixel.0[2] as i64);
                bb_sum[i] = _mm_add_epi64(bb_sum[i], _mm_mul_epi32(bb_i64x2, coeff_i64x2));
            }
            x += 1;
        }

        for i in 0..4 {
            _mm_storeu_si128((&mut rg_buf).as_mut_ptr() as *mut __m128i, rg_sum[i]);
            _mm_storeu_si128((&mut bb_buf).as_mut_ptr() as *mut __m128i, bb_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0[0] = normalizer.clip(rg_buf[0] + half_error);
            dst_pixel.0[1] = normalizer.clip(rg_buf[1] + half_error);
            dst_pixel.0[2] = normalizer.clip(bb_buf[0] + bb_buf[1] + half_error);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_8u(
    src_row: &[U16x3],
    dst_row: &mut [U16x3],
    coefficients_chunks: &[CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let rg_initial = _mm_set1_epi64x(1 << (precision - 1));
    let bb_initial = _mm_set1_epi64x(1 << (precision - 2));

    /*
        |R    G    B   | |R    G    B   | |R    G   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415|

        Shuffle to extract RG components of first pixel as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract RG components of second pixel as i64:
        -1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6

        Shuffle to extract B components of two pixels as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4

    */

    let rg0_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0);
    let rg1_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6);
    let bb_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4);
    let mut rg_buf = [0i64; 2];
    let mut bb_buf = [0i64; 2];

    let width = src_row.len();

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;

        let mut rg_sum = rg_initial;
        let mut bb_sum = bb_initial;

        let mut coeffs = coeffs_chunk.values;
        let end_x = x + coeffs.len();

        if width - end_x >= 1 {
            let coeffs_by_2 = coeffs.chunks_exact(2);
            coeffs = coeffs_by_2.remainder();

            for k in coeffs_by_2 {
                let coeff0_i64x2 = _mm_set1_epi64x(k[0] as i64);
                let coeff1_i64x2 = _mm_set1_epi64x(k[1] as i64);
                let coeff_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);

                let source = simd_utils::loadu_si128(src_row, x);

                let rg0_i64x2 = _mm_shuffle_epi8(source, rg0_shuffle);
                rg_sum = _mm_add_epi64(rg_sum, _mm_mul_epi32(rg0_i64x2, coeff0_i64x2));

                let rg1_i64x2 = _mm_shuffle_epi8(source, rg1_shuffle);
                rg_sum = _mm_add_epi64(rg_sum, _mm_mul_epi32(rg1_i64x2, coeff1_i64x2));

                let bb_i64x2 = _mm_shuffle_epi8(source, bb_shuffle);
                bb_sum = _mm_add_epi64(bb_sum, _mm_mul_epi32(bb_i64x2, coeff_i64x2));
                x += 2;
            }
        }

        for &k in coeffs {
            let coeff_i64x2 = _mm_set1_epi64x(k as i64);

            let &pixel = src_row.get_unchecked(x);
            let rg_i64x2 = _mm_set_epi64x(pixel.0[1] as i64, pixel.0[0] as i64);
            rg_sum = _mm_add_epi64(rg_sum, _mm_mul_epi32(rg_i64x2, coeff_i64x2));
            let bb_i64x2 = _mm_set_epi64x(0, pixel.0[2] as i64);
            bb_sum = _mm_add_epi64(bb_sum, _mm_mul_epi32(bb_i64x2, coeff_i64x2));

            x += 1;
        }

        _mm_storeu_si128((&mut rg_buf).as_mut_ptr() as *mut __m128i, rg_sum);
        _mm_storeu_si128((&mut bb_buf).as_mut_ptr() as *mut __m128i, bb_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0[0] = normalizer.clip(rg_buf[0]);
        dst_pixel.0[1] = normalizer.clip(rg_buf[1]);
        dst_pixel.0[2] = normalizer.clip(bb_buf[0] + bb_buf[1]);
    }
}
