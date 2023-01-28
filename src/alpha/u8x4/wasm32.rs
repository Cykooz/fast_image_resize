use std::arch::wasm32::*;

use crate::pixels::U8x4;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

use super::native;

pub(crate) unsafe fn multiply_alpha(
    src_image: &ImageView<U8x4>,
    dst_image: &mut ImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    // A simple for-loop in this case is as fast as implementation with pre-reading
    for (src, dst) in src_dst {
        let mut pixels = v128_load(src.as_ptr() as *const v128);
        pixels = multiply_alpha_4_pixels(pixels);
        v128_store(dst.as_mut_ptr() as *mut v128, pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    // A simple for-loop in this case is as fast as implementation with pre-reading
    for chunk in &mut chunks {
        let mut pixels = v128_load(chunk.as_ptr() as *const v128);
        pixels = multiply_alpha_4_pixels(pixels);
        v128_store(chunk.as_mut_ptr() as *mut v128, pixels);
    }

    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        native::multiply_alpha_row_inplace(tail);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_4_pixels(pixels: v128) -> v128 {
    let half = u16x8_splat(128);
    let max_alpha = u32x4_splat(0xff000000);
    const FACTOR_MASK: v128 = i8x16(3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15);

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

pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x4>, dst_image: &mut ImageViewMut<U8x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    // A simple for-loop in this case is faster than implementation with pre-reading
    for (src, dst) in src_dst {
        let mut pixels = v128_load(src.as_ptr() as *const v128);
        pixels = divide_alpha_4_pixels(pixels);
        v128_store(dst.as_mut_ptr() as *mut v128, pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_buffer = [U8x4::new(0); 4];
        src_buffer
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_buffer = [U8x4::new(0); 4];
        let src_pixels = v128_load(src_buffer.as_ptr() as *const v128);
        let dst_pixels = divide_alpha_4_pixels(src_pixels);
        v128_store(dst_buffer.as_mut_ptr() as *mut v128, dst_pixels);

        dst_buffer
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    // A simple for-loop in this case is faster than implementation with pre-reading
    for chunk in &mut chunks {
        let mut pixels = v128_load(chunk.as_ptr() as *const v128);
        pixels = divide_alpha_4_pixels(pixels);
        v128_store(chunk.as_mut_ptr() as *mut v128, pixels);
    }

    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        let mut src_buffer = [U8x4::new(0); 4];
        src_buffer
            .iter_mut()
            .zip(tail.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_buffer = [U8x4::new(0); 4];
        let src_pixels = v128_load(src_buffer.as_ptr() as *const v128);
        let dst_pixels = divide_alpha_4_pixels(src_pixels);
        v128_store(dst_buffer.as_mut_ptr() as *mut v128, dst_pixels);

        dst_buffer.iter().zip(tail).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_4_pixels(pixels: v128) -> v128 {
    const FACTOR_LO_SHUFFLE: v128 = i8x16(0, 1, 0, 1, 0, 1, -1, -1, 2, 3, 2, 3, 2, 3, -1, -1);
    const FACTOR_HI_SHUFFLE: v128 = i8x16(4, 5, 4, 5, 4, 5, -1, -1, 6, 7, 6, 7, 6, 7, -1, -1);
    let alpha_mask = u32x4_splat(0xff000000);
    let alpha_scale = f32x4_splat(255.0 * 256.0);

    let alpha_f32 = f32x4_convert_i32x4(u32x4_shr(pixels, 24));
    // In case of zero division the result will be u32::MAX or 0.
    let scaled_alpha_u32 = u32x4_trunc_sat_f32x4(f32x4_div(alpha_scale, alpha_f32));
    // All u32::MAX values in arguments will interpreted as -1i32.
    // u16x8_narrow_i32x4() converts all negative values into 0.
    let scaled_alpha_u16 = u16x8_narrow_i32x4(scaled_alpha_u32, scaled_alpha_u32);
    let factor_lo_u16x8 = u8x16_swizzle(scaled_alpha_u16, FACTOR_LO_SHUFFLE);
    let factor_hi_u16x8 = u8x16_swizzle(scaled_alpha_u16, FACTOR_HI_SHUFFLE);

    // alpha_mask's first byte is 0
    let src_u16_lo =
        u8x16_shuffle::<0, 16, 0, 17, 0, 18, 0, 19, 0, 20, 0, 21, 0, 22, 0, 23>(alpha_mask, pixels);
    let src_u16_hi =
        u8x16_shuffle::<0, 24, 0, 25, 0, 26, 0, 27, 0, 28, 0, 29, 0, 30, 0, 31>(alpha_mask, pixels);

    let dst_lo = wasm32_utils::u16x8_mul_shr16(src_u16_lo, factor_lo_u16x8);
    let dst_hi = wasm32_utils::u16x8_mul_shr16(src_u16_hi, factor_hi_u16x8);

    let alpha = v128_and(pixels, alpha_mask);
    let rgb = u8x16_narrow_i16x8(dst_lo, dst_hi);
    v128_or(rgb, alpha)
}
