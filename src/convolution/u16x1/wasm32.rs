use std::arch::wasm32::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U16;
use crate::wasm32_utils;
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
#[target_feature(enable = "simd128")]
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
        0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1

        Shuffle to extract L2 and L3 as i64:
        4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1

        Shuffle to extract L4 and L5 as i64:
        8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1

        Shuffle to extract L6 and L7 as i64:
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1
    */

    const L0L1_SHUFFLE: v128 = i8x16(0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1);
    const L2L3_SHUFFLE: v128 = i8x16(4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1);
    const L4L5_SHUFFLE: v128 = i8x16(8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1);
    const L6L7_SHUFFLE: v128 = i8x16(
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum: [v128; 4] = [i64x2_splat(0i64); 4];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
            let coeff23_i64x2 = i64x2(k[2] as i64, k[3] as i64);
            let coeff45_i64x2 = i64x2(k[4] as i64, k[5] as i64);
            let coeff67_i64x2 = i64x2(k[6] as i64, k[7] as i64);

            for i in 0..4 {
                let mut sum = ll_sum[i];
                let source = wasm32_utils::load_v128(src_rows[i], x);

                let l0l1_i64x2 = i8x16_swizzle(source, L0L1_SHUFFLE);
                sum = i64x2_add(sum, wasm32_utils::i64x2_mul_lo(l0l1_i64x2, coeff01_i64x2));

                let l2l3_i64x2 = i8x16_swizzle(source, L2L3_SHUFFLE);
                sum = i64x2_add(sum, wasm32_utils::i64x2_mul_lo(l2l3_i64x2, coeff23_i64x2));

                let l4l5_i64x2 = i8x16_swizzle(source, L4L5_SHUFFLE);
                sum = i64x2_add(sum, wasm32_utils::i64x2_mul_lo(l4l5_i64x2, coeff45_i64x2));

                let l6l7_i64x2 = i8x16_swizzle(source, L6L7_SHUFFLE);
                sum = i64x2_add(sum, wasm32_utils::i64x2_mul_lo(l6l7_i64x2, coeff67_i64x2));

                ll_sum[i] = sum;
            }
            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
            let coeff23_i64x2 = i64x2(k[2] as i64, k[3] as i64);

            for i in 0..4 {
                let mut sum = ll_sum[i];
                let source = wasm32_utils::load_v128(src_rows[i], x);

                let l0l1_i64x2 = i8x16_swizzle(source, L0L1_SHUFFLE);
                sum = i64x2_add(sum, wasm32_utils::i64x2_mul_lo(l0l1_i64x2, coeff01_i64x2));

                let l2l3_i64x2 = i8x16_swizzle(source, L2L3_SHUFFLE);
                sum = i64x2_add(sum, wasm32_utils::i64x2_mul_lo(l2l3_i64x2, coeff23_i64x2));

                ll_sum[i] = sum;
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
            for i in 0..4 {
                let source = wasm32_utils::load_v128(src_rows[i], x);
                let l_i64x2 = i8x16_swizzle(source, L0L1_SHUFFLE);
                ll_sum[i] = i64x2_add(
                    ll_sum[i],
                    wasm32_utils::i64x2_mul_lo(l_i64x2, coeff01_i64x2),
                );
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff01_i64x2 = i64x2(k as i64, 0);
            for i in 0..4 {
                let pixel = (*src_rows[i].get_unchecked(x)).0 as i64;
                let source = i64x2(pixel, 0);
                ll_sum[i] = i64x2_add(ll_sum[i], wasm32_utils::i64x2_mul_lo(source, coeff01_i64x2));
            }
        }

        for i in 0..4 {
            v128_store((&mut ll_buf).as_mut_ptr() as *mut v128, ll_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = normalizer.clip(ll_buf.iter().sum::<i64>() + half_error);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
#[target_feature(enable = "simd128")]
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
        0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1

        Shuffle to extract L2 and L3 as i64:
        4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1

        Shuffle to extract L4 and L5 as i64:
        8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1

        Shuffle to extract L6 and L7 as i64:
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1
    */

    const L01_SHUFFLE: v128 = i8x16(0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1);
    const L23_SHUFFLE: v128 = i8x16(4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1);
    const L45_SHUFFLE: v128 = i8x16(8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1);
    const L67_SHUFFLE: v128 = i8x16(
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut ll_sum = i64x2_splat(0);
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
            let coeff23_i64x2 = i64x2(k[2] as i64, k[3] as i64);
            let coeff45_i64x2 = i64x2(k[4] as i64, k[5] as i64);
            let coeff67_i64x2 = i64x2(k[6] as i64, k[7] as i64);

            let source = wasm32_utils::load_v128(src_row, x);

            let l_i64x2 = i8x16_swizzle(source, L01_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff01_i64x2));

            let l_i64x2 = i8x16_swizzle(source, L23_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff23_i64x2));

            let l_i64x2 = i8x16_swizzle(source, L45_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff45_i64x2));

            let l_i64x2 = i8x16_swizzle(source, L67_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff67_i64x2));

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
            let coeff23_i64x2 = i64x2(k[2] as i64, k[3] as i64);

            let source = wasm32_utils::load_v128(src_row, x);

            let l_i64x2 = i8x16_swizzle(source, L01_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff01_i64x2));

            let l_i64x2 = i8x16_swizzle(source, L23_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff23_i64x2));

            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff01_i64x2 = i64x2(k[0] as i64, k[1] as i64);
            let source = wasm32_utils::load_v128(src_row, x);

            let l_i64x2 = i8x16_swizzle(source, L01_SHUFFLE);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(l_i64x2, coeff01_i64x2));

            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff01_i64x2 = i64x2(k as i64, 0);
            let pixel = (*src_row.get_unchecked(x)).0 as i64;
            let source = i64x2(pixel, 0);
            ll_sum = i64x2_add(ll_sum, wasm32_utils::i64x2_mul_lo(source, coeff01_i64x2));
        }

        v128_store((&mut ll_buf).as_mut_ptr() as *mut v128, ll_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0 = normalizer.clip(ll_buf[0] + ll_buf[1] + half_error);
    }
}
