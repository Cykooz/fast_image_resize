use std::arch::x86_64::*;

use crate::pixels::U8x2;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        multiply_alpha_row(src_row, dst_row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let zero = _mm_setzero_si128();
    let half = _mm_set1_epi16(128);

    const MAX_A: i16 = 0xff00u16 as i16;
    let max_alpha = _mm_set1_epi16(MAX_A);
    /*
       |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A | |L  A |
       |00 01| |02 03| |04 05| |06 07| |08 09| |10 11| |12 13| |14 15|
    */
    let factor_mask = _mm_set_epi8(15, 15, 13, 13, 11, 11, 9, 9, 7, 7, 5, 5, 3, 3, 1, 1);

    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);

        let factor_pixels = _mm_shuffle_epi8(src_pixels, factor_mask);
        let factor_pixels = _mm_or_si128(factor_pixels, max_alpha);

        let src_i16_lo = _mm_unpacklo_epi8(src_pixels, zero);
        let factors = _mm_unpacklo_epi8(factor_pixels, zero);
        let src_i16_lo = _mm_add_epi16(_mm_mullo_epi16(src_i16_lo, factors), half);
        let dst_i16_lo = _mm_add_epi16(src_i16_lo, _mm_srli_epi16::<8>(src_i16_lo));
        let dst_i16_lo = _mm_srli_epi16::<8>(dst_i16_lo);

        let src_i16_hi = _mm_unpackhi_epi8(src_pixels, zero);
        let factors = _mm_unpackhi_epi8(factor_pixels, zero);
        let src_i16_hi = _mm_add_epi16(_mm_mullo_epi16(src_i16_hi, factors), half);
        let dst_i16_hi = _mm_add_epi16(src_i16_hi, _mm_srli_epi16::<8>(src_i16_hi));
        let dst_i16_hi = _mm_srli_epi16::<8>(dst_i16_hi);

        let dst_pixels = _mm_packus_epi16(dst_i16_lo, dst_i16_hi);

        _mm_storeu_si128(dst.as_mut_ptr() as *mut __m128i, dst_pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

// Divide

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x2>, dst_image: &mut ImageViewMut<U8x2>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U8x2], dst_row: &mut [U8x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        divide_alpha_eight_pixels(src.as_ptr(), dst.as_mut_ptr());
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x2(0); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2(0); 8];
        divide_alpha_eight_pixels(src_pixels.as_ptr(), dst_pixels.as_mut_ptr());

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_eight_pixels(src: *const U8x2, dst: *mut U8x2) {
    let alpha_mask = _mm_set1_epi16(0xff00u16 as i16);
    let luma_mask = _mm_set1_epi16(0xff);
    let alpha32_sh_lo = _mm_set_epi8(-1, -1, -1, 7, -1, -1, -1, 5, -1, -1, -1, 3, -1, -1, -1, 1);
    let alpha32_sh_hi = _mm_set_epi8(
        -1, -1, -1, 15, -1, -1, -1, 13, -1, -1, -1, 11, -1, -1, -1, 9,
    );
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let src_pixels = _mm_loadu_si128(src as *const __m128i);

    let alpha_lo_f32 = _mm_cvtepi32_ps(_mm_shuffle_epi8(src_pixels, alpha32_sh_lo));
    let scaled_alpha_lo_i32 = _mm_cvtps_epi32(_mm_div_ps(alpha_scale, alpha_lo_f32));
    let alpha_hi_f32 = _mm_cvtepi32_ps(_mm_shuffle_epi8(src_pixels, alpha32_sh_hi));
    let scaled_alpha_hi_i32 = _mm_cvtps_epi32(_mm_div_ps(alpha_scale, alpha_hi_f32));
    let scaled_alpha_i16 = _mm_packus_epi32(scaled_alpha_lo_i32, scaled_alpha_hi_i32);

    let luma_i16 = _mm_and_si128(src_pixels, luma_mask);
    let scaled_luma_i16 = _mm_mullo_epi16(luma_i16, scaled_alpha_i16);
    let scaled_luma_i16 = _mm_srli_epi16::<8>(scaled_luma_i16);

    let alpha = _mm_and_si128(src_pixels, alpha_mask);
    let dst_pixels = _mm_blendv_epi8(scaled_luma_i16, alpha, alpha_mask);
    _mm_storeu_si128(dst as *mut __m128i, dst_pixels);
}
