use std::arch::wasm32::*;

use crate::pixels::U8x2;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};
use std::fs;
use std::path::Path;

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
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = v128_load(src.as_ptr() as *const v128);
            let dst_ptr = dst.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiplies_alpha_8_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    // Using a simple for-loop in this case is faster than implementation with pre-reading
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
unsafe fn multiplies_alpha_8_pixels(pixels: v128) -> v128 {
    let zero = i64x2_splat(0);
    let half = i16x8_splat(128);
    const MAX_A: i16 = 0xff00u16 as i16;
    let max_alpha = i16x8_splat(MAX_A);
    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|
    */
    let factor_mask = i8x16(1, 1, 3, 3, 5, 5, 7, 7, 9, 9, 11, 11, 13, 13, 15, 15);

    let factor_pixels = i8x16_swizzle(pixels, factor_mask);
    let factor_pixels = v128_or(factor_pixels, max_alpha);

    let src_i16_lo =
        i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(pixels, zero);
    let factors = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
        factor_pixels,
        zero,
    );
    let src_i16_lo = i16x8_add(i16x8_mul(src_i16_lo, factors), half);
    let dst_i16_lo = i16x8_add(src_i16_lo, u16x8_shr(src_i16_lo, 8));
    let dst_i16_lo = u16x8_shr(dst_i16_lo, 8);

    let src_i16_hi =
        i8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(pixels, zero);
    let factors = i8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(
        factor_pixels,
        zero,
    );
    let src_i16_hi = i16x8_add(i16x8_mul(src_i16_hi, factors), half);
    let dst_i16_hi = i16x8_add(src_i16_hi, u16x8_shr(src_i16_hi, 8));
    let dst_i16_hi = u16x8_shr(dst_i16_hi, 8);

    u8x16_narrow_i16x8(dst_i16_lo, dst_i16_hi)
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
pub(crate) unsafe fn divide_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    let file = "wasm32";
    let mut debugout = String::new();
    let file_exists = Path::new(file).exists();
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = v128_load(src.as_ptr() as *const v128);
            let dst_ptr = dst.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            if !file_exists {
                debugout += &format!(
                    "142: {:?} {:?}\n",
                    i64x2_extract_lane::<0>(pixels),
                    i64x2_extract_lane::<1>(pixels)
                );
            }
            v128_store(dst_ptr, pixels);
        },
    );

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
        if !file_exists {
            debugout += &format!(
                "164: {:?} {:?}\n",
                i64x2_extract_lane::<0>(pixels),
                i64x2_extract_lane::<1>(pixels)
            );
        }
        v128_store(dst_pixels.as_mut_ptr() as *mut v128, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
    if !file_exists {
        fs::write(file, debugout).unwrap();
    }
}

