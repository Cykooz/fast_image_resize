use std::arch::x86_64::*;

use crate::simd_utils;
use crate::{DstImageView, SrcImageView};

pub(crate) fn multiply_alpha_avx2(src_image: &SrcImageView, dst_image: &mut DstImageView) {
    let width = src_image.width().get() as usize;
    let src_rows = src_image.iter_rows(0, src_image.height().get());
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            multiply_alpha_row_avx2(src_row, dst_row, width);
        }
    }
}

pub(crate) fn multiply_alpha_inplace_avx2(image: &mut DstImageView) {
    let width = image.width().get() as usize;
    for dst_row in image.iter_rows_mut() {
        unsafe {
            let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
            multiply_alpha_row_avx2(src_row, dst_row, width);
        }
    }
}

#[allow(dead_code)]
pub(crate) fn multiply_alpha_sse2(src_image: &SrcImageView, dst_image: &mut DstImageView) {
    let width = src_image.width().get() as usize;
    let src_rows = src_image.iter_rows(0, src_image.height().get());
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            multiply_alpha_row_sse2(src_row, dst_row, width);
        }
    }
}

pub(crate) fn multiply_alpha_native(src_image: &SrcImageView, dst_image: &mut DstImageView) {
    let src_rows = src_image.iter_rows(0, src_image.height().get());
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row_native(src_row, dst_row);
    }
}

pub(crate) fn multiply_alpha_inplace_native(image: &mut DstImageView) {
    for dst_row in image.iter_rows_mut() {
        let src_row = unsafe { std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len()) };
        multiply_alpha_row_native(src_row, dst_row);
    }
}

/// https://github.com/Wizermil/premultiply_alpha/blob/master/premultiply_alpha/premultiply_alpha.hpp#L232
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_row_avx2(src_row: &[u32], dst_row: &mut [u32], width: usize) {
    let mask_alpha_color_odd_255 = _mm256_set1_epi32(0xff000000u32 as i32);
    let div_255 = _mm256_set1_epi16(0x8081u16 as i16);
    #[rustfmt::skip]
    let mask_shuffle_alpha = _mm256_set_epi8(
        15, -1, 15, -1, 11, -1, 11, -1, 7, -1, 7, -1, 3, -1, 3, -1,
        15, -1, 15, -1, 11, -1, 11, -1, 7, -1, 7, -1, 3, -1, 3, -1,
    );
    #[rustfmt::skip]
    let mask_shuffle_color_odd = _mm256_set_epi8(
        -1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1,
        -1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1,
    );

    let mut x: usize = 0;
    while x < width.saturating_sub(7) {
        let mut color = simd_utils::loadu_si256(src_row, x);

        let alpha = _mm256_shuffle_epi8(color, mask_shuffle_alpha);
        let mut color_even = _mm256_slli_epi16(color, 8);
        let mut color_odd = _mm256_shuffle_epi8(color, mask_shuffle_color_odd);
        color_odd = _mm256_or_si256(color_odd, mask_alpha_color_odd_255);

        color_odd = _mm256_mulhi_epu16(color_odd, alpha);
        color_even = _mm256_mulhi_epu16(color_even, alpha);

        color_odd = _mm256_srli_epi16(_mm256_mulhi_epu16(color_odd, div_255), 7);
        color_even = _mm256_srli_epi16(_mm256_mulhi_epu16(color_even, div_255), 7);

        color = _mm256_or_si256(color_even, _mm256_slli_epi16(color_odd, 8));

        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, color);

        x += 8;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    multiply_alpha_row_native(src_tail, dst_tail);
}

/// https://github.com/Wizermil/premultiply_alpha/blob/master/premultiply_alpha/premultiply_alpha.hpp#L108
/// This implementation is twice slowly than native version.
#[allow(dead_code)]
#[target_feature(enable = "sse2")]
unsafe fn multiply_alpha_row_sse2(src_row: &[u32], dst_row: &mut [u32], width: usize) {
    let mask_alpha_color_odd_255 = _mm_set1_epi32(0xff000000u32 as i32);
    let div_255 = _mm_set1_epi16(0x8081u16 as i16);

    let mask_shuffle_alpha =
        _mm_set_epi8(15, -1, 15, -1, 11, -1, 11, -1, 7, -1, 7, -1, 3, -1, 3, -1);
    let mask_shuffle_color_odd =
        _mm_set_epi8(-1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1);

    let mut x: usize = 0;
    while x < width.saturating_sub(3) {
        let mut color = simd_utils::loadu_si128(src_row, x);
        let alpha = _mm_shuffle_epi8(color, mask_shuffle_alpha);

        let mut color_even = _mm_slli_epi16(color, 8);
        let mut color_odd = _mm_shuffle_epi8(color, mask_shuffle_color_odd);
        color_odd = _mm_or_si128(color_odd, mask_alpha_color_odd_255);

        color_odd = _mm_mulhi_epu16(color_odd, alpha);
        color_even = _mm_mulhi_epu16(color_even, alpha);

        color_odd = _mm_srli_epi16(_mm_mulhi_epu16(color_odd, div_255), 7);
        color_even = _mm_srli_epi16(_mm_mulhi_epu16(color_even, div_255), 7);

        color = _mm_or_si128(color_even, _mm_slli_epi16(color_odd, 8));

        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m128i;
        _mm_storeu_si128(dst_ptr, color);

        x += 4;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    multiply_alpha_row_native(src_tail, dst_tail);
}

#[inline(always)]
fn multiply_alpha_row_native(src_row: &[u32], dst_row: &mut [u32]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components: [u8; 4] = src_pixel.to_le_bytes();
        let alpha = components[3];
        let res: [u8; 4] = [
            mul_div_255(components[0], alpha),
            mul_div_255(components[1], alpha),
            mul_div_255(components[2], alpha),
            alpha,
        ];
        *dst_pixel = u32::from_le_bytes(res);
    }
}

#[inline(always)]
fn mul_div_255(a: u8, b: u8) -> u8 {
    let tmp = a as u32 * b as u32 + 128;
    (((tmp >> 8) + tmp) >> 8) as u8
}
