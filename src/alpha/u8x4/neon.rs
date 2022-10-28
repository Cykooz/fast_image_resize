use crate::neon_utils;
use crate::pixels::U8x4;
use crate::{ImageView, ImageViewMut};
use std::arch::aarch64::*;
use std::intrinsics::transmute;

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
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        multiply_alpha_row(src_row, dst_row);
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let zero = vdupq_n_u8(0);

    const MAX_A: u32 = 0xff000000u32;
    let max_alpha = vreinterpretq_u8_u32(vdupq_n_u32(MAX_A));
    static FACTOR_MASK_DATA: [u8; 16] = [3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15];
    let factor_mask = vld1q_u8(FACTOR_MASK_DATA.as_ptr());

    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = neon_utils::load_u8x16(src, 0);

        let factor_pixels = vqtbl1q_u8(src_pixels, factor_mask);
        let factor_pixels = vorrq_u8(factor_pixels, max_alpha);

        let pix1 = vreinterpretq_u16_u8(vzip1q_u8(src_pixels, zero));
        let factors = vreinterpretq_u16_u8(vzip1q_u8(factor_pixels, zero));
        let pix1 = vmulq_u16(pix1, factors);
        let pix1 = vaddq_u16(pix1, vrshrq_n_u16::<8>(pix1));
        let pix1 = vrshrq_n_u16::<8>(pix1);

        let pix2 = vreinterpretq_u16_u8(vzip2q_u8(src_pixels, zero));
        let factors = vreinterpretq_u16_u8(vzip2q_u8(factor_pixels, zero));
        let pix2 = vmulq_u16(pix2, factors);
        let pix2 = vaddq_u16(pix2, vrshrq_n_u16::<8>(pix2));
        let pix2 = vrshrq_n_u16::<8>(pix2);

        let dst_pixels = vcombine_u8(vqmovn_u16(pix1), vqmovn_u16(pix2));

        let dst_ptr = dst.as_mut_ptr() as *mut u128;
        vstrq_p128(dst_ptr, transmute(dst_pixels));
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
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
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "neon")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        divide_alpha_four_pixels(src, dst);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x4::new(0); 4];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x4::new(0); 4];
        divide_alpha_four_pixels(src_pixels.as_slice(), dst_pixels.as_mut_slice());

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[target_feature(enable = "neon")]
#[inline]
unsafe fn divide_alpha_four_pixels(src: &[U8x4], dst: &mut [U8x4]) {
    let zero = vdupq_n_u8(0);
    let alpha_mask = vreinterpretq_u8_u32(vdupq_n_u32(0xff000000u32));

    static SHUFFLE1_DATA: [u8; 16] = [0, 1, 0, 1, 0, 1, 0, 1, 4, 5, 4, 5, 4, 5, 4, 5];
    let shuffle1 = vld1q_u8(SHUFFLE1_DATA.as_ptr());
    static SHUFFLE2_DATA: [u8; 16] = [8, 9, 8, 9, 8, 9, 8, 9, 12, 13, 12, 13, 12, 13, 12, 13];
    let shuffle2 = vld1q_u8(SHUFFLE2_DATA.as_ptr());
    let alpha_scale = vdupq_n_f32(255.0 * 256.0);

    let src_pixels = neon_utils::load_u8x16(src, 0);

    let alpha_u32 = vshrq_n_u32::<24>(vreinterpretq_u32_u8(src_pixels));
    let zero_alpha_mask = vceqzq_u32(alpha_u32);
    let alpha_f32 = vcvtq_f32_u32(vorrq_u32(alpha_u32, zero_alpha_mask));
    let scaled_alpha_f32 = vdivq_f32(alpha_scale, alpha_f32);
    let scaled_alpha_u32 = vcvtnq_u32_f32(scaled_alpha_f32);
    let mma0 = vreinterpretq_u16_u8(vqtbl1q_u8(vreinterpretq_u8_u32(scaled_alpha_u32), shuffle1));
    let mma1 = vreinterpretq_u16_u8(vqtbl1q_u8(vreinterpretq_u8_u32(scaled_alpha_u32), shuffle2));

    let pix0 = vreinterpretq_u16_u8(vzip1q_u8(zero, src_pixels));
    let pix1 = vreinterpretq_u16_u8(vzip2q_u8(zero, src_pixels));

    let pix0 = neon_utils::mulhi_u16x8(pix0, mma0);
    let pix1 = neon_utils::mulhi_u16x8(pix1, mma1);

    let alpha = vandq_u8(src_pixels, alpha_mask);
    let rgb = vcombine_u8(vmovn_u16(pix0), vmovn_u16(pix1));
    let dst_pixels = vbslq_u8(alpha_mask, alpha, rgb);

    let dst_ptr = dst.as_mut_ptr() as *mut u128;
    vstrq_p128(dst_ptr, transmute(dst_pixels));
}