#[inline]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = v128_load(chunk.as_ptr() as *const v128);
            let dst_ptr = chunk.as_mut_ptr() as *mut v128;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            v128_store(dst_ptr, pixels);
        },
    );

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
unsafe fn divide_alpha_8_pixels(pixels: v128) -> v128 {
    let mut file: String = "wasm328".to_string();
    let mut debugout = String::new();
    //let file_exists = Path::new(file).exists();
    while Path::new(&file).exists() {
        file += "a";
    }
    debugout += &format!(
        "pixels: {:?} {:?}\n",
        i64x2_extract_lane::<0>(pixels),
        i64x2_extract_lane::<1>(pixels)
    );
    let alpha_mask = i16x8_splat(0xff00u16 as i16);
    let luma_mask = i16x8_splat(0xff);
    //let scaled_max = f32x4_splat(1073741824f32);
    let alpha32_sh_lo = i8x16(1, -1, -1, -1, 3, -1, -1, -1, 5, -1, -1, -1, 7, -1, -1, -1);
    let alpha32_sh_hi = i8x16(
        9, -1, -1, -1, 11, -1, -1, -1, 13, -1, -1, -1, 15, -1, -1, -1,
    );
    let alpha_scale = f32x4_splat(255.0 * 256.0);

    let alpha_lo_f32 = f32x4_convert_i32x4(i8x16_swizzle(pixels, alpha32_sh_lo));
    debugout += &format!(
        "alpha_lo_f32: {:?} {:?} {:?} {:?}\n",
        f32x4_extract_lane::<0>(alpha_lo_f32),
        f32x4_extract_lane::<1>(alpha_lo_f32),
        f32x4_extract_lane::<2>(alpha_lo_f32),
        f32x4_extract_lane::<3>(alpha_lo_f32)
    );
    let scaled_alpha_lo_u32 =
        u32x4_trunc_sat_f32x4(f32x4_nearest(f32x4_div(alpha_scale, alpha_lo_f32)));
    debugout += &format!(
        "scaled_alpha_lo_u32: {:?} {:?} {:?} {:?}\n",
        u32x4_extract_lane::<0>(scaled_alpha_lo_u32),
        u32x4_extract_lane::<1>(scaled_alpha_lo_u32),
        u32x4_extract_lane::<2>(scaled_alpha_lo_u32),
        u32x4_extract_lane::<3>(scaled_alpha_lo_u32)
    );
    debugout += &format!(
        "as_u16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<1>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<2>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<3>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<4>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<5>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<6>(scaled_alpha_lo_u32),
        u16x8_extract_lane::<7>(scaled_alpha_lo_u32),
    );
    let alpha_hi_f32 = f32x4_convert_u32x4(i8x16_swizzle(pixels, alpha32_sh_hi));
    debugout += &format!(
        "alpha_hi_f32: {:?} {:?} {:?} {:?}\n",
        f32x4_extract_lane::<0>(alpha_hi_f32),
        f32x4_extract_lane::<1>(alpha_hi_f32),
        f32x4_extract_lane::<2>(alpha_hi_f32),
        f32x4_extract_lane::<3>(alpha_hi_f32)
    );
    let scaled_alpha_hi_u32 =
        u32x4_trunc_sat_f32x4(f32x4_nearest(f32x4_div(alpha_scale, alpha_hi_f32)));
    debugout += &format!(
        "scaled_alpha_hi_f32: {:?} {:?} {:?} {:?}\n",
        f32x4_extract_lane::<0>(f32x4_nearest(f32x4_div(alpha_scale, alpha_hi_f32))),
        f32x4_extract_lane::<1>(f32x4_nearest(f32x4_div(alpha_scale, alpha_hi_f32))),
        f32x4_extract_lane::<2>(f32x4_nearest(f32x4_div(alpha_scale, alpha_hi_f32))),
        f32x4_extract_lane::<3>(f32x4_nearest(f32x4_div(alpha_scale, alpha_hi_f32)))
    );
    debugout += &format!(
        "scaled_alpha_hi_u32: {:?} {:?} {:?} {:?}\n",
        u32x4_extract_lane::<0>(scaled_alpha_hi_u32),
        u32x4_extract_lane::<1>(scaled_alpha_hi_u32),
        u32x4_extract_lane::<2>(scaled_alpha_hi_u32),
        u32x4_extract_lane::<3>(scaled_alpha_hi_u32),
    );
    debugout += &format!(
        "as_u16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<1>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<2>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<3>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<4>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<5>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<6>(scaled_alpha_hi_u32),
        u16x8_extract_lane::<7>(scaled_alpha_hi_u32)
    );
    let scaled_alpha_u16 = u16x8_narrow_i32x4(scaled_alpha_lo_u32, scaled_alpha_hi_u32);
    debugout += &format!(
        "scaled_alpha_u16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(scaled_alpha_u16),
        u16x8_extract_lane::<1>(scaled_alpha_u16),
        u16x8_extract_lane::<2>(scaled_alpha_u16),
        u16x8_extract_lane::<3>(scaled_alpha_u16),
        u16x8_extract_lane::<4>(scaled_alpha_u16),
        u16x8_extract_lane::<5>(scaled_alpha_u16),
        u16x8_extract_lane::<6>(scaled_alpha_u16),
        u16x8_extract_lane::<7>(scaled_alpha_u16),
    );

    let luma_u16 = v128_and(pixels, luma_mask);
    debugout += &format!(
        "luma_u16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(luma_u16),
        u16x8_extract_lane::<1>(luma_u16),
        u16x8_extract_lane::<2>(luma_u16),
        u16x8_extract_lane::<3>(luma_u16),
        u16x8_extract_lane::<4>(luma_u16),
        u16x8_extract_lane::<5>(luma_u16),
        u16x8_extract_lane::<6>(luma_u16),
        u16x8_extract_lane::<7>(luma_u16),
    );
    let scaled_luma_u16 = u16x8_mul(luma_u16, scaled_alpha_u16);
    let scaled_luma_u16 = u16x8_shr(scaled_luma_u16, 8);
    debugout += &format!(
        "scaled_luma_u16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        u16x8_extract_lane::<0>(scaled_luma_u16),
        u16x8_extract_lane::<1>(scaled_luma_u16),
        u16x8_extract_lane::<2>(scaled_luma_u16),
        u16x8_extract_lane::<3>(scaled_luma_u16),
        u16x8_extract_lane::<4>(scaled_luma_u16),
        u16x8_extract_lane::<5>(scaled_luma_u16),
        u16x8_extract_lane::<6>(scaled_luma_u16),
        u16x8_extract_lane::<7>(scaled_luma_u16),
    );

    let alpha = v128_and(pixels, alpha_mask);
    fs::write(file, debugout).unwrap();
    u8x16_shuffle::<0, 17, 2, 19, 4, 21, 6, 23, 8, 25, 10, 27, 12, 29, 14, 31>(
        scaled_luma_u16,
        alpha,
    )
}
