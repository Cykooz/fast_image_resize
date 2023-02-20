use std::arch::wasm32::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U8x2;
use crate::wasm32_utils;
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
#[inline]
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U8x2]; 4],
    dst_rows: [&mut &mut [U8x2]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let precision = normalizer.precision();
    let initial = i32x4_splat(1 << (precision - 2));

    /*
        |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
        |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|

        Shuffle components with converting from u8 into i16:

        A: |-1 07| |-1 05| |-1 03| |-1 01|
        L: |-1 06| |-1 04| |-1 02| |-1 00|
    */
    const SH1: v128 = i8x16(0, -1, 2, -1, 4, -1, 6, -1, 1, -1, 3, -1, 5, -1, 7, -1);
    /*
        A: |-1 15| |-1 13| |-1 11| |-1 09|
        L: |-1 14| |-1 12| |-1 10| |-1 08|
    */
    const SH2: v128 = i8x16(8, -1, 10, -1, 12, -1, 14, -1, 9, -1, 11, -1, 13, -1, 15, -1);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x = coeffs_chunk.start as usize;

        let mut sss: [v128; 4] = [initial; 4];
        let coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        let reminder = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let mmk0 = wasm32_utils::ptr_i16_to_set1_i64(k, 0);
            let mmk1 = wasm32_utils::ptr_i16_to_set1_i64(k, 4);

            for i in 0..4 {
                let source = wasm32_utils::load_v128(src_rows[i], x);
                let pix = i8x16_swizzle(source, SH1);
                let tmp_sum = i32x4_add(sss[i], i32x4_dot_i16x8(pix, mmk0));
                let pix = i8x16_swizzle(source, SH2);
                sss[i] = i32x4_add(tmp_sum, i32x4_dot_i16x8(pix, mmk1));
            }
            x += 8;
        }

        let coeffs_by_4 = reminder.chunks_exact(4);
        let reminder = coeffs_by_4.remainder();
        for k in coeffs_by_4 {
            let mmk = wasm32_utils::ptr_i16_to_set1_i64(k, 0);

            for i in 0..4 {
                let source = wasm32_utils::loadl_i64(src_rows[i], x);
                let pix = i8x16_swizzle(source, SH1);
                sss[i] = i32x4_add(sss[i], i32x4_dot_i16x8(pix, mmk));
            }
            x += 4;
        }

        let coeffs_by_2 = reminder.chunks_exact(2);
        let reminder = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(k, 0);

            for i in 0..4 {
                let source = wasm32_utils::loadl_i32(src_rows[i], x);
                let pix = i8x16_swizzle(source, SH1);
                sss[i] = i32x4_add(sss[i], i32x4_dot_i16x8(pix, mmk));
            }
            x += 2;
        }

        if let Some(&k) = reminder.first() {
            let mmk = i32x4_splat(k as i32);

            for i in 0..4 {
                let source = wasm32_utils::loadl_i16(src_rows[i], x);
                let pix = i8x16_swizzle(source, SH1);
                sss[i] = i32x4_add(sss[i], i32x4_dot_i16x8(pix, mmk));
            }
        }

        for i in 0..4 {
            set_dst_pixel(sss[i], dst_rows[i], dst_x, normalizer);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_one_row(
    src_row: &[U8x2],
    dst_row: &mut [U8x2],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    const SH1: v128 = i8x16(0, -1, 2, -1, 4, -1, 6, -1, 1, -1, 3, -1, 5, -1, 7, -1);
    /*
        A: |-1 15| |-1 13| |-1 11| |-1 09|
        L: |-1 14| |-1 12| |-1 10| |-1 08|
    */
    const SH2: v128 = i8x16(8, -1, 10, -1, 12, -1, 14, -1, 9, -1, 11, -1, 13, -1, 15, -1);

    // Lower part will be added to higher, use only half of the error
    let precision = normalizer.precision();
    let initial = i32x4_splat(1 << (precision - 2));

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x = coeffs_chunk.start as usize;
        let mut coeffs = coeffs_chunk.values;

        let mut sss = initial;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let mmk0 = wasm32_utils::ptr_i16_to_set1_i64(k, 0);
            let mmk1 = wasm32_utils::ptr_i16_to_set1_i64(k, 4);

            let source = wasm32_utils::load_v128(src_row, x);
            let pix = i8x16_swizzle(source, SH1);
            let tmp_sum = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk0));
            let pix = i8x16_swizzle(source, SH2);
            sss = i32x4_add(tmp_sum, i32x4_dot_i16x8(pix, mmk1));

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        let reminder1 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let mmk = wasm32_utils::ptr_i16_to_set1_i64(k, 0);
            let source = wasm32_utils::loadl_i64(src_row, x);
            let pix = i8x16_swizzle(source, SH1);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));
            x += 4
        }

        let coeffs_by_2 = reminder1.chunks_exact(2);
        let reminder = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(k, 0);

            let source = wasm32_utils::loadl_i32(src_row, x);
            let pix = i8x16_swizzle(source, SH1);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            x += 2;
        }

        if let Some(&k) = reminder.first() {
            let mmk = i32x4_splat(k as i32);
            let source = wasm32_utils::loadl_i16(src_row, x);
            let pix = i8x16_swizzle(source, SH1);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));
        }

        set_dst_pixel(sss, dst_row, dst_x, normalizer);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn set_dst_pixel(
    raw: v128,
    d_row: &mut [U8x2],
    dst_x: usize,
    normalizer: &optimisations::Normalizer16,
) {
    let mut buf = [0i32; 4];
    v128_store(buf.as_mut_ptr() as _, raw);
    let l32 = buf[0].saturating_add(buf[1]);
    let a32 = buf[2].saturating_add(buf[3]);
    let l8 = normalizer.clip(l32);
    let a8 = normalizer.clip(a32);
    d_row.get_unchecked_mut(dst_x).0 = u16::from_le_bytes([l8, a8]);
}
