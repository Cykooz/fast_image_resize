use std::arch::x86_64::*;

use crate::pixels::U16x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(2);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(2);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_2_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(2);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_2_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    let remainder = chunks.into_remainder();
    if !remainder.is_empty() {
        native::multiply_alpha_row_inplace(remainder);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn multiply_alpha_2_pixels(pixels: __m128i) -> __m128i {
    let zero = _mm_setzero_si128();
    let half = _mm_set1_epi32(0x8000);
    const MAX_A: i64 = 0xffff000000000000u64 as i64;
    let max_alpha = _mm_set1_epi64x(MAX_A);
    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  |
       |0001 0203 0405 0607| |0809 1011 1213 1415|
    */
    let factor_mask = _mm_set_epi8(15, 14, 15, 14, 15, 14, 15, 14, 7, 6, 7, 6, 7, 6, 7, 6);

    let factor_pixels = _mm_shuffle_epi8(pixels, factor_mask);
    let factor_pixels = _mm_or_si128(factor_pixels, max_alpha);

    let src_i32_lo = _mm_unpacklo_epi16(pixels, zero);
    let factors = _mm_unpacklo_epi16(factor_pixels, zero);
    let src_i32_lo = _mm_add_epi32(_mm_mullo_epi32(src_i32_lo, factors), half);
    let dst_i32_lo = _mm_add_epi32(src_i32_lo, _mm_srli_epi32::<16>(src_i32_lo));
    let dst_i32_lo = _mm_srli_epi32::<16>(dst_i32_lo);

    let src_i32_hi = _mm_unpackhi_epi16(pixels, zero);
    let factors = _mm_unpackhi_epi16(factor_pixels, zero);
    let src_i32_hi = _mm_add_epi32(_mm_mullo_epi32(src_i32_hi, factors), half);
    let dst_i32_hi = _mm_add_epi32(src_i32_hi, _mm_srli_epi32::<16>(src_i32_hi));
    let dst_i32_hi = _mm_srli_epi32::<16>(dst_i32_hi);

    _mm_packus_epi32(dst_i32_lo, dst_i32_hi)
}

// Divide

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(2);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(2);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_2_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    if let Some(src) = src_remainder.first() {
        let src_pixels = [*src, U16x4::new([0, 0, 0, 0])];
        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 2];

        let mut pixels = _mm_loadu_si128(src_pixels.as_ptr() as *const __m128i);
        pixels = divide_alpha_2_pixels(pixels);
        _mm_storeu_si128(dst_pixels.as_mut_ptr() as *mut __m128i, pixels);

        let dst_reminder = dst_chunks.into_remainder();
        if let Some(dst) = dst_reminder.get_mut(0) {
            *dst = dst_pixels[0];
        }
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(2);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_2_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if let Some(pixel) = reminder.first_mut() {
        let src_pixels = [*pixel, U16x4::new([0, 0, 0, 0])];
        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 2];

        let mut pixels = _mm_loadu_si128(src_pixels.as_ptr() as *const __m128i);
        pixels = divide_alpha_2_pixels(pixels);
        _mm_storeu_si128(dst_pixels.as_mut_ptr() as *mut __m128i, pixels);
        *pixel = dst_pixels[0];
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_2_pixels(pixels: __m128i) -> __m128i {
    let zero = _mm_setzero_si128();
    let alpha_mask = _mm_set1_epi64x(0xffff000000000000u64 as i64);
    let alpha_max = _mm_set1_ps(65535.0);
    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  |
       |0001 0203 0405 0607| |0809 1011 1213 1415|
    */
    let alpha32_sh0 = _mm_set_epi8(-1, -1, 7, 6, -1, -1, 7, 6, -1, -1, 7, 6, -1, -1, 7, 6);
    let alpha32_sh1 = _mm_set_epi8(
        -1, -1, 15, 14, -1, -1, 15, 14, -1, -1, 15, 14, -1, -1, 15, 14,
    );

    let alpha0_f32x4 = _mm_cvtepi32_ps(_mm_shuffle_epi8(pixels, alpha32_sh0));
    let alpha1_f32x4 = _mm_cvtepi32_ps(_mm_shuffle_epi8(pixels, alpha32_sh1));

    let pix0_f32x4 = _mm_cvtepi32_ps(_mm_unpacklo_epi16(pixels, zero));
    let pix1_f32x4 = _mm_cvtepi32_ps(_mm_unpackhi_epi16(pixels, zero));

    let scaled_pix0_f32x4 = _mm_mul_ps(pix0_f32x4, alpha_max);
    let scaled_pix1_f32x4 = _mm_mul_ps(pix1_f32x4, alpha_max);

    let divided_pix0_i32x4 = _mm_cvtps_epi32(_mm_div_ps(scaled_pix0_f32x4, alpha0_f32x4));
    let divided_pix1_i32x4 = _mm_cvtps_epi32(_mm_div_ps(scaled_pix1_f32x4, alpha1_f32x4));

    let two_pixels_i16x8 = _mm_packus_epi32(divided_pix0_i32x4, divided_pix1_i32x4);
    let alpha = _mm_and_si128(pixels, alpha_mask);
    _mm_blendv_epi8(two_pixels_i16x8, alpha, alpha_mask)
}
