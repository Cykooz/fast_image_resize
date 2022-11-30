use std::arch::x86_64::*;

use crate::pixels::U8x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_4_pixels(pixels);
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
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    // Using a simple for-loop in this case is faster than implementation with pre-reading
    for chunk in &mut chunks {
        let mut pixels = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
        pixels = multiply_alpha_4_pixels(pixels);
        _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, pixels);
    }

    let tail = chunks.into_remainder();
    if !tail.is_empty() {
        native::multiply_alpha_row_inplace(tail);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn multiply_alpha_4_pixels(pixels: __m128i) -> __m128i {
    let zero = _mm_setzero_si128();
    let half = _mm_set1_epi16(128);

    const MAX_A: i32 = 0xff000000u32 as i32;
    let max_alpha = _mm_set1_epi32(MAX_A);
    let factor_mask = _mm_set_epi8(15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3);

    let factor_pixels = _mm_shuffle_epi8(pixels, factor_mask);
    let factor_pixels = _mm_or_si128(factor_pixels, max_alpha);

    let pix1 = _mm_unpacklo_epi8(pixels, zero);
    let factors = _mm_unpacklo_epi8(factor_pixels, zero);
    let pix1 = _mm_add_epi16(_mm_mullo_epi16(pix1, factors), half);
    let pix1 = _mm_add_epi16(pix1, _mm_srli_epi16::<8>(pix1));
    let pix1 = _mm_srli_epi16::<8>(pix1);

    let pix2 = _mm_unpackhi_epi8(pixels, zero);
    let factors = _mm_unpackhi_epi8(factor_pixels, zero);
    let pix2 = _mm_add_epi16(_mm_mullo_epi16(pix2, factors), half);
    let pix2 = _mm_add_epi16(pix2, _mm_srli_epi16::<8>(pix2));
    let pix2 = _mm_srli_epi16::<8>(pix2);

    _mm_packus_epi16(pix1, pix2)
}

// Divide

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x4>, dst_image: &mut ImageViewMut<U8x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_4_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
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
        let src_pixels = _mm_loadu_si128(src_buffer.as_ptr() as *const __m128i);
        let dst_pixels = divide_alpha_4_pixels(src_pixels);
        _mm_storeu_si128(dst_buffer.as_mut_ptr() as *mut __m128i, dst_pixels);

        dst_buffer
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U8x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_4_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
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
        let src_pixels = _mm_loadu_si128(src_buffer.as_ptr() as *const __m128i);
        let dst_pixels = divide_alpha_4_pixels(src_pixels);
        _mm_storeu_si128(dst_buffer.as_mut_ptr() as *mut __m128i, dst_pixels);

        dst_buffer.iter().zip(tail).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_4_pixels(src_pixels: __m128i) -> __m128i {
    let zero = _mm_setzero_si128();
    let alpha_mask = _mm_set1_epi32(0xff000000u32 as i32);
    let shuffle1 = _mm_set_epi8(5, 4, 5, 4, 5, 4, 5, 4, 1, 0, 1, 0, 1, 0, 1, 0);
    let shuffle2 = _mm_set_epi8(13, 12, 13, 12, 13, 12, 13, 12, 9, 8, 9, 8, 9, 8, 9, 8);
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let alpha_f32 = _mm_cvtepi32_ps(_mm_srli_epi32::<24>(src_pixels));
    let scaled_alpha_f32 = _mm_div_ps(alpha_scale, alpha_f32);
    let scaled_alpha_i32 = _mm_cvtps_epi32(scaled_alpha_f32);
    let mma0 = _mm_shuffle_epi8(scaled_alpha_i32, shuffle1);
    let mma1 = _mm_shuffle_epi8(scaled_alpha_i32, shuffle2);

    let pix0 = _mm_unpacklo_epi8(zero, src_pixels);
    let pix1 = _mm_unpackhi_epi8(zero, src_pixels);

    let pix0 = _mm_mulhi_epu16(pix0, mma0);
    let pix1 = _mm_mulhi_epu16(pix1, mma1);

    let alpha = _mm_and_si128(src_pixels, alpha_mask);
    let rgb = _mm_packus_epi16(pix0, pix1);

    _mm_blendv_epi8(rgb, alpha, alpha_mask)
}
