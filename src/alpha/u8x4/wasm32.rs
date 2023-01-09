use std::arch::wasm32::*;

use crate::pixels::U8x4;
use crate::utils::foreach_with_pre_reading;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};
use std::fs;
use std::path::Path;

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
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
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
            pixels = multiply_alpha_4_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    // Using a simple for-loop in this case is faster than implementation with pre-reading
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
unsafe fn multiply_alpha_4_pixels(pixels: v128) -> v128 {
    let zero = i64x2_splat(0);
    let half = i16x8_splat(128);

    const MAX_A: i32 = 0xff000000u32 as i32;
    let max_alpha = i32x4_splat(MAX_A);
    let factor_mask = i8x16(15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3);

    let factor_pixels = u8x16_swizzle(pixels, factor_mask);
    let factor_pixels = v128_or(factor_pixels, max_alpha);

    let pix1 =
        i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(pixels, zero);
    let factors = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
        factor_pixels,
        zero,
    );
    let pix1 = i16x8_add(i16x8_mul(pix1, factors), half);
    let pix1 = i16x8_add(pix1, u16x8_shr(pix1, 8));
    let pix1 = u16x8_shr(pix1, 8);

    let pix2 =
        i8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(pixels, zero);
    let factors = i8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(
        factor_pixels,
        zero,
    );
    let pix2 = i16x8_add(i16x8_mul(pix2, factors), half);
    let pix2 = i16x8_add(pix2, u16x8_shr(pix2, 8));
    let pix2 = u16x8_shr(pix2, 8);

    u8x16_narrow_i16x8(pix1, pix2)
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
pub(crate) unsafe fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let mut file: String = "wasm32r".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "b";
    }
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
            debugout += &format!(
                "143 pixels: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
                u8x16_extract_lane::<0>(pixels),
                u8x16_extract_lane::<1>(pixels),
                u8x16_extract_lane::<2>(pixels),
                u8x16_extract_lane::<3>(pixels),
                u8x16_extract_lane::<4>(pixels),
                u8x16_extract_lane::<5>(pixels),
                u8x16_extract_lane::<6>(pixels),
                u8x16_extract_lane::<7>(pixels),
                u8x16_extract_lane::<8>(pixels),
                u8x16_extract_lane::<9>(pixels),
                u8x16_extract_lane::<10>(pixels),
                u8x16_extract_lane::<11>(pixels),
                u8x16_extract_lane::<12>(pixels),
                u8x16_extract_lane::<13>(pixels),
                u8x16_extract_lane::<14>(pixels),
                u8x16_extract_lane::<15>(pixels),
            );
            v128_store(dst_ptr, pixels);
        },
    );

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
        debugout += &format!(
            "165 dst_pixels: {:?} {:?} {:?} {:?}\n",
            u32x4_extract_lane::<0>(dst_pixels),
            u32x4_extract_lane::<1>(dst_pixels),
            u32x4_extract_lane::<2>(dst_pixels),
            u32x4_extract_lane::<3>(dst_pixels),
        );
        v128_store(dst_buffer.as_mut_ptr() as *mut v128, dst_pixels);

        dst_buffer
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
    fs::write(file, debugout).unwrap();
}

#[inline]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U8x4]) {
    let mut file: String = "wasm32i".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "c";
    }
    let mut chunks = row.chunks_exact_mut(4);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = v128_load(chunk.as_ptr() as *const v128);
            let dst_ptr = chunk.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_4_pixels(pixels);
            debugout += &format!(
                "179 pixels: {:?} {:?} {:?} {:?}\n",
                u32x4_extract_lane::<0>(pixels),
                u32x4_extract_lane::<1>(pixels),
                u32x4_extract_lane::<2>(pixels),
                u32x4_extract_lane::<3>(pixels),
            );
            v128_store(dst_ptr, pixels);
        },
    );

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
        debugout += &format!(
            "201 dst_pixels: {:?} {:?} {:?} {:?}\n",
            u32x4_extract_lane::<0>(dst_pixels),
            u32x4_extract_lane::<1>(dst_pixels),
            u32x4_extract_lane::<2>(dst_pixels),
            u32x4_extract_lane::<3>(dst_pixels),
        );
        v128_store(dst_buffer.as_mut_ptr() as *mut v128, dst_pixels);

        dst_buffer.iter().zip(tail).for_each(|(s, d)| *d = *s);
    }
    fs::write(file, debugout).unwrap();
}

