use std::arch::x86_64::*;

use crate::pixels::U8x2;
use crate::simd_utils;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::sse4;

#[target_feature(enable = "avx2")]
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

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(16);
    let src_tail = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(16);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = simd_utils::loadu_si256(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_16_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );
    if !src_tail.is_empty() {
        let dst_tail = dst_chunks.into_remainder();
        sse4::multiply_alpha_row(src_tail, dst_tail);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(16);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = simd_utils::loadu_si256(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_16_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );
    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        sse4::multiply_alpha_row_inplace(tail);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_16_pixels(pixels: __m256i) -> __m256i {
    let zero = _mm256_setzero_si256();
    let half = _mm256_set1_epi16(128);
    const MAX_A: i16 = 0xff00u16 as i16;
    let max_alpha = _mm256_set1_epi16(MAX_A);
    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|
    */
    #[rustfmt::skip]
        let factor_mask = _mm256_set_epi8(
        15, 15, 13, 13, 11, 11, 9, 9, 7, 7, 5, 5, 3, 3, 1, 1,
        15, 15, 13, 13, 11, 11, 9, 9, 7, 7, 5, 5, 3, 3, 1, 1
    );

    let factor_pixels = _mm256_shuffle_epi8(pixels, factor_mask);
    let factor_pixels = _mm256_or_si256(factor_pixels, max_alpha);

    let src_i16_lo = _mm256_unpacklo_epi8(pixels, zero);
    let factors = _mm256_unpacklo_epi8(factor_pixels, zero);
    let src_i16_lo = _mm256_add_epi16(_mm256_mullo_epi16(src_i16_lo, factors), half);
    let dst_i16_lo = _mm256_add_epi16(src_i16_lo, _mm256_srli_epi16::<8>(src_i16_lo));
    let dst_i16_lo = _mm256_srli_epi16::<8>(dst_i16_lo);

    let src_i16_hi = _mm256_unpackhi_epi8(pixels, zero);
    let factors = _mm256_unpackhi_epi8(factor_pixels, zero);
    let src_i16_hi = _mm256_add_epi16(_mm256_mullo_epi16(src_i16_hi, factors), half);
    let dst_i16_hi = _mm256_add_epi16(src_i16_hi, _mm256_srli_epi16::<8>(src_i16_hi));
    let dst_i16_hi = _mm256_srli_epi16::<8>(dst_i16_hi);

    _mm256_packus_epi16(dst_i16_lo, dst_i16_hi)
}

// Divide

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x2>, dst_image: &mut ImageViewMut<U8x2>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(16);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(16);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = simd_utils::loadu_si256(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_16_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );
    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(16);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = simd_utils::loadu_si256(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_16_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );
    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        sse4::divide_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_16_pixels(pixels: __m256i) -> __m256i {
    let alpha_mask = _mm256_set1_epi16(0xff00u16 as i16);
    let luma_mask = _mm256_set1_epi16(0xff);
    #[rustfmt::skip]
        let alpha32_sh_lo = _mm256_set_epi8(
        -1, -1, -1, 7, -1, -1, -1, 5, -1, -1, -1, 3, -1, -1, -1, 1,
        -1, -1, -1, 7, -1, -1, -1, 5, -1, -1, -1, 3, -1, -1, -1, 1,
    );
    #[rustfmt::skip]
        let alpha32_sh_hi = _mm256_set_epi8(
        -1, -1, -1, 15, -1, -1, -1, 13, -1, -1, -1, 11, -1, -1, -1, 9,
        -1, -1, -1, 15, -1, -1, -1, 13, -1, -1, -1, 11, -1, -1, -1, 9,
    );
    let alpha_scale = _mm256_set1_ps(255.0 * 256.0);

    let alpha_lo_f32 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(pixels, alpha32_sh_lo));
    let scaled_alpha_lo_i32 = _mm256_cvtps_epi32(_mm256_div_ps(alpha_scale, alpha_lo_f32));
    let alpha_hi_f32 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(pixels, alpha32_sh_hi));
    let scaled_alpha_hi_i32 = _mm256_cvtps_epi32(_mm256_div_ps(alpha_scale, alpha_hi_f32));
    let scaled_alpha_i16 = _mm256_packus_epi32(scaled_alpha_lo_i32, scaled_alpha_hi_i32);

    let luma_i16 = _mm256_and_si256(pixels, luma_mask);
    let scaled_luma_i16 = _mm256_mullo_epi16(luma_i16, scaled_alpha_i16);
    let scaled_luma_i16 = _mm256_srli_epi16::<8>(scaled_luma_i16);

    let alpha = _mm256_and_si256(pixels, alpha_mask);
    _mm256_blendv_epi8(scaled_luma_i16, alpha, alpha_mask)
}
