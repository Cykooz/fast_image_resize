use std::arch::x86_64::*;

use super::native;
use crate::pixels::U8x2;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha(
    src_view: &impl ImageView<Pixel = U8x2>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = U8x2>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiplies_alpha_8_pixels(pixels);
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
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    // Using a simple for-loop in this case is faster than implementation with pre-reading
    for chunk in &mut chunks {
        let src_pixels = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
        let dst_pixels = multiplies_alpha_8_pixels(src_pixels);
        _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, dst_pixels);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn multiplies_alpha_8_pixels(pixels: __m128i) -> __m128i {
    let zero = _mm_setzero_si128();
    let half = _mm_set1_epi16(128);
    const MAX_A: i16 = 0xff00u16 as i16;
    let max_alpha = _mm_set1_epi16(MAX_A);
    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|
    */
    let factor_mask = _mm_set_epi8(15, 15, 13, 13, 11, 11, 9, 9, 7, 7, 5, 5, 3, 3, 1, 1);

    let factor_pixels = _mm_shuffle_epi8(pixels, factor_mask);
    let factor_pixels = _mm_or_si128(factor_pixels, max_alpha);

    let src_i16_lo = _mm_unpacklo_epi8(pixels, zero);
    let factors = _mm_unpacklo_epi8(factor_pixels, zero);
    let src_i16_lo = _mm_add_epi16(_mm_mullo_epi16(src_i16_lo, factors), half);
    let dst_i16_lo = _mm_add_epi16(src_i16_lo, _mm_srli_epi16::<8>(src_i16_lo));
    let dst_i16_lo = _mm_srli_epi16::<8>(dst_i16_lo);

    let src_i16_hi = _mm_unpackhi_epi8(pixels, zero);
    let factors = _mm_unpackhi_epi8(factor_pixels, zero);
    let src_i16_hi = _mm_add_epi16(_mm_mullo_epi16(src_i16_hi, factors), half);
    let dst_i16_hi = _mm_add_epi16(src_i16_hi, _mm_srli_epi16::<8>(src_i16_hi));
    let dst_i16_hi = _mm_srli_epi16::<8>(dst_i16_hi);

    _mm_packus_epi16(dst_i16_lo, dst_i16_hi)
}

// Divide

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha(
    src_view: &impl ImageView<Pixel = U8x2>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = U8x2>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x2::new([0; 2]); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2::new([0; 2]); 8];
        let mut pixels = _mm_loadu_si128(src_pixels.as_ptr() as *const __m128i);
        pixels = divide_alpha_8_pixels(pixels);
        _mm_storeu_si128(dst_pixels.as_mut_ptr() as *mut __m128i, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U8x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        let mut src_pixels = [U8x2::new([0; 2]); 8];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2::new([0; 2]); 8];
        let mut pixels = _mm_loadu_si128(src_pixels.as_ptr() as *const __m128i);
        pixels = divide_alpha_8_pixels(pixels);
        _mm_storeu_si128(dst_pixels.as_mut_ptr() as *mut __m128i, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_8_pixels(pixels: __m128i) -> __m128i {
    let alpha_mask = _mm_set1_epi16(0xff00u16 as i16);
    let luma_mask = _mm_set1_epi16(0xff);
    let alpha32_sh_lo = _mm_set_epi8(-1, -1, -1, 7, -1, -1, -1, 5, -1, -1, -1, 3, -1, -1, -1, 1);
    let alpha32_sh_hi = _mm_set_epi8(
        -1, -1, -1, 15, -1, -1, -1, 13, -1, -1, -1, 11, -1, -1, -1, 9,
    );
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let alpha_lo_f32 = _mm_cvtepi32_ps(_mm_shuffle_epi8(pixels, alpha32_sh_lo));
    // In case of zero division the `scaled_alpha_lo_i32` will contain negative value (-2147483648).
    let scaled_alpha_lo_i32 = _mm_cvtps_epi32(_mm_div_ps(alpha_scale, alpha_lo_f32));
    let alpha_hi_f32 = _mm_cvtepi32_ps(_mm_shuffle_epi8(pixels, alpha32_sh_hi));
    let scaled_alpha_hi_i32 = _mm_cvtps_epi32(_mm_div_ps(alpha_scale, alpha_hi_f32));
    // All negative values will be stored as 0.
    let scaled_alpha_i16 = _mm_packus_epi32(scaled_alpha_lo_i32, scaled_alpha_hi_i32);

    let luma_i16 = _mm_and_si128(pixels, luma_mask);
    let luma_i16 = _mm_slli_epi16::<7>(luma_i16);
    let scaled_luma_i16 = _mm_mulhrs_epi16(luma_i16, scaled_alpha_i16);
    let scaled_luma_i16 = _mm_min_epu16(scaled_luma_i16, luma_mask);

    let alpha = _mm_and_si128(pixels, alpha_mask);
    _mm_blendv_epi8(scaled_luma_i16, alpha, alpha_mask)
}
