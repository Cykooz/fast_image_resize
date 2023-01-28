use std::arch::wasm32::*;

use crate::pixels::U8x2;
use crate::{ImageView, ImageViewMut};

use super::native;

pub(crate) unsafe fn multiply_alpha(
    src_image: &ImageView<U8x2>,
    dst_image: &mut ImageViewMut<U8x2>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    // A simple for-loop in this case is as fast as implementation with pre-reading
    for (src, dst) in src_dst {
        let src_pixels = v128_load(src.as_ptr() as *const v128);
        let dst_pixels = multiplies_alpha_8_pixels(src_pixels);
        v128_store(dst.as_mut_ptr() as *mut v128, dst_pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    // Using a simple for-loop in this case is as fast as implementation with pre-reading
    for chunk in &mut chunks {
        let src_pixels = v128_load(chunk.as_ptr() as *const v128);
        let dst_pixels = multiplies_alpha_8_pixels(src_pixels);
        v128_store(chunk.as_mut_ptr() as *mut v128, dst_pixels);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiplies_alpha_8_pixels(pixels: v128) -> v128 {
    let half = u16x8_splat(128);
    let max_alpha = u16x8_splat(0xff00);
    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|
    */
    const FACTOR_MASK: v128 = i8x16(1, 1, 3, 3, 5, 5, 7, 7, 9, 9, 11, 11, 13, 13, 15, 15);

    let factor_pixels = u8x16_swizzle(pixels, FACTOR_MASK);
    let factor_pixels = v128_or(factor_pixels, max_alpha);

    let src_u16_lo = u16x8_extend_low_u8x16(pixels);
    let factors = u16x8_extend_low_u8x16(factor_pixels);
    let mut dst_u16_lo = u16x8_add(u16x8_mul(src_u16_lo, factors), half);
    dst_u16_lo = u16x8_add(dst_u16_lo, u16x8_shr(dst_u16_lo, 8));
    dst_u16_lo = u16x8_shr(dst_u16_lo, 8);

    let src_u16_hi = u16x8_extend_high_u8x16(pixels);
    let factors = u16x8_extend_high_u8x16(factor_pixels);
    let mut dst_u16_hi = u16x8_add(u16x8_mul(src_u16_hi, factors), half);
    dst_u16_hi = u16x8_add(dst_u16_hi, u16x8_shr(dst_u16_hi, 8));
    dst_u16_hi = u16x8_shr(dst_u16_hi, 8);

    u8x16_narrow_i16x8(dst_u16_lo, dst_u16_hi)
}

// Divide

pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x2>, dst_image: &mut ImageViewMut<U8x2>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    // Using a simple for-loop in this case is as fast as implementation with pre-reading
    for (src, dst) in src_dst {
        let src_pixels = v128_load(src.as_ptr() as *const v128);
        let dst_pixels = divide_alpha_8_pixels(src_pixels);
        v128_store(dst.as_mut_ptr() as *mut v128, dst_pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x2::new(0); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2::new(0); 8];
        let mut pixels = v128_load(src_pixels.as_ptr() as *const v128);
        pixels = divide_alpha_8_pixels(pixels);
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    // Using a simple for-loop in this case is as fast as implementation with pre-reading
    for chunk in &mut chunks {
        let src_pixels = v128_load(chunk.as_ptr() as *const v128);
        let dst_pixels = divide_alpha_8_pixels(src_pixels);
        v128_store(chunk.as_mut_ptr() as *mut v128, dst_pixels);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        let mut src_pixels = [U8x2::new(0); 8];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2::new(0); 8];
        let mut pixels = v128_load(src_pixels.as_ptr() as *const v128);
        pixels = divide_alpha_8_pixels(pixels);
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_8_pixels(pixels: v128) -> v128 {
    let alpha_mask = i16x8_splat(0xff00u16 as i16);
    let luma_mask = i16x8_splat(0xff);
    const ALPHA32_SH_LO: v128 = i8x16(1, -1, -1, -1, 3, -1, -1, -1, 5, -1, -1, -1, 7, -1, -1, -1);
    const ALPHA32_SH_HI: v128 = i8x16(
        9, -1, -1, -1, 11, -1, -1, -1, 13, -1, -1, -1, 15, -1, -1, -1,
    );
    let alpha_scale = f32x4_splat(255.0 * 256.0);

    let alpha_lo_f32 = f32x4_convert_u32x4(u8x16_swizzle(pixels, ALPHA32_SH_LO));
    // In case of zero division the result will be u32::MAX or 0.
    let scaled_alpha_lo_u32 = u32x4_trunc_sat_f32x4(f32x4_div(alpha_scale, alpha_lo_f32));

    let alpha_hi_f32 = f32x4_convert_u32x4(i8x16_swizzle(pixels, ALPHA32_SH_HI));
    let scaled_alpha_hi_u32 = u32x4_trunc_sat_f32x4(f32x4_div(alpha_scale, alpha_hi_f32));

    // All u32::MAX values in arguments will interpreted as -1i32.
    // u16x8_narrow_i32x4() converts all negative values into 0.
    let scaled_alpha_u16 = u16x8_narrow_i32x4(scaled_alpha_lo_u32, scaled_alpha_hi_u32);

    let luma_u16 = v128_and(pixels, luma_mask);
    let scaled_luma_u16 = u16x8_mul(luma_u16, scaled_alpha_u16);
    let scaled_luma_u16 = u16x8_shr(scaled_luma_u16, 8);

    // Blend scaled luma with original alpha channel.
    let alpha = v128_and(pixels, alpha_mask);
    v128_or(scaled_luma_u16, alpha)
}
