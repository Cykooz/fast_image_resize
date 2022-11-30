use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{ImageView, ImageViewMut};
use crate::pixels::U16x3;
use crate::simd_utils;

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
    src_rows: [&[U16x3]; 4],
    dst_rows: [&mut &mut [U16x3]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut rg_buf = [0i64; 4];
    let mut rg_bb_buf = [0i64; 4];
    let mut bbb_buf = [0i64; 4];

    /*
        |R    G    B   | |R    G    B   | |R    G   | - |B   | |R    G    B   | |R    G    B   | |R   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415| - |0001| |0203 0405 0607| |0809 1011 1213| |1415|

        Shuffle to extract RG components of pixels 0 and 3 as i64:
        lo: -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0
        hi: -1, -1, -1, -1, -1, -1, 5, 4, -1, -1, -1, -1, -1, -1, 3, 2

        Shuffle to extract RG components of pixels 1 and 4 as i64:
        lo: -1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6
        hi: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract RG components of pixel 2 and BB of pixels 2-3 as i64:
        lo: -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
        hi: -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract BB components of pixels 0, 1 and 4 as i64:
        lo: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4
        hi: -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let rg03_shuffle = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 5, 4, -1, -1, -1, -1, -1, -1, 3, 2),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
    );
    let rg14_shuffle = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6),
    );
    let rg3_b3b4_shuffle = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 1, 0),
        _mm_set_epi8(
            -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
        ),
    );
    let b1b2_b5_shuffle = _mm256_set_m128i(
        _mm_set_epi8(
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 13, 12,
        ),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4),
    );

    let width = src_rows[0].len();

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sum = [_mm256_set1_epi8(0); 4];
        let mut rg_bb_sum = [_mm256_set1_epi8(0); 4];
        let mut bbb_sum = [_mm256_set1_epi8(0); 4];

        let mut coeffs = coeffs_chunk.values;
        let end_x = x + coeffs.len();

        if width - end_x >= 1 {
            let coeffs_by_5 = coeffs.chunks_exact(5);
            coeffs = coeffs_by_5.remainder();

            for k in coeffs_by_5 {
                let coeff0033_i64x4 =
                    _mm256_set_epi64x(k[3] as i64, k[3] as i64, k[0] as i64, k[0] as i64);
                let coeff1144_i64x2 =
                    _mm256_set_epi64x(k[4] as i64, k[4] as i64, k[1] as i64, k[1] as i64);
                let coeff2223_i64x2 =
                    _mm256_set_epi64x(k[3] as i64, k[2] as i64, k[2] as i64, k[2] as i64);
                let coeff014_i64x2 = _mm256_set_epi64x(0, k[4] as i64, k[1] as i64, k[0] as i64);

                for i in 0..4 {
                    let source = simd_utils::loadu_si256(src_rows[i], x);

                    let rg03_i64x4 = _mm256_shuffle_epi8(source, rg03_shuffle);
                    rg_sum[i] =
                        _mm256_add_epi64(rg_sum[i], _mm256_mul_epi32(rg03_i64x4, coeff0033_i64x4));

                    let rg14_i64x4 = _mm256_shuffle_epi8(source, rg14_shuffle);
                    rg_sum[i] =
                        _mm256_add_epi64(rg_sum[i], _mm256_mul_epi32(rg14_i64x4, coeff1144_i64x2));

                    let rg_bb_i64x4 = _mm256_shuffle_epi8(source, rg3_b3b4_shuffle);
                    rg_bb_sum[i] = _mm256_add_epi64(
                        rg_bb_sum[i],
                        _mm256_mul_epi32(rg_bb_i64x4, coeff2223_i64x2),
                    );

                    let bbb_i64x4 = _mm256_shuffle_epi8(source, b1b2_b5_shuffle);
                    bbb_sum[i] =
                        _mm256_add_epi64(bbb_sum[i], _mm256_mul_epi32(bbb_i64x4, coeff014_i64x2));
                }
                x += 5;
            }
        }

        for &k in coeffs {
            let coeff_i64x4 = _mm256_set1_epi64x(k as i64);

            for i in 0..4 {
                let &pixel = src_rows[i].get_unchecked(x);
                let rgb_i64x4 =
                    _mm256_set_epi64x(0, pixel.0[2] as i64, pixel.0[1] as i64, pixel.0[0] as i64);
                rg_bb_sum[i] =
                    _mm256_add_epi64(rg_bb_sum[i], _mm256_mul_epi32(rgb_i64x4, coeff_i64x4));
            }
            x += 1;
        }

        for i in 0..4 {
            _mm256_storeu_si256((&mut rg_buf).as_mut_ptr() as *mut __m256i, rg_sum[i]);
            _mm256_storeu_si256((&mut rg_bb_buf).as_mut_ptr() as *mut __m256i, rg_bb_sum[i]);
            _mm256_storeu_si256((&mut bbb_buf).as_mut_ptr() as *mut __m256i, bbb_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0[0] = normalizer.clip(rg_buf[0] + rg_buf[2] + rg_bb_buf[0] + half_error);
            dst_pixel.0[1] = normalizer.clip(rg_buf[1] + rg_buf[3] + rg_bb_buf[1] + half_error);
            dst_pixel.0[2] = normalizer.clip(
                rg_bb_buf[2] + rg_bb_buf[3] + bbb_buf[0] + bbb_buf[1] + bbb_buf[2] + half_error,
            );
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_one_row(
    src_row: &[U16x3],
    dst_row: &mut [U16x3],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut rg_buf = [0i64; 4];
    let mut rg_bb_buf = [0i64; 4];
    let mut bbb_buf = [0i64; 4];

    /*
        |R    G    B   | |R    G    B   | |R    G   | - |B   | |R    G    B   | |R    G    B   | |R   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415| - |0001| |0203 0405 0607| |0809 1011 1213| |1415|

        Shuffle to extract RG components of pixels 0 and 3 as i64:
        lo: -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0
        hi: -1, -1, -1, -1, -1, -1, 5, 4, -1, -1, -1, -1, -1, -1, 3, 2

        Shuffle to extract RG components of pixels 1 and 4 as i64:
        lo: -1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6
        hi: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract RG components of pixel 2 and BB of pixels 2-3 as i64:
        lo: -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
        hi: -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract BB components of pixels 0, 1 and 4 as i64:
        lo: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4
        hi: -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let rg03_shuffle = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 5, 4, -1, -1, -1, -1, -1, -1, 3, 2),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
    );
    let rg14_shuffle = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 9, 8, -1, -1, -1, -1, -1, -1, 7, 6),
    );
    let rg3_b3b4_shuffle = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 1, 0),
        _mm_set_epi8(
            -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
        ),
    );
    let b1b2_b5_shuffle = _mm256_set_m128i(
        _mm_set_epi8(
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 13, 12,
        ),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 5, 4),
    );

    let zero_i64x4 = _mm256_set1_epi8(0);

    let width = src_row.len();

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sum = zero_i64x4;
        let mut rg_bb_sum = zero_i64x4;
        let mut bbb_sum = zero_i64x4;

        let mut coeffs = coeffs_chunk.values;
        let end_x = x + coeffs.len();

        if width - end_x >= 1 {
            let coeffs_by_5 = coeffs.chunks_exact(5);
            coeffs = coeffs_by_5.remainder();

            for k in coeffs_by_5 {
                let coeff0033_i64x4 =
                    _mm256_set_epi64x(k[3] as i64, k[3] as i64, k[0] as i64, k[0] as i64);
                let coeff1144_i64x2 =
                    _mm256_set_epi64x(k[4] as i64, k[4] as i64, k[1] as i64, k[1] as i64);
                let coeff2223_i64x2 =
                    _mm256_set_epi64x(k[3] as i64, k[2] as i64, k[2] as i64, k[2] as i64);
                let coeff014_i64x2 = _mm256_set_epi64x(0, k[4] as i64, k[1] as i64, k[0] as i64);

                let source = simd_utils::loadu_si256(src_row, x);

                let rg03_i64x4 = _mm256_shuffle_epi8(source, rg03_shuffle);
                rg_sum = _mm256_add_epi64(rg_sum, _mm256_mul_epi32(rg03_i64x4, coeff0033_i64x4));

                let rg14_i64x4 = _mm256_shuffle_epi8(source, rg14_shuffle);
                rg_sum = _mm256_add_epi64(rg_sum, _mm256_mul_epi32(rg14_i64x4, coeff1144_i64x2));

                let rg_bb_i64x4 = _mm256_shuffle_epi8(source, rg3_b3b4_shuffle);
                rg_bb_sum =
                    _mm256_add_epi64(rg_bb_sum, _mm256_mul_epi32(rg_bb_i64x4, coeff2223_i64x2));

                let bbb_i64x4 = _mm256_shuffle_epi8(source, b1b2_b5_shuffle);
                bbb_sum = _mm256_add_epi64(bbb_sum, _mm256_mul_epi32(bbb_i64x4, coeff014_i64x2));

                x += 5;
            }
        }

        for &k in coeffs {
            let coeff_i64x4 = _mm256_set1_epi64x(k as i64);
            let &pixel = src_row.get_unchecked(x);
            let rgb_i64x4 =
                _mm256_set_epi64x(0, pixel.0[2] as i64, pixel.0[1] as i64, pixel.0[0] as i64);
            rg_bb_sum = _mm256_add_epi64(rg_bb_sum, _mm256_mul_epi32(rgb_i64x4, coeff_i64x4));

            x += 1;
        }

        _mm256_storeu_si256((&mut rg_buf).as_mut_ptr() as *mut __m256i, rg_sum);
        _mm256_storeu_si256((&mut rg_bb_buf).as_mut_ptr() as *mut __m256i, rg_bb_sum);
        _mm256_storeu_si256((&mut bbb_buf).as_mut_ptr() as *mut __m256i, bbb_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0[0] = normalizer.clip(rg_buf[0] + rg_buf[2] + rg_bb_buf[0] + half_error);
        dst_pixel.0[1] = normalizer.clip(rg_buf[1] + rg_buf[3] + rg_bb_buf[1] + half_error);
        dst_pixel.0[2] = normalizer
            .clip(rg_bb_buf[2] + rg_bb_buf[3] + bbb_buf[0] + bbb_buf[1] + bbb_buf[2] + half_error);
    }
}
