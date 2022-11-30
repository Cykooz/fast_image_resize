use std::arch::aarch64::*;

use crate::neon_utils;
use crate::pixels::U8x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "neon")]
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

#[target_feature(enable = "neon")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
unsafe fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(16);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(16);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = neon_utils::load_deintrel_u8x16x4(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut u8;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiplies_alpha_16_pixles(pixels);
            vst4q_u8(dst_ptr, pixels);
        },
    );

    let src_chunks = src_remainder.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let dst_reminder = dst_chunks.into_remainder();
    let mut dst_chunks = dst_reminder.chunks_exact_mut(8);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let mut pixels = neon_utils::load_deintrel_u8x8x4(src, 0);
        pixels = multiplies_alpha_8_pixles(pixels);
        let dst_ptr = dst.as_mut_ptr() as *mut u8;
        vst4_u8(dst_ptr, pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline(always)]
unsafe fn multiply_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(16);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = neon_utils::load_deintrel_u8x16x4(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut u8;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiplies_alpha_16_pixles(pixels);
            vst4q_u8(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    let mut chunks = reminder.chunks_exact_mut(8);
    if let Some(chunk) = chunks.next() {
        let mut pixels = neon_utils::load_deintrel_u8x8x4(chunk, 0);
        pixels = multiplies_alpha_8_pixles(pixels);
        let dst_ptr = chunk.as_mut_ptr() as *mut u8;
        vst4_u8(dst_ptr, pixels);
    }

    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        native::multiply_alpha_row_inplace(tail);
    }
}

#[inline(always)]
unsafe fn multiplies_alpha_16_pixles(mut pixels: uint8x16x4_t) -> uint8x16x4_t {
    let zero_u8x16 = vdupq_n_u8(0);
    let alpha_u16 = uint16x8x2_t(
        vreinterpretq_u16_u8(vzip1q_u8(pixels.3, zero_u8x16)),
        vreinterpretq_u16_u8(vzip2q_u8(pixels.3, zero_u8x16)),
    );
    pixels.0 = neon_utils::mul_color_to_alpha_u8x16(pixels.0, alpha_u16, zero_u8x16);
    pixels.1 = neon_utils::mul_color_to_alpha_u8x16(pixels.1, alpha_u16, zero_u8x16);
    pixels.2 = neon_utils::mul_color_to_alpha_u8x16(pixels.2, alpha_u16, zero_u8x16);
    pixels
}

#[inline(always)]
unsafe fn multiplies_alpha_8_pixles(mut pixels: uint8x8x4_t) -> uint8x8x4_t {
    let zero_u8x8 = vdup_n_u8(0);
    let alpha_u8 = pixels.3;
    let alpha_u16_lo = vreinterpret_u16_u8(vzip1_u8(alpha_u8, zero_u8x8));
    let alpha_u16_hi = vreinterpret_u16_u8(vzip2_u8(alpha_u8, zero_u8x8));
    let alpha_u16 = vcombine_u16(alpha_u16_lo, alpha_u16_hi);
    pixels.0 = neon_utils::mul_color_to_alpha_u8x8(pixels.0, alpha_u16, zero_u8x8);
    pixels.1 = neon_utils::mul_color_to_alpha_u8x8(pixels.1, alpha_u16, zero_u8x8);
    pixels.2 = neon_utils::mul_color_to_alpha_u8x8(pixels.2, alpha_u16, zero_u8x8);
    pixels
}

// Divide

#[target_feature(enable = "neon")]
pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x4>, dst_image: &mut ImageViewMut<U8x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "neon")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inline(row);
    }
}

#[inline(always)]
unsafe fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(16);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(16);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = neon_utils::load_deintrel_u8x16x4(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut u8;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_16_pixels(pixels);
            vst4q_u8(dst_ptr, pixels);
        },
    );

    let src_chunks = src_remainder.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let dst_reminder = dst_chunks.into_remainder();
    let mut dst_chunks = dst_reminder.chunks_exact_mut(8);
    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let mut pixels = neon_utils::load_deintrel_u8x8x4(src, 0);
        pixels = divide_alpha_8_pixels(pixels);
        let dst_ptr = dst.as_mut_ptr() as *mut u8;
        vst4_u8(dst_ptr, pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x4::new(0); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x4::new(0); 8];
        let mut pixels = neon_utils::load_deintrel_u8x8x4(src_pixels.as_slice(), 0);
        pixels = divide_alpha_8_pixels(pixels);
        let dst_ptr = dst_pixels.as_mut_ptr() as *mut u8;
        vst4_u8(dst_ptr, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline(always)]
