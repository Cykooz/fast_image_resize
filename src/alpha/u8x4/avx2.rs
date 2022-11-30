use std::arch::x86_64::*;

use crate::pixels::U8x4;
use crate::simd_utils;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::sse4;

#[target_feature(enable = "avx2")]
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

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_tail = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = simd_utils::loadu_si256(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_8_pixels(pixels);
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
unsafe fn multiply_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = simd_utils::loadu_si256(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_8_pixels(pixels);
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
unsafe fn multiply_alpha_8_pixels(pixels: __m256i) -> __m256i {
    let zero = _mm256_setzero_si256();
    let half = _mm256_set1_epi16(128);

    const MAX_A: i32 = 0xff000000u32 as i32;
    let max_alpha = _mm256_set1_epi32(MAX_A);
    #[rustfmt::skip]
    let factor_mask = _mm256_set_epi8(
        15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3,
        15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3,
    );

    let factor_pixels = _mm256_shuffle_epi8(pixels, factor_mask);
    let factor_pixels = _mm256_or_si256(factor_pixels, max_alpha);

    let pix1 = _mm256_unpacklo_epi8(pixels, zero);
    let factors = _mm256_unpacklo_epi8(factor_pixels, zero);
    let pix1 = _mm256_add_epi16(_mm256_mullo_epi16(pix1, factors), half);
    let pix1 = _mm256_add_epi16(pix1, _mm256_srli_epi16::<8>(pix1));
    let pix1 = _mm256_srli_epi16::<8>(pix1);

    let pix2 = _mm256_unpackhi_epi8(pixels, zero);
    let factors = _mm256_unpackhi_epi8(factor_pixels, zero);
    let pix2 = _mm256_add_epi16(_mm256_mullo_epi16(pix2, factors), half);
    let pix2 = _mm256_add_epi16(pix2, _mm256_srli_epi16::<8>(pix2));
    let pix2 = _mm256_srli_epi16::<8>(pix2);

    _mm256_packus_epi16(pix1, pix2)
}

// Divide

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x4>, dst_image: &mut ImageViewMut<U8x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = simd_utils::loadu_si256(src, 0);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = simd_utils::loadu_si256(chunk, 0);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );

    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        sse4::divide_alpha_row_inplace(tail);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_8_pixels(pixels: __m256i) -> __m256i {
    let zero = _mm256_setzero_si256();
    let alpha_mask = _mm256_set1_epi32(0xff000000u32 as i32);
    #[rustfmt::skip]
    let shuffle1 = _mm256_set_epi8(
        5, 4, 5, 4, 5, 4, 5, 4, 1, 0, 1, 0, 1, 0, 1, 0,
        5, 4, 5, 4, 5, 4, 5, 4, 1, 0, 1, 0, 1, 0, 1, 0,
    );
    #[rustfmt::skip]
    let shuffle2 = _mm256_set_epi8(
        13, 12, 13, 12, 13, 12, 13, 12, 9, 8, 9, 8, 9, 8, 9, 8,
        13, 12, 13, 12, 13, 12, 13, 12, 9, 8, 9, 8, 9, 8, 9, 8,
    );
    let alpha_scale = _mm256_set1_ps(255.0 * 256.0);

    let alpha_f32 = _mm256_cvtepi32_ps(_mm256_srli_epi32::<24>(pixels));
    let scaled_alpha_f32 = _mm256_div_ps(alpha_scale, alpha_f32);
    let scaled_alpha_i32 = _mm256_cvtps_epi32(scaled_alpha_f32);
    let mma0 = _mm256_shuffle_epi8(scaled_alpha_i32, shuffle1);
    let mma1 = _mm256_shuffle_epi8(scaled_alpha_i32, shuffle2);

    let pix0 = _mm256_unpacklo_epi8(zero, pixels);
    let pix1 = _mm256_unpackhi_epi8(zero, pixels);

    let pix0 = _mm256_mulhi_epu16(pix0, mma0);
    let pix1 = _mm256_mulhi_epu16(pix1, mma1);

    let alpha = _mm256_and_si256(pixels, alpha_mask);
    let rgb = _mm256_packus_epi16(pix0, pix1);
    _mm256_blendv_epi8(rgb, alpha, alpha_mask)
}
