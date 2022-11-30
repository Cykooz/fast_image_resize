use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U16;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U16>,
    dst_image: &mut ImageViewMut<U16>,
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
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U16]; 4],
    dst_rows: [&mut &mut [U16]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut ll_buf = [0i64; 2];

    /*
        |L0  | |L1  | |L2  | |L3  | |L4  | |L5  | |L6  | |L7  |
        |0001| |0203| |0405| |0607| |0809| |1011| |1213| |1415|

        Shuffle to extract L0 and L1 as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract L2 and L3 as i64:
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract L4 and L5 as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract L6 and L7 as i64:
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let l0l1_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0);
    let l2l3_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4);
    let l4l5_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8);
    let l6l7_shuffle = _mm_set_epi8(
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = [_mm_set1_epi64x(0); 4];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeff01_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);
            let coeff23_i64x2 = _mm_set_epi64x(k[3] as i64, k[2] as i64);
            let coeff45_i64x2 = _mm_set_epi64x(k[5] as i64, k[4] as i64);
            let coeff67_i64x2 = _mm_set_epi64x(k[7] as i64, k[6] as i64);

            for i in 0..4 {
                let mut sum = ll_sum[i];
                let source = simd_utils::loadu_si128(src_rows[i], x);

                let l0l1_i64x2 = _mm_shuffle_epi8(source, l0l1_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(l0l1_i64x2, coeff01_i64x2));

                let l2l3_i64x2 = _mm_shuffle_epi8(source, l2l3_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(l2l3_i64x2, coeff23_i64x2));

                let l4l5_i64x2 = _mm_shuffle_epi8(source, l4l5_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(l4l5_i64x2, coeff45_i64x2));

                let l6l7_i64x2 = _mm_shuffle_epi8(source, l6l7_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(l6l7_i64x2, coeff67_i64x2));

                ll_sum[i] = sum;
            }
            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff01_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);
            let coeff23_i64x2 = _mm_set_epi64x(k[3] as i64, k[2] as i64);

            for i in 0..4 {
                let mut sum = ll_sum[i];
                let source = simd_utils::loadl_epi64(src_rows[i], x);

                let l0l1_i64x2 = _mm_shuffle_epi8(source, l0l1_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(l0l1_i64x2, coeff01_i64x2));

                let l2l3_i64x2 = _mm_shuffle_epi8(source, l2l3_shuffle);
                sum = _mm_add_epi64(sum, _mm_mul_epi32(l2l3_i64x2, coeff23_i64x2));

                ll_sum[i] = sum;
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff01_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);
            for i in 0..4 {
                let source = simd_utils::loadl_epi32(src_rows[i], x);
                let l_i64x2 = _mm_shuffle_epi8(source, l0l1_shuffle);
                ll_sum[i] = _mm_add_epi64(ll_sum[i], _mm_mul_epi32(l_i64x2, coeff01_i64x2));
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff01_i64x2 = _mm_set_epi64x(0, k as i64);
            for i in 0..4 {
                let pixel = (*src_rows[i].get_unchecked(x)).0 as i64;
                let source = _mm_set_epi64x(0, pixel);
                ll_sum[i] = _mm_add_epi64(ll_sum[i], _mm_mul_epi32(source, coeff01_i64x2));
            }
        }

        for i in 0..4 {
            _mm_storeu_si128((&mut ll_buf).as_mut_ptr() as *mut __m128i, ll_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = normalizer.clip(ll_buf.iter().sum::<i64>() + half_error);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_one_row(
    src_row: &[U16],
    dst_row: &mut [U16],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut ll_buf = [0i64; 2];

    /*
        |L0  | |L1  | |L2  | |L3  | |L4  | |L5  | |L6  | |L7  |
        |0001| |0203| |0405| |0607| |0809| |1011| |1213| |1415|

        Shuffle to extract L0 and L1 as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract L2 and L3 as i64:
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract L4 and L5 as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract L6 and L7 as i64:
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let l01_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0);
    let l23_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4);
    let l45_shuffle = _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8);
    let l67_shuffle = _mm_set_epi8(
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = _mm_set1_epi64x(0);
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeff01_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);
            let coeff23_i64x2 = _mm_set_epi64x(k[3] as i64, k[2] as i64);
            let coeff45_i64x2 = _mm_set_epi64x(k[5] as i64, k[4] as i64);
            let coeff67_i64x2 = _mm_set_epi64x(k[7] as i64, k[6] as i64);

            let source = simd_utils::loadu_si128(src_row, x);

            let l_i64x2 = _mm_shuffle_epi8(source, l01_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff01_i64x2));

            let l_i64x2 = _mm_shuffle_epi8(source, l23_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff23_i64x2));

            let l_i64x2 = _mm_shuffle_epi8(source, l45_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff45_i64x2));

            let l_i64x2 = _mm_shuffle_epi8(source, l67_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff67_i64x2));

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff01_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);
            let coeff23_i64x2 = _mm_set_epi64x(k[3] as i64, k[2] as i64);

            let source = simd_utils::loadl_epi64(src_row, x);

            let l_i64x2 = _mm_shuffle_epi8(source, l01_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff01_i64x2));

            let l_i64x2 = _mm_shuffle_epi8(source, l23_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff23_i64x2));

            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff01_i64x2 = _mm_set_epi64x(k[1] as i64, k[0] as i64);
            let source = simd_utils::loadl_epi32(src_row, x);

            let l_i64x2 = _mm_shuffle_epi8(source, l01_shuffle);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(l_i64x2, coeff01_i64x2));

            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff01_i64x2 = _mm_set_epi64x(0, k as i64);
            let pixel = (*src_row.get_unchecked(x)).0 as i64;
            let source = _mm_set_epi64x(0, pixel);
            ll_sum = _mm_add_epi64(ll_sum, _mm_mul_epi32(source, coeff01_i64x2));
        }

        _mm_storeu_si128((&mut ll_buf).as_mut_ptr() as *mut __m128i, ll_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0 = normalizer.clip(ll_buf[0] + ll_buf[1] + half_error);
    }
}
