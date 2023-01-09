use std::arch::wasm32::*;

use crate::pixels::U16x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::native;

pub(crate) unsafe fn multiply_alpha(
    src_image: &ImageView<U16x4>,
    dst_image: &mut ImageViewMut<U16x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(2);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(2);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = v128_load(src.as_ptr() as *const v128);
            let dst_ptr = dst.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_2_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(2);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = v128_load(chunk.as_ptr() as *const v128);
            let dst_ptr = chunk.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_2_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    let remainder = chunks.into_remainder();
    if !remainder.is_empty() {
        native::multiply_alpha_row_inplace(remainder);
    }
}

#[inline]
unsafe fn multiply_alpha_2_pixels(pixels: v128) -> v128 {
    let zero = i64x2_splat(0);
    let half = i32x4_splat(0x8000);
    const MAX_A: i64 = 0xffff000000000000u64 as i64;
    let max_alpha = i64x2_splat(MAX_A);
    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  |
       |0001 0203 0405 0607| |0809 1011 1213 1415|
    */
    const FACTOR_MASK: v128 = i8x16(6, 7, 6, 7, 6, 7, 6, 7, 14, 15, 14, 15, 14, 15, 14, 15);

    let factor_pixels = u8x16_swizzle(pixels, FACTOR_MASK);
    let factor_pixels = v128_or(factor_pixels, max_alpha);

    let src_i32_lo = i16x8_shuffle::<0, 8, 1, 9, 2, 10, 3, 11>(pixels, zero);
    let factors = i16x8_shuffle::<0, 8, 1, 9, 2, 10, 3, 11>(factor_pixels, zero);
    let src_i32_lo = i32x4_add(i32x4_mul(src_i32_lo, factors), half);
    let dst_i32_lo = i32x4_add(src_i32_lo, u32x4_shr(src_i32_lo, 16));
    let dst_i32_lo = u32x4_shr(dst_i32_lo, 16);

    let src_i32_hi = i16x8_shuffle::<4, 12, 5, 13, 6, 14, 7, 15>(pixels, zero);
    let factors = i16x8_shuffle::<4, 12, 5, 13, 6, 14, 7, 15>(factor_pixels, zero);
    let src_i32_hi = i32x4_add(i32x4_mul(src_i32_hi, factors), half);
    let dst_i32_hi = i32x4_add(src_i32_hi, u32x4_shr(src_i32_hi, 16));
    let dst_i32_hi = u32x4_shr(dst_i32_hi, 16);

    u16x8_narrow_i32x4(dst_i32_lo, dst_i32_hi)
}

// Divide

pub(crate) unsafe fn divide_alpha(
    src_image: &ImageView<U16x4>,
    dst_image: &mut ImageViewMut<U16x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(2);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(2);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = v128_load(src.as_ptr() as *const v128);
            let dst_ptr = dst.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_2_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    if let Some(src) = src_remainder.first() {
        let src_pixels = [*src, U16x4::new([0, 0, 0, 0])];
        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 2];

        let mut pixels = v128_load(src_pixels.as_ptr() as *const v128);
        pixels = divide_alpha_2_pixels(pixels);
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);

        let dst_reminder = dst_chunks.into_remainder();
        if let Some(dst) = dst_reminder.get_mut(0) {
            *dst = dst_pixels[0];
        }
    }
}

pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(2);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = v128_load(chunk.as_ptr() as *const v128);
            let dst_ptr = chunk.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_2_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if let Some(pixel) = reminder.first_mut() {
        let src_pixels = [*pixel, U16x4::new([0, 0, 0, 0])];
        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 2];

        let mut pixels = v128_load(src_pixels.as_ptr() as *const v128);
        pixels = divide_alpha_2_pixels(pixels);
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);
        *pixel = dst_pixels[0];
    }
}

#[inline]
unsafe fn divide_alpha_2_pixels(pixels: v128) -> v128 {
    let zero = i64x2_splat(0);
    let alpha_mask = i64x2_splat(0xffff000000000000u64 as i64);
    let alpha_max = f32x4_splat(65535.0);
    let alpha_scale_max = f32x4_splat(2147483648f32);
    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  |
       |0001 0203 0405 0607| |0809 1011 1213 1415|
    */
    const ALPHA32_SH0: v128 = i8x16(6, 7, -1, -1, 6, 7, -1, -1, 6, 7, -1, -1, 6, 7, -1, -1);
    const ALPHA32_SH1: v128 = i8x16(
        14, 15, -1, -1, 14, 15, -1, -1, 14, 15, -1, -1, 14, 15, -1, -1,
    );

    let alpha0_f32x4 = f32x4_convert_i32x4(u8x16_swizzle(pixels, ALPHA32_SH0));
    let alpha1_f32x4 = f32x4_convert_i32x4(u8x16_swizzle(pixels, ALPHA32_SH1));

    let pix0_f32x4 = f32x4_convert_i32x4(i16x8_shuffle::<0, 8, 1, 9, 2, 10, 3, 11>(pixels, zero));
    let pix1_f32x4 = f32x4_convert_i32x4(i16x8_shuffle::<4, 12, 5, 13, 6, 14, 7, 15>(pixels, zero));

    let scaled_pix0_f32x4 = f32x4_mul(pix0_f32x4, alpha_max);
    let scaled_pix1_f32x4 = f32x4_mul(pix1_f32x4, alpha_max);

    let divided_pix0_i32x4 = u32x4_trunc_sat_f32x4(f32x4_pmin(
        f32x4_div(scaled_pix0_f32x4, alpha0_f32x4),
        alpha_scale_max,
    ));
    let divided_pix1_i32x4 = u32x4_trunc_sat_f32x4(f32x4_pmin(
        f32x4_div(scaled_pix1_f32x4, alpha1_f32x4),
        alpha_scale_max,
    ));

    let two_pixels_i16x8 = u16x8_narrow_i32x4(divided_pix0_i32x4, divided_pix1_i32x4);
    let alpha = v128_and(pixels, alpha_mask);
    u8x16_shuffle::<0, 1, 2, 3, 4, 5, 22, 23, 8, 9, 10, 11, 12, 13, 30, 31>(two_pixels_i16x8, alpha)
}
