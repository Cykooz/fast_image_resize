use std::arch::x86_64::*;

use crate::pixels::U16x2;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::sse4;

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha(
    src_image: &ImageView<U16x2>,
    dst_image: &mut ImageViewMut<U16x2>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x2>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U16x2], dst_row: &mut [U16x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm256_loadu_si256(src.as_ptr() as *const __m256i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_8_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U16x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_8_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        sse4::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_8_pixels(pixels: __m256i) -> __m256i {
    let zero = _mm256_setzero_si256();
    let half = _mm256_set1_epi32(0x8000);

    const MAX_A: i32 = 0xffff0000u32 as i32;
    let max_alpha = _mm256_set1_epi32(MAX_A);
    /*
       |L0   A0  | |L1   A1  | |L2   A2  | |L3   A3  |
       |0001 0203| |0405 0607| |0809 1011| |1213 1415|
    */
    #[rustfmt::skip]
        let factor_mask = _mm256_set_epi8(
        15, 14, 15, 14, 11, 10, 11, 10, 7, 6, 7, 6, 3, 2, 3, 2,
        15, 14, 15, 14, 11, 10, 11, 10, 7, 6, 7, 6, 3, 2, 3, 2
    );

    let factor_pixels = _mm256_shuffle_epi8(pixels, factor_mask);
    let factor_pixels = _mm256_or_si256(factor_pixels, max_alpha);

    let src_i32_lo = _mm256_unpacklo_epi16(pixels, zero);
    let factors = _mm256_unpacklo_epi16(factor_pixels, zero);
    let src_i32_lo = _mm256_add_epi32(_mm256_mullo_epi32(src_i32_lo, factors), half);
    let dst_i32_lo = _mm256_add_epi32(src_i32_lo, _mm256_srli_epi32::<16>(src_i32_lo));
    let dst_i32_lo = _mm256_srli_epi32::<16>(dst_i32_lo);

    let src_i32_hi = _mm256_unpackhi_epi16(pixels, zero);
    let factors = _mm256_unpackhi_epi16(factor_pixels, zero);
    let src_i32_hi = _mm256_add_epi32(_mm256_mullo_epi32(src_i32_hi, factors), half);
    let dst_i32_hi = _mm256_add_epi32(src_i32_hi, _mm256_srli_epi32::<16>(src_i32_hi));
    let dst_i32_hi = _mm256_srli_epi32::<16>(dst_i32_hi);

    _mm256_packus_epi32(dst_i32_lo, dst_i32_hi)
}

// Divide

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha(
    src_image: &ImageView<U16x2>,
    dst_image: &mut ImageViewMut<U16x2>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U16x2>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x2], dst_row: &mut [U16x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm256_loadu_si256(src.as_ptr() as *const __m256i);
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
        let mut src_pixels = [U16x2::new([0, 0]); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x2::new([0, 0]); 8];
        let mut pixels = _mm256_loadu_si256(src_pixels.as_ptr() as *const __m256i);
        pixels = divide_alpha_8_pixels(pixels);
        _mm256_storeu_si256(dst_pixels.as_mut_ptr() as *mut __m256i, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U16x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    // Using a simple for-loop in this case is faster than implementation with pre-reading
    for chunk in &mut chunks {
        let mut pixels = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
        pixels = divide_alpha_8_pixels(pixels);
        _mm256_storeu_si256(chunk.as_mut_ptr() as *mut __m256i, pixels);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        let mut src_pixels = [U16x2::new([0, 0]); 8];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x2::new([0, 0]); 8];
        let mut pixels = _mm256_loadu_si256(src_pixels.as_ptr() as *const __m256i);
        pixels = divide_alpha_8_pixels(pixels);
        _mm256_storeu_si256(dst_pixels.as_mut_ptr() as *mut __m256i, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_8_pixels(pixels: __m256i) -> __m256i {
    let alpha_mask = _mm256_set1_epi32(0xffff0000u32 as i32);
    let luma_mask = _mm256_set1_epi32(0xffff);
    let alpha_max = _mm256_set1_ps(65535.0);
    /*
       |L0   A0  | |L1   A1  | |L2   A2  | |L3   A3  |
       |0001 0203| |0405 0607| |0809 1011| |1213 1415|
    */
    #[rustfmt::skip]
    let alpha32_sh = _mm256_set_epi8(
        -1, -1, 15, 14, -1, -1, 11, 10, -1, -1, 7, 6, -1, -1, 3, 2,
        -1, -1, 15, 14, -1, -1, 11, 10, -1, -1, 7, 6, -1, -1, 3, 2,
    );

    let alpha_f32x8 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(pixels, alpha32_sh));
    let luma_i32x8 = _mm256_and_si256(pixels, luma_mask);
    let luma_f32x8 = _mm256_cvtepi32_ps(luma_i32x8);
    let scaled_luma_f32x8 = _mm256_mul_ps(luma_f32x8, alpha_max);
    let divided_luma_f32x8 = _mm256_div_ps(scaled_luma_f32x8, alpha_f32x8);
    let divided_luma_i32x8 = _mm256_cvtps_epi32(divided_luma_f32x8);

    let alpha = _mm256_and_si256(pixels, alpha_mask);
    _mm256_blendv_epi8(divided_luma_i32x8, alpha, alpha_mask)
}
