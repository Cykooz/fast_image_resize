use std::arch::wasm32::*;

use crate::pixels::U16x2;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::native;

pub(crate) unsafe fn multiply_alpha(
    src_image: &ImageView<U16x2>,
    dst_image: &mut ImageViewMut<U16x2>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x2>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U16x2], dst_row: &mut [U16x2]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = v128_load(src.as_ptr() as *const v128);
            let dst_ptr = dst.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiplies_alpha_4_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U16x2]) {
    let mut chunks = row.chunks_exact_mut(4);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = v128_load(chunk.as_ptr() as *const v128);
            let dst_ptr = chunk.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiplies_alpha_4_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
unsafe fn multiplies_alpha_4_pixels(pixels: v128) -> v128 {
    const HALF: v128 = i32x4(0x8000, 0x8000, 0x8000, 0x8000);

    const MAX_ALPHA: v128 = i32x4(
        0xffff0000u32 as i32,
        0xffff0000u32 as i32,
        0xffff0000u32 as i32,
        0xffff0000u32 as i32,
    );
    /*
       |L0   A0  | |L1   A1  | |L2   A2  | |L3   A3  |
       |0001 0203| |0405 0607| |0809 1011| |1213 1415|
    */
    const FACTOR_MASK: v128 = i8x16(2, 3, 2, 3, 6, 7, 6, 7, 10, 11, 10, 11, 14, 15, 14, 15);

    let factor_pixels = u8x16_swizzle(pixels, FACTOR_MASK);
    let factor_pixels = v128_or(factor_pixels, MAX_ALPHA);

    let src_u32_lo = u32x4_extend_low_u16x8(pixels);
    let factors = u32x4_extend_low_u16x8(factor_pixels);
    let src_i32_lo = i32x4_add(i32x4_mul(src_u32_lo, factors), HALF);
    let dst_i32_lo = i32x4_add(src_i32_lo, u32x4_shr(src_i32_lo, 16));
    let dst_i32_lo = u32x4_shr(dst_i32_lo, 16);

    let src_u32_hi = u32x4_extend_high_u16x8(pixels);
    let factors = u32x4_extend_high_u16x8(factor_pixels);
    let src_i32_hi = i32x4_add(i32x4_mul(src_u32_hi, factors), HALF);
    let dst_i32_hi = i32x4_add(src_i32_hi, u32x4_shr(src_i32_hi, 16));
    let dst_i32_hi = u32x4_shr(dst_i32_hi, 16);

    u16x8_narrow_i32x4(dst_i32_lo, dst_i32_hi)
}

// Divide

pub(crate) unsafe fn divide_alpha(
    src_image: &ImageView<U16x2>,
    dst_image: &mut ImageViewMut<U16x2>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U16x2>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x2], dst_row: &mut [U16x2]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = v128_load(src.as_ptr() as *const v128);
            let dst_ptr = dst.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_4_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U16x2::new([0, 0]); 4];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x2::new([0, 0]); 4];
        let mut pixels = v128_load(src_pixels.as_ptr() as *const v128);
        pixels = divide_alpha_4_pixels(pixels);
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U16x2]) {
    let mut chunks = row.chunks_exact_mut(4);
    // Using a simple for-loop in this case is faster than implementation with pre-reading
    for chunk in &mut chunks {
        let mut pixels = v128_load(chunk.as_ptr() as *const v128);
        pixels = divide_alpha_4_pixels(pixels);
        v128_store(chunk.as_mut_ptr() as *mut v128, pixels);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        let mut src_pixels = [U16x2::new([0, 0]); 4];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x2::new([0, 0]); 4];
        let mut pixels = v128_load(src_pixels.as_ptr() as *const v128);
        pixels = divide_alpha_4_pixels(pixels);
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
unsafe fn divide_alpha_4_pixels(pixels: v128) -> v128 {
    const ALPHA_MASK: v128 = i32x4(
        0xffff0000u32 as i32,
        0xffff0000u32 as i32,
        0xffff0000u32 as i32,
        0xffff0000u32 as i32,
    );
    const LUMA_MASK: v128 = i32x4(0xffff, 0xffff, 0xffff, 0xffff);
    const ALPHA_MAX: v128 = f32x4(65535.0, 65535.0, 65535.0, 65535.0);
    const ALPHA_SCALE_MAX: v128 = f32x4(2147483648f32, 2147483648f32, 2147483648f32, 2147483648f32);
    /*
       |L0   A0  | |L1   A1  | |L2   A2  | |L3   A3  |
       |0001 0203| |0405 0607| |0809 1011| |1213 1415|
    */
    const ALPHA32_SH: v128 = i8x16(2, 3, -1, -1, 6, 7, -1, -1, 10, 11, -1, -1, 14, 15, -1, -1);

    let alpha_f32x4 = f32x4_convert_i32x4(u8x16_swizzle(pixels, ALPHA32_SH));
    let luma_f32x4 = f32x4_convert_i32x4(v128_and(pixels, LUMA_MASK));
    let scaled_luma_f32x4 = f32x4_mul(luma_f32x4, ALPHA_MAX);
    let divided_luma_u32x4 = u32x4_trunc_sat_f32x4(f32x4_pmin(
        f32x4_div(scaled_luma_f32x4, alpha_f32x4),
        ALPHA_SCALE_MAX,
    ));

    let alpha = v128_and(pixels, ALPHA_MASK);
    u8x16_shuffle::<0, 1, 18, 19, 4, 5, 22, 23, 8, 9, 26, 27, 12, 13, 30, 31>(
        divided_luma_u32x4,
        alpha,
    )
}
