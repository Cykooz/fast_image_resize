use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U16x2;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U16x2>,
    dst_image: &mut ImageViewMut<U16x2>,
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
            horiz_convolution_four_rows(src_rows, dst_rows, &coefficients_chunks, &normalizer);
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
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U16x2]; 4],
    dst_rows: [&mut &mut [U16x2]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut ll_buf = [0i64; 2];

    /*
       |L0   A0  | |L1   A1  | |L2   A2  | |L3   A3  |
       |0001 0203| |0405 0607| |0809 1011| |1213 1415|

        Shuffle to extract L0 and A0 as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract L1 and A1 as i64:
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract L2 and A2 as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract L3 and A3 as i64:
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let p0_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0);
    let p1_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4);
    let p2_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8);
    let p3_shuffle = _mm_set_epi8(
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = [_mm_set1_epi64x(half_error); 4];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff0_i64x2 = _mm_set1_epi64x(k[0] as i64);
            let coeff1_i64x2 = _mm_set1_epi64x(k[1] as i64);
            let coeff2_i64x2 = _mm_set1_epi64x(k[2] as i64);
            let coeff3_i64x2 = _mm_set1_epi64x(k[3] as i64);

            for i in 0..4 {
                let mut sum = ll_sum[i];
                let source = simd_utils::loadu_si128(src_rows[i], x);

                let p_i64x2 = _mm_shuffle_epi8(source, p0_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(p_i64x2, coeff0_i64x2));

                let p_i64x2 = _mm_shuffle_epi8(source, p1_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(p_i64x2, coeff1_i64x2));

                let p_i64x2 = _mm_shuffle_epi8(source, p2_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(p_i64x2, coeff2_i64x2));

                let p_i64x2 = _mm_shuffle_epi8(source, p3_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(p_i64x2, coeff3_i64x2));

                ll_sum[i] = sum;
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff0_i64x2 = _mm_set1_epi64x(k[0] as i64);
            let coeff1_i64x2 = _mm_set1_epi64x(k[1] as i64);

            for i in 0..4 {
                let mut sum = ll_sum[i];
                let source = simd_utils::loadl_epi64(src_rows[i], x);

                let p_i64x2 = _mm_shuffle_epi8(source, p0_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(p_i64x2, coeff0_i64x2));

                let p_i64x2 = _mm_shuffle_epi8(source, p1_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(p_i64x2, coeff1_i64x2));

                ll_sum[i] = sum;
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_i64x2 = _mm_set1_epi64x(k as i64);
            for i in 0..4 {
                let source = simd_utils::loadl_epi32(src_rows[i], x);
                let p_i64x2 = _mm_shuffle_epi8(source, p0_shuffle);
                ll_sum[i] = _mm_add_epi64(ll_sum[i], _mm_mul_epi32(p_i64x2, coeff0_i64x2));
            }
        }

        for i in 0..4 {
            _mm_storeu_si128((&mut ll_buf).as_mut_ptr() as *mut __m128i, ll_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = [normalizer.clip(ll_buf[0]), normalizer.clip(ll_buf[1])];
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
unsafe fn horiz_convolution_one_row(
    src_row: &[U16x2],
    dst_row: &mut [U16x2],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut ll_buf = [0i64; 2];

    /*
       |L0   A0  | |L1   A1  | |L2   A2  | |L3   A3  |
       |0001 0203| |0405 0607| |0809 1011| |1213 1415|

        Shuffle to extract L0 and A0 as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract L1 and A1 as i64:
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract L2 and A2 as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract L3 and A3 as i64:
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let p0_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0);
    let p1_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4);
    let p2_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8);
    let p3_shuffle = _mm_set_epi8(
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = _mm_set1_epi64x(half_error);
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff0_i64x2 = _mm_set1_epi64x(k[0] as i64);
            let coeff1_i64x2 = _mm_set1_epi64x(k[1] as i64);
            let coeff2_i64x2 = _mm_set1_epi64x(k[2] as i64);
            let coeff3_i64x2 = _mm_set1_epi64x(k[3] as i64);

            let source = simd_utils::loadu_si128(src_row, x);

            let p_i64x2 = _mm_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff0_i64x2));

            let p_i64x2 = _mm_shuffle_epi8(source, p1_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff1_i64x2));

            let p_i64x2 = _mm_shuffle_epi8(source, p2_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff2_i64x2));

            let p_i64x2 = _mm_shuffle_epi8(source, p3_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff3_i64x2));

            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff0_i64x2 = _mm_set1_epi64x(k[0] as i64);
            let coeff1_i64x2 = _mm_set1_epi64x(k[1] as i64);

            let source = simd_utils::loadl_epi64(src_row, x);

            let p_i64x2 = _mm_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff0_i64x2));

            let p_i64x2 = _mm_shuffle_epi8(source, p1_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff1_i64x2));

            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_i64x2 = _mm_set1_epi64x(k as i64);
            let source = simd_utils::loadl_epi32(src_row, x);

            let p_i64x2 = _mm_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(p_i64x2, coeff0_i64x2));
        }

        _mm_storeu_si128((&mut ll_buf).as_mut_ptr() as *mut __m128i, ll_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0 = [normalizer.clip(ll_buf[0]), normalizer.clip(ll_buf[1])];
    }
}
