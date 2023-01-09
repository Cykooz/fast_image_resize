use std::arch::x86_64::*;

use crate::pixels::U8x2;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};
use std::fs;
use std::path::Path;

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
    for row in image.iter_rows_mut() {
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
pub(crate) unsafe fn divide_alpha(src_image: &ImageView<U8x2>, dst_image: &mut ImageViewMut<U8x2>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U8x2>) {
    for row in image.iter_rows_mut() {
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
    let file = "sse4";
    let mut debugout = String::new();
    let file_exists = Path::new(file).exists();
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m128i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_8_pixels(pixels);
            if !file_exists {
                debugout += &format!(
                    "142: {:?} {:?}\n",
                    _mm_extract_epi64(pixels, 0),
                    _mm_extract_epi64(pixels, 1)
                );
            }
            _mm_storeu_si128(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x2::new(0); 8];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2::new(0); 8];
        let mut pixels = _mm_loadu_si128(src_pixels.as_ptr() as *const __m128i);
        pixels = divide_alpha_8_pixels(pixels);
        if !file_exists {
            debugout += &format!(
                "164: {:?} {:?}\n",
                _mm_extract_epi64(pixels, 0),
                _mm_extract_epi64(pixels, 1)
            );
        }
        _mm_storeu_si128(dst_pixels.as_mut_ptr() as *mut __m128i, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
    if !file_exists {
        fs::write(file, debugout).unwrap();
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
        let mut src_pixels = [U8x2::new(0); 8];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x2::new(0); 8];
        let mut pixels = _mm_loadu_si128(src_pixels.as_ptr() as *const __m128i);
        pixels = divide_alpha_8_pixels(pixels);
        _mm_storeu_si128(dst_pixels.as_mut_ptr() as *mut __m128i, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_8_pixels(pixels: __m128i) -> __m128i {
    let mut file: String = "sse48".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "a";
    }
    debugout += &format!(
        "pixels: {:?} {:?}\n",
        _mm_extract_epi64(pixels, 0),
        _mm_extract_epi64(pixels, 1)
    );
    let alpha_mask = _mm_set1_epi16(0xff00u16 as i16);
    let luma_mask = _mm_set1_epi16(0xff);
    let alpha32_sh_lo = _mm_set_epi8(-1, -1, -1, 7, -1, -1, -1, 5, -1, -1, -1, 3, -1, -1, -1, 1);
    let alpha32_sh_hi = _mm_set_epi8(
        -1, -1, -1, 15, -1, -1, -1, 13, -1, -1, -1, 11, -1, -1, -1, 9,
    );
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let alpha_lo_f32 = _mm_cvtepi32_ps(_mm_shuffle_epi8(pixels, alpha32_sh_lo));
    debugout += &format!(
        "alpha_lo_f32: {:?} {:?} {:?} {:?}\n",
        f32::from_bits(_mm_extract_ps(alpha_lo_f32, 0) as u32),
        f32::from_bits(_mm_extract_ps(alpha_lo_f32, 1) as u32),
        f32::from_bits(_mm_extract_ps(alpha_lo_f32, 2) as u32),
        f32::from_bits(_mm_extract_ps(alpha_lo_f32, 3) as u32),
    );
    let scaled_alpha_lo_i32 = _mm_cvtps_epi32(_mm_div_ps(alpha_scale, alpha_lo_f32));
    debugout += &format!(
        "scaled_alpha_lo_i32: {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi32(scaled_alpha_lo_i32, 0) as u32,
        _mm_extract_epi32(scaled_alpha_lo_i32, 1) as u32,
        _mm_extract_epi32(scaled_alpha_lo_i32, 2) as u32,
        _mm_extract_epi32(scaled_alpha_lo_i32, 3) as u32,
    );
    debugout += &format!(
        "as_i16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(scaled_alpha_lo_i32, 0) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 1) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 2) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 3) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 4) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 5) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 6) as u32,
        _mm_extract_epi16(scaled_alpha_lo_i32, 7) as u32,
    );
    let alpha_hi_f32 = _mm_cvtepi32_ps(_mm_shuffle_epi8(pixels, alpha32_sh_hi));
    debugout += &format!(
        "alpha_hi_f32: {:?} {:?} {:?} {:?}\n",
        f32::from_bits(_mm_extract_ps(alpha_hi_f32, 0) as u32),
        f32::from_bits(_mm_extract_ps(alpha_hi_f32, 1) as u32),
        f32::from_bits(_mm_extract_ps(alpha_hi_f32, 2) as u32),
        f32::from_bits(_mm_extract_ps(alpha_hi_f32, 3) as u32),
    );
    let scaled_alpha_hi_i32 = _mm_cvtps_epi32(_mm_div_ps(alpha_scale, alpha_hi_f32));
    debugout += &format!(
        "scaled_alpha_hi_f32: {:?} {:?} {:?} {:?}\n",
        f32::from_bits(_mm_extract_ps(_mm_div_ps(alpha_scale, alpha_hi_f32), 0) as u32),
        f32::from_bits(_mm_extract_ps(_mm_div_ps(alpha_scale, alpha_hi_f32), 1) as u32),
        f32::from_bits(_mm_extract_ps(_mm_div_ps(alpha_scale, alpha_hi_f32), 2) as u32),
        f32::from_bits(_mm_extract_ps(_mm_div_ps(alpha_scale, alpha_hi_f32), 3) as u32),
    );
    debugout += &format!(
        "scaled_alpha_hi_i32: {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi32(scaled_alpha_hi_i32, 0) as u32,
        _mm_extract_epi32(scaled_alpha_hi_i32, 1) as u32,
        _mm_extract_epi32(scaled_alpha_hi_i32, 2) as u32,
        _mm_extract_epi32(scaled_alpha_hi_i32, 3) as u32,
    );
    debugout += &format!(
        "as_i16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(scaled_alpha_hi_i32, 0) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 1) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 2) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 3) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 4) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 5) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 6) as u16,
        _mm_extract_epi16(scaled_alpha_hi_i32, 7) as u16,
    );
    let scaled_alpha_i16 = _mm_packus_epi32(scaled_alpha_lo_i32, scaled_alpha_hi_i32);
    debugout += &format!(
        "scaled_alpha_i16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(scaled_alpha_i16, 0) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 1) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 2) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 3) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 4) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 5) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 6) as u16,
        _mm_extract_epi16(scaled_alpha_i16, 7) as u16,
    );

    let luma_i16 = _mm_and_si128(pixels, luma_mask);
    debugout += &format!(
        "luma_i16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(luma_i16, 0) as u16,
        _mm_extract_epi16(luma_i16, 1) as u16,
        _mm_extract_epi16(luma_i16, 2) as u16,
        _mm_extract_epi16(luma_i16, 3) as u16,
        _mm_extract_epi16(luma_i16, 4) as u16,
        _mm_extract_epi16(luma_i16, 5) as u16,
        _mm_extract_epi16(luma_i16, 6) as u16,
        _mm_extract_epi16(luma_i16, 7) as u16,
    );
    let scaled_luma_i16 = _mm_mullo_epi16(luma_i16, scaled_alpha_i16);
    let scaled_luma_i16 = _mm_srli_epi16::<8>(scaled_luma_i16);
    debugout += &format!(
        "scaled_luma_i16: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(scaled_luma_i16, 0) as u16,
        _mm_extract_epi16(scaled_luma_i16, 1) as u16,
        _mm_extract_epi16(scaled_luma_i16, 2) as u16,
        _mm_extract_epi16(scaled_luma_i16, 3) as u16,
        _mm_extract_epi16(scaled_luma_i16, 4) as u16,
        _mm_extract_epi16(scaled_luma_i16, 5) as u16,
        _mm_extract_epi16(scaled_luma_i16, 6) as u16,
        _mm_extract_epi16(scaled_luma_i16, 7) as u16,
    );

    let alpha = _mm_and_si128(pixels, alpha_mask);
    fs::write(file, debugout).unwrap();
    _mm_blendv_epi8(scaled_luma_i16, alpha, alpha_mask)
}