#[inline]
unsafe fn divide_alpha_4_pixels(src_pixels: v128) -> v128 {
    let mut file: String = "wasm328".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "a";
    }
    debugout += &format!(
        "src_pixels: {:?} {:?} {:?} {:?}\n",
        u32x4_extract_lane::<0>(src_pixels),
        u32x4_extract_lane::<1>(src_pixels),
        u32x4_extract_lane::<2>(src_pixels),
        u32x4_extract_lane::<3>(src_pixels),
    );
    let zero = i64x2_splat(0);
    let alpha_mask = i32x4_splat(0xff000000u32 as i32);
    let shuffle1 = i8x16(0, 1, 0, 1, 0, 1, 0, 1, 4, 5, 4, 5, 4, 5, 4, 5);
    let shuffle2 = i8x16(8, 9, 8, 9, 8, 9, 8, 9, 12, 13, 12, 13, 12, 13, 12, 13);
    let alpha_scale = f32x4_splat(255.0 * 256.0);
    let alpha_max = f32x4_splat(2147483648f32);

    let alpha_f32 = f32x4_convert_i32x4(u32x4_shr(src_pixels, 24));
    debugout += &format!(
        "shift24: {:?} {:?} {:?} {:?}\n",
        i32x4_extract_lane::<0>(u32x4_shr(src_pixels, 24)),
        i32x4_extract_lane::<1>(u32x4_shr(src_pixels, 24)),
        i32x4_extract_lane::<2>(u32x4_shr(src_pixels, 24)),
        i32x4_extract_lane::<3>(u32x4_shr(src_pixels, 24)),
    );
    debugout += &format!(
        "alpha_f32: {:?} {:?} {:?} {:?}\n",
        f32x4_extract_lane::<0>(alpha_f32),
        f32x4_extract_lane::<1>(alpha_f32),
        f32x4_extract_lane::<2>(alpha_f32),
        f32x4_extract_lane::<3>(alpha_f32),
    );
    let scaled_alpha_f32 = f32x4_div(alpha_scale, alpha_f32);
    debugout += &format!(
        "scaled_alpha_f32: {:?} {:?} {:?} {:?}\n",
        f32x4_extract_lane::<0>(scaled_alpha_f32),
        f32x4_extract_lane::<1>(scaled_alpha_f32),
        f32x4_extract_lane::<2>(scaled_alpha_f32),
        f32x4_extract_lane::<3>(scaled_alpha_f32),
    );
    let scaled_alpha_u32 = u32x4_trunc_sat_f32x4(f32x4_min(scaled_alpha_f32, alpha_max));
    debugout += &format!(
        "scaled_alpha_u32: {:?} {:?} {:?} {:?}\n",
        u32x4_extract_lane::<0>(scaled_alpha_u32),
        u32x4_extract_lane::<1>(scaled_alpha_u32),
        u32x4_extract_lane::<2>(scaled_alpha_u32),
        u32x4_extract_lane::<3>(scaled_alpha_u32),
    );
    debugout += &format!(
        "scaled_alpha_u32: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u8x16_extract_lane::<0>(scaled_alpha_u32),
        u8x16_extract_lane::<1>(scaled_alpha_u32),
        u8x16_extract_lane::<2>(scaled_alpha_u32),
        u8x16_extract_lane::<3>(scaled_alpha_u32),
        u8x16_extract_lane::<4>(scaled_alpha_u32),
        u8x16_extract_lane::<5>(scaled_alpha_u32),
        u8x16_extract_lane::<6>(scaled_alpha_u32),
        u8x16_extract_lane::<7>(scaled_alpha_u32),
        u8x16_extract_lane::<8>(scaled_alpha_u32),
        u8x16_extract_lane::<9>(scaled_alpha_u32),
        u8x16_extract_lane::<10>(scaled_alpha_u32),
        u8x16_extract_lane::<11>(scaled_alpha_u32),
        u8x16_extract_lane::<12>(scaled_alpha_u32),
        u8x16_extract_lane::<13>(scaled_alpha_u32),
        u8x16_extract_lane::<14>(scaled_alpha_u32),
        u8x16_extract_lane::<15>(scaled_alpha_u32),
    );
    let mma0 = u8x16_swizzle(scaled_alpha_u32, shuffle1);
    let mma1 = u8x16_swizzle(scaled_alpha_u32, shuffle2);

    let pix0 =
        u8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(zero, src_pixels);
    let pix1 = u8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(
        zero, src_pixels,
    );

    let pix0 = wasm32_utils::u16x8_mul_hi(pix0, mma0);
    debugout += &format!(
        "pix0: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(pix0),
        u16x8_extract_lane::<1>(pix0),
        u16x8_extract_lane::<2>(pix0),
        u16x8_extract_lane::<3>(pix0),
        u16x8_extract_lane::<4>(pix0),
        u16x8_extract_lane::<5>(pix0),
        u16x8_extract_lane::<6>(pix0),
        u16x8_extract_lane::<7>(pix0),
    );
    let pix1 = wasm32_utils::u16x8_mul_hi(pix1, mma1);
    debugout += &format!(
        "pix1: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(pix1),
        u16x8_extract_lane::<1>(pix1),
        u16x8_extract_lane::<2>(pix1),
        u16x8_extract_lane::<3>(pix1),
        u16x8_extract_lane::<4>(pix1),
        u16x8_extract_lane::<5>(pix1),
        u16x8_extract_lane::<6>(pix1),
        u16x8_extract_lane::<7>(pix1),
    );

    let alpha = v128_and(src_pixels, alpha_mask);
    debugout += &format!(
        "alpha: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u8x16_extract_lane::<0>(alpha),
        u8x16_extract_lane::<1>(alpha),
        u8x16_extract_lane::<2>(alpha),
        u8x16_extract_lane::<3>(alpha),
        u8x16_extract_lane::<4>(alpha),
        u8x16_extract_lane::<5>(alpha),
        u8x16_extract_lane::<6>(alpha),
        u8x16_extract_lane::<7>(alpha),
        u8x16_extract_lane::<8>(alpha),
        u8x16_extract_lane::<9>(alpha),
        u8x16_extract_lane::<10>(alpha),
        u8x16_extract_lane::<11>(alpha),
        u8x16_extract_lane::<12>(alpha),
        u8x16_extract_lane::<13>(alpha),
        u8x16_extract_lane::<14>(alpha),
        u8x16_extract_lane::<15>(alpha),
    );
    let rgb = u8x16_narrow_i16x8(pix0, pix1);
    debugout += &format!(
        "rgb: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u8x16_extract_lane::<0>(rgb),
        u8x16_extract_lane::<1>(rgb),
        u8x16_extract_lane::<2>(rgb),
        u8x16_extract_lane::<3>(rgb),
        u8x16_extract_lane::<4>(rgb),
        u8x16_extract_lane::<5>(rgb),
        u8x16_extract_lane::<6>(rgb),
        u8x16_extract_lane::<7>(rgb),
        u8x16_extract_lane::<8>(rgb),
        u8x16_extract_lane::<9>(rgb),
        u8x16_extract_lane::<10>(rgb),
        u8x16_extract_lane::<11>(rgb),
        u8x16_extract_lane::<12>(rgb),
        u8x16_extract_lane::<13>(rgb),
        u8x16_extract_lane::<14>(rgb),
        u8x16_extract_lane::<15>(rgb),
    );
    fs::write(file, debugout).unwrap();

    u8x16_shuffle::<0, 1, 2, 19, 4, 5, 6, 23, 8, 9, 10, 27, 12, 13, 14, 31>(rgb, alpha)
}
