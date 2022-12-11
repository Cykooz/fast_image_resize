use std::arch::aarch64::*;

use crate::neon_utils;
use crate::pixels::U16x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "neon")]
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

#[target_feature(enable = "neon")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
unsafe fn multiply_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = neon_utils::load_deintrel_u16x8x4(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut u16;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels.0 = neon_utils::multiply_color_to_alpha_u16x8(pixels.0, pixels.3);
            pixels.1 = neon_utils::multiply_color_to_alpha_u16x8(pixels.1, pixels.3);
            pixels.2 = neon_utils::multiply_color_to_alpha_u16x8(pixels.2, pixels.3);
            vst4q_u16(dst_ptr, pixels);
        },
    );

    let src_chunks = src_remainder.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let dst_reminder = dst_chunks.into_remainder();
    let mut dst_chunks = dst_reminder.chunks_exact_mut(4);
    let mut src_dst = src_chunks.zip(&mut dst_chunks);
    if let Some((src, dst)) = src_dst.next() {
        let mut pixels = neon_utils::load_deintrel_u16x4x4(src, 0);
        pixels.0 = neon_utils::multiply_color_to_alpha_u16x4(pixels.0, pixels.3);
        pixels.1 = neon_utils::multiply_color_to_alpha_u16x4(pixels.1, pixels.3);
        pixels.2 = neon_utils::multiply_color_to_alpha_u16x4(pixels.2, pixels.3);
        let dst_ptr = dst.as_mut_ptr() as *mut u16;
        vst4_u16(dst_ptr, pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline(always)]
unsafe fn multiply_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = neon_utils::load_deintrel_u16x8x4(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut u16;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels.0 = neon_utils::multiply_color_to_alpha_u16x8(pixels.0, pixels.3);
            pixels.1 = neon_utils::multiply_color_to_alpha_u16x8(pixels.1, pixels.3);
            pixels.2 = neon_utils::multiply_color_to_alpha_u16x8(pixels.2, pixels.3);
            vst4q_u16(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    let mut chunks = reminder.chunks_exact_mut(4);
    if let Some(chunk) = chunks.next() {
        let mut pixels = neon_utils::load_deintrel_u16x4x4(chunk, 0);
        pixels.0 = neon_utils::multiply_color_to_alpha_u16x4(pixels.0, pixels.3);
        pixels.1 = neon_utils::multiply_color_to_alpha_u16x4(pixels.1, pixels.3);
        pixels.2 = neon_utils::multiply_color_to_alpha_u16x4(pixels.2, pixels.3);
        let dst_ptr = chunk.as_mut_ptr() as *mut u16;
        vst4_u16(dst_ptr, pixels);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

// Divide

#[target_feature(enable = "neon")]
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

#[target_feature(enable = "neon")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = neon_utils::load_deintrel_u16x8x4(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut u16;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            vst4q_u16(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U16x4::new([0; 4]); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x4::new([0; 4]); 8];
        let mut pixels = neon_utils::load_deintrel_u16x8x4(&src_pixels, 0);
        pixels = divide_alpha_8_pixels(pixels);
        let dst_ptr = dst_pixels.as_mut_ptr() as *mut u16;
        vst4q_u16(dst_ptr, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline(always)]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = neon_utils::load_deintrel_u16x8x4(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut u16;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            vst4q_u16(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        let mut src_pixels = [U16x4::new([0; 4]); 8];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x4::new([0; 4]); 8];
        let mut pixels = neon_utils::load_deintrel_u16x8x4(&src_pixels, 0);
        pixels = divide_alpha_8_pixels(pixels);
        let dst_ptr = dst_pixels.as_mut_ptr() as *mut u16;
        vst4q_u16(dst_ptr, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline(always)]
unsafe fn divide_alpha_8_pixels(mut pixels: uint16x8x4_t) -> uint16x8x4_t {
    let zero = vdupq_n_u16(0);
    let alpha_scale = vdupq_n_f32(65535.0);
    let nonzero_alpha_mask = vmvnq_u16(vceqzq_u16(pixels.3));

    // Low
    let alpha_scaled_u32 = vreinterpretq_u32_u16(vzip1q_u16(pixels.3, zero));
    let alpha_scaled_f32 = vcvtq_f32_u32(alpha_scaled_u32);
    let recip_alpha_lo_f32 = vdivq_f32(alpha_scale, alpha_scaled_f32);

    // High
    let alpha_scaled_u32 = vreinterpretq_u32_u16(vzip2q_u16(pixels.3, zero));
    let alpha_scaled_f32 = vcvtq_f32_u32(alpha_scaled_u32);
    let recip_alpha_hi_f32 = vdivq_f32(alpha_scale, alpha_scaled_f32);

    pixels.0 = mul_color_recip_alpha(pixels.0, recip_alpha_lo_f32, recip_alpha_hi_f32, zero);
    pixels.0 = vandq_u16(pixels.0, nonzero_alpha_mask);
    pixels.1 = mul_color_recip_alpha(pixels.1, recip_alpha_lo_f32, recip_alpha_hi_f32, zero);
    pixels.1 = vandq_u16(pixels.1, nonzero_alpha_mask);
    pixels.2 = mul_color_recip_alpha(pixels.2, recip_alpha_lo_f32, recip_alpha_hi_f32, zero);
    pixels.2 = vandq_u16(pixels.2, nonzero_alpha_mask);
    pixels
}

#[inline(always)]
unsafe fn mul_color_recip_alpha(
    color: uint16x8_t,
    recip_alpha_lo: float32x4_t,
    recip_alpha_hi: float32x4_t,
    zero: uint16x8_t,
) -> uint16x8_t {
    let color_lo_f32 = vcvtq_f32_u32(vreinterpretq_u32_u16(vzip1q_u16(color, zero)));
    let res_lo_u32 = vcvtaq_u32_f32(vmulq_f32(color_lo_f32, recip_alpha_lo));

    let color_hi_f32 = vcvtq_f32_u32(vreinterpretq_u32_u16(vzip2q_u16(color, zero)));
    let res_hi_u32 = vcvtaq_u32_f32(vmulq_f32(color_hi_f32, recip_alpha_hi));

    vcombine_u16(vmovn_u32(res_lo_u32), vmovn_u32(res_hi_u32))
}