unsafe fn divide_alpha_row_inline(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(16);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = neon_utils::load_deintrel_u8x16x4(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut u8;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_16_pixels(pixels);
            vst4q_u8(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    let mut chunks = reminder.chunks_exact_mut(8);
    if let Some(chunk) = chunks.next() {
        let mut pixels = neon_utils::load_deintrel_u8x8x4(chunk, 0);
        pixels = divide_alpha_8_pixels(pixels);
        let dst_ptr = chunk.as_mut_ptr() as *mut u8;
        vst4_u8(dst_ptr, pixels);
    }

    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        let mut src_pixels = [U8x4::new(0); 8];
        src_pixels
            .iter_mut()
            .zip(tail.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x4::new(0); 8];
        let mut pixels = neon_utils::load_deintrel_u8x8x4(src_pixels.as_slice(), 0);
        pixels = divide_alpha_8_pixels(pixels);
        let dst_ptr = dst_pixels.as_mut_ptr() as *mut u8;
        vst4_u8(dst_ptr, pixels);

        dst_pixels.iter().zip(tail).for_each(|(s, d)| *d = *s);
    }
}

#[inline(always)]
unsafe fn divide_alpha_16_pixels(mut pixels: uint8x16x4_t) -> uint8x16x4_t {
    let zero = vdupq_n_u8(0);
    let alpha_scale = vdupq_n_f32(255.0 * 256.0);
    let nonzero_alpha_mask = vmvnq_u8(vceqzq_u8(pixels.3));

    let alpha_u16_lo = vzip1q_u8(pixels.3, zero);
    let alpha_u16_hi = vzip2q_u8(pixels.3, zero);

    let alpha_f32_0 = vcvtq_f32_u32(vreinterpretq_u32_u8(vzip1q_u8(alpha_u16_lo, zero)));
    let recip_alpha_f32_0 = vdivq_f32(alpha_scale, alpha_f32_0);
    let recip_alpha_u16_0 = vmovn_u32(vcvtaq_u32_f32(recip_alpha_f32_0));

    let alpha_f32_1 = vcvtq_f32_u32(vreinterpretq_u32_u8(vzip2q_u8(alpha_u16_lo, zero)));
    let recip_alpha_f32_1 = vdivq_f32(alpha_scale, alpha_f32_1);
    let recip_alpha_u16_1 = vmovn_u32(vcvtaq_u32_f32(recip_alpha_f32_1));

    let alpha_f32_2 = vcvtq_f32_u32(vreinterpretq_u32_u8(vzip1q_u8(alpha_u16_hi, zero)));
    let recip_alpha_f32_2 = vdivq_f32(alpha_scale, alpha_f32_2);
    let recip_alpha_u16_2 = vmovn_u32(vcvtaq_u32_f32(recip_alpha_f32_2));

    let alpha_f32_3 = vcvtq_f32_u32(vreinterpretq_u32_u8(vzip2q_u8(alpha_u16_hi, zero)));
    let recip_alpha_f32_3 = vdivq_f32(alpha_scale, alpha_f32_3);
    let recip_alpha_u16_3 = vmovn_u32(vcvtaq_u32_f32(recip_alpha_f32_3));

    let recip_alpha = uint16x8x2_t(
        vcombine_u16(recip_alpha_u16_0, recip_alpha_u16_1),
        vcombine_u16(recip_alpha_u16_2, recip_alpha_u16_3),
    );

    pixels.0 = neon_utils::mul_color_recip_alpha_u8x16(pixels.0, recip_alpha, zero);
    pixels.0 = vandq_u8(pixels.0, nonzero_alpha_mask);
    pixels.1 = neon_utils::mul_color_recip_alpha_u8x16(pixels.1, recip_alpha, zero);
    pixels.1 = vandq_u8(pixels.1, nonzero_alpha_mask);
    pixels.2 = neon_utils::mul_color_recip_alpha_u8x16(pixels.2, recip_alpha, zero);
    pixels.2 = vandq_u8(pixels.2, nonzero_alpha_mask);

    pixels
}

#[inline(always)]
unsafe fn divide_alpha_8_pixels(mut pixels: uint8x8x4_t) -> uint8x8x4_t {
    let zero_u8x8 = vdup_n_u8(0);
    let zero_u8x16 = vdupq_n_u8(0);
    let alpha_scale = vdupq_n_f32(255.0 * 256.0);
    let nonzero_alpha_mask = vmvn_u8(vceqz_u8(pixels.3));

    let alpha_u16_lo = vzip1_u8(pixels.3, zero_u8x8);
    let alpha_u16_hi = vzip2_u8(pixels.3, zero_u8x8);
    let alpha_u16 = vcombine_u8(alpha_u16_lo, alpha_u16_hi);

    let alpha_f32_0 = vcvtq_f32_u32(vreinterpretq_u32_u8(vzip1q_u8(alpha_u16, zero_u8x16)));
    let recip_alpha_f32_0 = vdivq_f32(alpha_scale, alpha_f32_0);
    let recip_alpha_u16_0 = vmovn_u32(vcvtaq_u32_f32(recip_alpha_f32_0));

    let alpha_f32_1 = vcvtq_f32_u32(vreinterpretq_u32_u8(vzip2q_u8(alpha_u16, zero_u8x16)));
    let recip_alpha_f32_1 = vdivq_f32(alpha_scale, alpha_f32_1);
    let recip_alpha_u16_1 = vmovn_u32(vcvtaq_u32_f32(recip_alpha_f32_1));

    let recip_alpha = vcombine_u16(recip_alpha_u16_0, recip_alpha_u16_1);

    pixels.0 = neon_utils::mul_color_recip_alpha_u8x8(pixels.0, recip_alpha, zero_u8x8);
    pixels.0 = vand_u8(pixels.0, nonzero_alpha_mask);
    pixels.1 = neon_utils::mul_color_recip_alpha_u8x8(pixels.1, recip_alpha, zero_u8x8);
    pixels.1 = vand_u8(pixels.1, nonzero_alpha_mask);
    pixels.2 = neon_utils::mul_color_recip_alpha_u8x8(pixels.2, recip_alpha, zero_u8x8);
    pixels.2 = vand_u8(pixels.2, nonzero_alpha_mask);

    pixels
}
