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
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U16x2]; 4],
    dst_rows: [&mut &mut [U16x2]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut ll_buf = [0i64; 4];

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

    #[rustfmt::skip]
    let p0_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0,
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0,
    );
    #[rustfmt::skip]
    let p1_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4,
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4,
    );
    #[rustfmt::skip]
    let p2_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8,
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8,
    );
    #[rustfmt::skip]
    let p3_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = [_mm256_set1_epi64x(half_error); 2];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        for k in coeffs_by_4 {
            let coeff0_i64x4 = _mm256_set1_epi64x(k[0] as i64);
            let coeff1_i64x4 = _mm256_set1_epi64x(k[1] as i64);
            let coeff2_i64x4 = _mm256_set1_epi64x(k[2] as i64);
            let coeff3_i64x4 = _mm256_set1_epi64x(k[3] as i64);

            for (i, sum) in ll_sum.iter_mut().enumerate() {
                let source = _mm256_set_m128i(
                    simd_utils::loadu_si128(src_rows[i * 2 + 1], x),
                    simd_utils::loadu_si128(src_rows[i * 2], x),
                );

                let pp_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff0_i64x4));

                let pp_i64x4 = _mm256_shuffle_epi8(source, p1_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff1_i64x4));

                let pp_i64x4 = _mm256_shuffle_epi8(source, p2_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff2_i64x4));

                let pp_i64x4 = _mm256_shuffle_epi8(source, p3_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff3_i64x4));
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let coeff0_i64x4 = _mm256_set1_epi64x(k[0] as i64);
            let coeff1_i64x4 = _mm256_set1_epi64x(k[1] as i64);

            for (i, sum) in ll_sum.iter_mut().enumerate() {
                let source = _mm256_set_m128i(
                    simd_utils::loadl_epi64(src_rows[i * 2 + 1], x),
                    simd_utils::loadl_epi64(src_rows[i * 2], x),
                );

                let pp_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff0_i64x4));

                let pp_i64x4 = _mm256_shuffle_epi8(source, p1_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff1_i64x4));
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_i64x4 = _mm256_set1_epi64x(k as i64);

            for (i, sum) in ll_sum.iter_mut().enumerate() {
                let source = _mm256_set_m128i(
                    simd_utils::loadl_epi32(src_rows[i * 2 + 1], x),
                    simd_utils::loadl_epi32(src_rows[i * 2], x),
                );

                let pp_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
                *sum = _mm256_add_epi64(*sum, _mm256_mul_epi32(pp_i64x4, coeff0_i64x4));
            }
        }

        // ll_sum.into_iter().enumerate() executes slowly than ll_sum.iter().enumerate()
        for (i, &ll) in ll_sum.iter().enumerate() {
            _mm256_storeu_si256((&mut ll_buf).as_mut_ptr() as *mut __m256i, ll);
            let dst_pixel = dst_rows[i * 2].get_unchecked_mut(dst_x);

            dst_pixel.0 = [normalizer.clip(ll_buf[0]), normalizer.clip(ll_buf[1])];

            let dst_pixel = dst_rows[i * 2 + 1].get_unchecked_mut(dst_x);
            dst_pixel.0 = [normalizer.clip(ll_buf[2]), normalizer.clip(ll_buf[3])];
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_one_row(
    src_row: &[U16x2],
    dst_row: &mut [U16x2],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut ll_buf = [0i64; 4];

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

    #[rustfmt::skip]
    let p0_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0,
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0,
    );
    #[rustfmt::skip]
    let p1_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4,
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4,
    );
    #[rustfmt::skip]
    let p2_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8,
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8,
    );
    #[rustfmt::skip]
    let p3_shuffle = _mm256_set_epi8(
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = _mm256_setzero_si256();
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeff04_i64x4 =
                _mm256_set_epi64x(k[4] as i64, k[4] as i64, k[0] as i64, k[0] as i64);
            let coeff15_i64x4 =
                _mm256_set_epi64x(k[5] as i64, k[5] as i64, k[1] as i64, k[1] as i64);
            let coeff26_i64x4 =
                _mm256_set_epi64x(k[6] as i64, k[6] as i64, k[2] as i64, k[2] as i64);
            let coeff37_i64x4 =
                _mm256_set_epi64x(k[7] as i64, k[7] as i64, k[3] as i64, k[3] as i64);

            let source = simd_utils::loadu_si256(src_row, x);

            let pp_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff04_i64x4));

            let pp_i64x4 = _mm256_shuffle_epi8(source, p1_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff15_i64x4));

            let pp_i64x4 = _mm256_shuffle_epi8(source, p2_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff26_i64x4));

            let pp_i64x4 = _mm256_shuffle_epi8(source, p3_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff37_i64x4));

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        for k in coeffs_by_4 {
            let coeff02_i64x4 =
                _mm256_set_epi64x(k[2] as i64, k[2] as i64, k[0] as i64, k[0] as i64);
            let coeff13_i64x4 =
                _mm256_set_epi64x(k[3] as i64, k[3] as i64, k[1] as i64, k[1] as i64);

            let source = _mm256_set_m128i(
                simd_utils::loadl_epi64(src_row, x + 2),
                simd_utils::loadl_epi64(src_row, x),
            );

            let pp_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff02_i64x4));

            let pp_i64x4 = _mm256_shuffle_epi8(source, p1_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff13_i64x4));

            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let coeff01_i64x4 =
                _mm256_set_epi64x(k[1] as i64, k[1] as i64, k[0] as i64, k[0] as i64);

            let source = _mm256_set_m128i(
                simd_utils::loadl_epi32(src_row, x + 1),
                simd_utils::loadl_epi32(src_row, x),
            );

            let pp_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(pp_i64x4, coeff01_i64x4));

            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_i64x4 = _mm256_set_epi64x(0, 0, k as i64, k as i64);
            let source = _mm256_set_m128i(_mm_setzero_si128(), simd_utils::loadl_epi32(src_row, x));
            let p_i64x4 = _mm256_shuffle_epi8(source, p0_shuffle);
            ll_sum = _mm256_add_epi64(ll_sum, _mm256_mul_epi32(p_i64x4, coeff0_i64x4));
        }

        _mm256_storeu_si256((&mut ll_buf).as_mut_ptr() as *mut __m256i, ll_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0 = [
            normalizer.clip(ll_buf[0] + ll_buf[2] + half_error),
            normalizer.clip(ll_buf[1] + ll_buf[3] + half_error),
        ];
    }
}
