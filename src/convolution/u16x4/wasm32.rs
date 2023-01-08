use std::arch::wasm32::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U16x4;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U16x4>,
    dst_image: &mut ImageViewMut<U16x4>,
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
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U16x4]; 4],
    dst_rows: [&mut &mut [U16x4]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut rg_buf = [0i64; 2];
    let mut ba_buf = [0i64; 2];

    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  |
       |0001 0203 0405 0607| |0809 1011 1213 1415|

        Shuffle to extract R0 and G0 as i64:
        0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1

        Shuffle to extract R1 and G1 as i64:
        8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1

        Shuffle to extract B0 and A0 as i64:
        4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1

        Shuffle to extract B1 and A1 as i64:
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1
    */

    let rg0_shuffle = i8x16(0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1);
    let rg1_shuffle = i8x16(8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1);
    let ba0_shuffle = i8x16(4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1);
    let ba1_shuffle = i8x16(
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sum = [i64x2_splat(half_error); 4];
        let mut ba_sum = [i64x2_splat(half_error); 4];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff0_i64x2 = i64x2_splat(k[0] as i64);
            let coeff1_i64x2 = i64x2_splat(k[1] as i64);

            for i in 0..4 {
                let source = wasm32_utils::load_v128(src_rows[i], x);
                let mut sum = rg_sum[i];
                let rg_i64x2 = i8x16_swizzle(source, rg0_shuffle);
                sum = i64x2_add(sum, i64x2_mul(rg_i64x2, coeff0_i64x2));
                let rg_i64x2 = i8x16_swizzle(source, rg1_shuffle);
                sum = i64x2_add(sum, i64x2_mul(rg_i64x2, coeff1_i64x2));
                rg_sum[i] = sum;

                let mut sum = ba_sum[i];
                let ba_i64x2 = i8x16_swizzle(source, ba0_shuffle);
                sum = i64x2_add(sum, i64x2_mul(ba_i64x2, coeff0_i64x2));
                let ba_i64x2 = i8x16_swizzle(source, ba1_shuffle);
                sum = i64x2_add(sum, i64x2_mul(ba_i64x2, coeff1_i64x2));
                ba_sum[i] = sum;
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_i64x2 = i64x2_splat(k as i64);
            for i in 0..4 {
                let source = wasm32_utils::loadl_i64(src_rows[i], x);
                let rg_i64x2 = i8x16_swizzle(source, rg0_shuffle);
                rg_sum[i] = i64x2_add(rg_sum[i], i64x2_mul(rg_i64x2, coeff0_i64x2));
                let ba_i64x2 = i8x16_swizzle(source, ba0_shuffle);
                ba_sum[i] = i64x2_add(ba_sum[i], i64x2_mul(ba_i64x2, coeff0_i64x2));
            }
        }

        for i in 0..4 {
            v128_store((&mut rg_buf).as_mut_ptr() as *mut v128, rg_sum[i]);
            v128_store((&mut ba_buf).as_mut_ptr() as *mut v128, ba_sum[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = [
                normalizer.clip(rg_buf[0]),
                normalizer.clip(rg_buf[1]),
                normalizer.clip(ba_buf[0]),
                normalizer.clip(ba_buf[1]),
            ];
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
unsafe fn horiz_convolution_one_row(
    src_row: &[U16x4],
    dst_row: &mut [U16x4],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    normalizer: &optimisations::Normalizer32,
) {
    let precision = normalizer.precision();
    let half_error = 1i64 << (precision - 1);
    let mut rg_buf = [0i64; 2];
    let mut ba_buf = [0i64; 2];

    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  |
       |0001 0203 0405 0607| |0809 1011 1213 1415|

        Shuffle to extract R0 and G0 as i64:
        0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1

        Shuffle to extract R1 and G1 as i64:
        8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1

        Shuffle to extract B0 and A0 as i64:
        4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1

        Shuffle to extract B1 and A1 as i64:
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1
    */

    let rg0_shuffle = i8x16(0, 1, -1, -1, -1, -1, -1, -1, 2, 3, -1, -1, -1, -1, -1, -1);
    let rg1_shuffle = i8x16(8, 9, -1, -1, -1, -1, -1, -1, 10, 11, -1, -1, -1, -1, -1, -1);
    let ba0_shuffle = i8x16(4, 5, -1, -1, -1, -1, -1, -1, 6, 7, -1, -1, -1, -1, -1, -1);
    let ba1_shuffle = i8x16(
        12, 13, -1, -1, -1, -1, -1, -1, 14, 15, -1, -1, -1, -1, -1, -1,
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut coeffs = coeffs_chunk.values;
        let mut rg_sum = i64x2_splat(half_error);
        let mut ba_sum = i64x2_splat(half_error);

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff0_i64x2 = i64x2_splat(k[0] as i64);
            let coeff1_i64x2 = i64x2_splat(k[1] as i64);

            let source = wasm32_utils::load_v128(src_row, x);

            let rg_i64x2 = i8x16_swizzle(source, rg0_shuffle);
            rg_sum = i64x2_add(rg_sum, i64x2_mul(rg_i64x2, coeff0_i64x2));
            let rg_i64x2 = i8x16_swizzle(source, rg1_shuffle);
            rg_sum = i64x2_add(rg_sum, i64x2_mul(rg_i64x2, coeff1_i64x2));

            let ba_i64x2 = i8x16_swizzle(source, ba0_shuffle);
            ba_sum = i64x2_add(ba_sum, i64x2_mul(ba_i64x2, coeff0_i64x2));
            let ba_i64x2 = i8x16_swizzle(source, ba1_shuffle);
            ba_sum = i64x2_add(ba_sum, i64x2_mul(ba_i64x2, coeff1_i64x2));

            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_i64x2 = i64x2_splat(k as i64);
            let source = wasm32_utils::loadl_i64(src_row, x);
            let rg_i64x2 = i8x16_swizzle(source, rg0_shuffle);
            rg_sum = i64x2_add(rg_sum, i64x2_mul(rg_i64x2, coeff0_i64x2));
            let ba_i64x2 = i8x16_swizzle(source, ba0_shuffle);
            ba_sum = i64x2_add(ba_sum, i64x2_mul(ba_i64x2, coeff0_i64x2));
        }

        v128_store((&mut rg_buf).as_mut_ptr() as *mut v128, rg_sum);
        v128_store((&mut ba_buf).as_mut_ptr() as *mut v128, ba_sum);
        let dst_pixel = dst_row.get_unchecked_mut(dst_x);
        dst_pixel.0 = [
            normalizer.clip(rg_buf[0]),
            normalizer.clip(rg_buf[1]),
            normalizer.clip(ba_buf[0]),
            normalizer.clip(ba_buf[1]),
        ];
    }
}
