use std::arch::x86_64::*;

use crate::convolution::optimisations::CoefficientsI32Chunk;
use crate::convolution::vertical_u16::native::convolution_by_u16;
use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::Pixel;
use crate::simd_utils;

pub(crate) fn vert_convolution<T: Pixel<Component = u16>>(
    src_image: TypedImageView<T>,
    mut dst_image: TypedImageViewMut<T>,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u16(&src_image, dst_row, coeffs_chunk, &normalizer);
        }
    }
}

#[target_feature(enable = "sse4.1")]
unsafe fn vert_convolution_into_one_row_u16<T: Pixel<Component = u16>>(
    src_img: &TypedImageView<T>,
    dst_row: &mut [T],
    coeffs_chunk: CoefficientsI32Chunk,
    normalizer: &optimisations::Normalizer32,
) {
    let mut xx: usize = 0;
    let src_width = src_img.width().get() as usize * T::components_count();
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let max_y = y_start + coeffs.len() as u32;
    let dst_components = T::components_mut(dst_row);
    let mut dst_ptr_u16 = dst_components.as_mut_ptr() as *mut u16;

    /*
        |0    1    2    3    4    5    6    7   |
        |0001 0203 0405 0607 0809 1011 1213 1415|

        Shuffle to extract 0-1 components as i64:
        -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract 2-3 components as i64:
        -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract 4-5 components as i64:
        -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract 6-7 components as i64:
        -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12

    */

    let c_shuffles = [
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4),
        _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
        _mm_set_epi8(
            -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
        ),
    ];

    let precision = normalizer.precision();
    let initial = _mm_set1_epi64x(1 << (precision - 1));
    let mut c_buf = [0i64; 2];

    // 16 components - 1 = 15
    while xx < src_width.saturating_sub(15) {
        let mut sums = [[initial; 2], [initial; 2], [initial; 2], [initial; 2]];

        let mut y: u32 = 0;
        let coeffs_2 = coeffs.chunks_exact(2);
        let coeffs_reminder = coeffs_2.remainder();

        for ((s_row0, s_row1), two_coeffs) in src_img.iter_2_rows(y_start, max_y).zip(coeffs_2) {
            let s_rows = [T::components(s_row0), T::components(s_row1)];

            for r in 0..2 {
                let coeff_i64x2 = _mm_set1_epi64x(two_coeffs[r] as i64);
                for x in 0..2 {
                    let source = simd_utils::loadu_si128(s_rows[r], xx + x * 8);
                    for i in 0..4 {
                        let c_i64x2 = _mm_shuffle_epi8(source, c_shuffles[i]);
                        sums[i][x] = _mm_add_epi64(sums[i][x], _mm_mul_epi32(c_i64x2, coeff_i64x2));
                    }
                }
            }
            y += 2;
        }

        if let Some(&k) = coeffs_reminder.get(0) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let coeff_i64x2 = _mm_set1_epi64x(k as i64);

            for x in 0..2 {
                let source = simd_utils::loadu_si128(components, xx + x * 8);
                for i in 0..4 {
                    let c_i64x2 = _mm_shuffle_epi8(source, c_shuffles[i]);
                    sums[i][x] = _mm_add_epi64(sums[i][x], _mm_mul_epi32(c_i64x2, coeff_i64x2));
                }
            }
        }

        for x in 0..2 {
            for sum in sums {
                _mm_storeu_si128((&mut c_buf).as_mut_ptr() as *mut __m128i, sum[x]);
                *dst_ptr_u16 = normalizer.clip(c_buf[0]);
                dst_ptr_u16 = dst_ptr_u16.add(1);
                *dst_ptr_u16 = normalizer.clip(c_buf[1]);
                dst_ptr_u16 = dst_ptr_u16.add(1);
            }
        }

        xx += 16;
    }

    // 8 components - 1 = 7
    while xx < src_width.saturating_sub(7) {
        let mut sums = [initial, initial, initial, initial];

        let mut y: u32 = 0;
        let coeffs_2 = coeffs.chunks_exact(2);
        let coeffs_reminder = coeffs_2.remainder();

        for ((s_row0, s_row1), two_coeffs) in src_img.iter_2_rows(y_start, max_y).zip(coeffs_2) {
            let s_rows = [T::components(s_row0), T::components(s_row1)];
            let coeffs_i64 = [
                _mm_set1_epi64x(two_coeffs[0] as i64),
                _mm_set1_epi64x(two_coeffs[1] as i64),
            ];

            for r in 0..2 {
                let source = simd_utils::loadu_si128(s_rows[r], xx);
                for i in 0..4 {
                    let c_i64x2 = _mm_shuffle_epi8(source, c_shuffles[i]);
                    sums[i] = _mm_add_epi64(sums[i], _mm_mul_epi32(c_i64x2, coeffs_i64[r]));
                }
            }
            y += 2;
        }

        if let Some(&k) = coeffs_reminder.get(0) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let coeff_i64x2 = _mm_set1_epi64x(k as i64);
            let source = simd_utils::loadu_si128(components, xx);
            for i in 0..4 {
                let c_i64x2 = _mm_shuffle_epi8(source, c_shuffles[i]);
                sums[i] = _mm_add_epi64(sums[i], _mm_mul_epi32(c_i64x2, coeff_i64x2));
            }
        }

        for sum in sums {
            // let mask = _mm_cmpgt_epi64(sums[i], zero);
            // sums[i] = _mm_and_si128(sums[i] , mask);
            // sums[i] = _mm_srl_epi64(sums[i] , precision_i64);
            // _mm_packus_epi32(sums[i] , sums[i] );
            _mm_storeu_si128((&mut c_buf).as_mut_ptr() as *mut __m128i, sum);
            *dst_ptr_u16 = normalizer.clip(c_buf[0]);
            dst_ptr_u16 = dst_ptr_u16.add(1);
            *dst_ptr_u16 = normalizer.clip(c_buf[1]);
            dst_ptr_u16 = dst_ptr_u16.add(1);
        }

        xx += 8;
    }

    // 4 components - 1 = 3
    while xx < src_width.saturating_sub(3) {
        let mut c01 = initial;
        let mut c23 = initial;
        let mut y: u32 = 0;
        let coeffs_2 = coeffs.chunks_exact(2);
        let coeffs_reminder = coeffs_2.remainder();

        for ((s_row0, s_row1), two_coeffs) in src_img.iter_2_rows(y_start, max_y).zip(coeffs_2) {
            let s_rows = [T::components(s_row0), T::components(s_row1)];
            let coeffs_i64 = [
                _mm_set1_epi64x(two_coeffs[0] as i64),
                _mm_set1_epi64x(two_coeffs[1] as i64),
            ];
            for r in 0..2 {
                let comp_x4 = s_rows[r].get_unchecked(xx..xx + 4);
                let c_i64x2 = _mm_set_epi64x(comp_x4[1] as i64, comp_x4[0] as i64);
                c01 = _mm_add_epi64(c01, _mm_mul_epi32(c_i64x2, coeffs_i64[r]));
                let c_i64x2 = _mm_set_epi64x(comp_x4[3] as i64, comp_x4[2] as i64);
                c23 = _mm_add_epi64(c23, _mm_mul_epi32(c_i64x2, coeffs_i64[r]));
            }
            y += 2;
        }

        if let Some(&k) = coeffs_reminder.get(0) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let coeff_i64x2 = _mm_set1_epi64x(k as i64);

            let comp_x4 = components.get_unchecked(xx..xx + 4);
            let c_i64x2 = _mm_set_epi64x(comp_x4[1] as i64, comp_x4[0] as i64);
            c01 = _mm_add_epi64(c01, _mm_mul_epi32(c_i64x2, coeff_i64x2));
            let c_i64x2 = _mm_set_epi64x(comp_x4[3] as i64, comp_x4[2] as i64);
            c23 = _mm_add_epi64(c23, _mm_mul_epi32(c_i64x2, coeff_i64x2));
        }

        _mm_storeu_si128((&mut c_buf).as_mut_ptr() as *mut __m128i, c01);
        *dst_ptr_u16 = normalizer.clip(c_buf[0]);
        dst_ptr_u16 = dst_ptr_u16.add(1);
        *dst_ptr_u16 = normalizer.clip(c_buf[1]);
        dst_ptr_u16 = dst_ptr_u16.add(1);
        _mm_storeu_si128((&mut c_buf).as_mut_ptr() as *mut __m128i, c23);
        *dst_ptr_u16 = normalizer.clip(c_buf[0]);
        dst_ptr_u16 = dst_ptr_u16.add(1);
        *dst_ptr_u16 = normalizer.clip(c_buf[1]);
        dst_ptr_u16 = dst_ptr_u16.add(1);

        xx += 4;
    }

    if xx < src_width {
        let initial = 1 << (precision - 1);
        convolution_by_u16(
            src_img,
            normalizer,
            initial,
            dst_components,
            xx,
            y_start,
            coeffs,
        );
    }
}
