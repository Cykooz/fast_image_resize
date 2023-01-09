use std::arch::x86_64::*;

use crate::pixels::U8x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};
use std::fs;
use std::path::Path;

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
    let mut file: String = "sse4r".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "b";
    }
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
            debugout += &format!(
                "143 pixels: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
                _mm_extract_epi8(pixels, 0) as u8,
                _mm_extract_epi8(pixels, 1) as u8,
                _mm_extract_epi8(pixels, 2) as u8,
                _mm_extract_epi8(pixels, 3) as u8,
                _mm_extract_epi8(pixels, 4) as u8,
                _mm_extract_epi8(pixels, 5) as u8,
                _mm_extract_epi8(pixels, 6) as u8,
                _mm_extract_epi8(pixels, 7) as u8,
                _mm_extract_epi8(pixels, 8) as u8,
                _mm_extract_epi8(pixels, 9) as u8,
                _mm_extract_epi8(pixels, 10) as u8,
                _mm_extract_epi8(pixels, 11) as u8,
                _mm_extract_epi8(pixels, 12) as u8,
                _mm_extract_epi8(pixels, 13) as u8,
                _mm_extract_epi8(pixels, 14) as u8,
                _mm_extract_epi8(pixels, 15) as u8,
            );
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
        debugout += &format!(
            "165 dst_pixels: {:?} {:?} {:?} {:?}\n",
            _mm_extract_epi32(dst_pixels, 0) as u32,
            _mm_extract_epi32(dst_pixels, 1) as u32,
            _mm_extract_epi32(dst_pixels, 2) as u32,
            _mm_extract_epi32(dst_pixels, 3) as u32,
        );
        _mm_storeu_si128(dst_buffer.as_mut_ptr() as *mut __m128i, dst_pixels);

        dst_buffer
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
    fs::write(file, debugout).unwrap();
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U8x4]) {
    let mut file: String = "sse4i".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "c";
    }
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
            debugout += &format!(
                "179 pixels: {:?} {:?} {:?} {:?}\n",
                _mm_extract_epi32(pixels, 0) as u32,
                _mm_extract_epi32(pixels, 1) as u32,
                _mm_extract_epi32(pixels, 2) as u32,
                _mm_extract_epi32(pixels, 3) as u32,
            );
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
        debugout += &format!(
            "201 dst_pixels: {:?} {:?} {:?} {:?}\n",
            _mm_extract_epi32(dst_pixels, 0) as u32,
            _mm_extract_epi32(dst_pixels, 1) as u32,
            _mm_extract_epi32(dst_pixels, 2) as u32,
            _mm_extract_epi32(dst_pixels, 3) as u32,
        );
        _mm_storeu_si128(dst_buffer.as_mut_ptr() as *mut __m128i, dst_pixels);

        dst_buffer.iter().zip(tail).for_each(|(s, d)| *d = *s);
    }
    fs::write(file, debugout).unwrap();
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_4_pixels(src_pixels: __m128i) -> __m128i {
    let mut file: String = "sse48".to_string();
    let mut debugout = String::new();
    while Path::new(&file).exists() {
        file += "a";
    }
    debugout += &format!(
        "src_pixels: {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi32(src_pixels, 0) as u32,
        _mm_extract_epi32(src_pixels, 1) as u32,
        _mm_extract_epi32(src_pixels, 2) as u32,
        _mm_extract_epi32(src_pixels, 3) as u32,
    );
    let zero = _mm_setzero_si128();
    let alpha_mask = _mm_set1_epi32(0xff000000u32 as i32);
    let shuffle1 = _mm_set_epi8(5, 4, 5, 4, 5, 4, 5, 4, 1, 0, 1, 0, 1, 0, 1, 0);
    let shuffle2 = _mm_set_epi8(13, 12, 13, 12, 13, 12, 13, 12, 9, 8, 9, 8, 9, 8, 9, 8);
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let alpha_f32 = _mm_cvtepi32_ps(_mm_srli_epi32::<24>(src_pixels));
    debugout += &format!(
        "shift24: {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi32(_mm_srli_epi32::<24>(src_pixels), 0) as u32,
        _mm_extract_epi32(_mm_srli_epi32::<24>(src_pixels), 1) as u32,
        _mm_extract_epi32(_mm_srli_epi32::<24>(src_pixels), 2) as u32,
        _mm_extract_epi32(_mm_srli_epi32::<24>(src_pixels), 3) as u32,
    );
    debugout += &format!(
        "alpha_f32: {:?} {:?} {:?} {:?}\n",
        f32::from_bits(_mm_extract_ps(alpha_f32, 0) as u32),
        f32::from_bits(_mm_extract_ps(alpha_f32, 1) as u32),
        f32::from_bits(_mm_extract_ps(alpha_f32, 2) as u32),
        f32::from_bits(_mm_extract_ps(alpha_f32, 3) as u32),
    );
    let scaled_alpha_f32 = _mm_div_ps(alpha_scale, alpha_f32);
    debugout += &format!(
        "scaled_alpha_f32: {:?} {:?} {:?} {:?}\n",
        f32::from_bits(_mm_extract_ps(scaled_alpha_f32, 0) as u32),
        f32::from_bits(_mm_extract_ps(scaled_alpha_f32, 1) as u32),
        f32::from_bits(_mm_extract_ps(scaled_alpha_f32, 2) as u32),
        f32::from_bits(_mm_extract_ps(scaled_alpha_f32, 3) as u32),
    );
    let scaled_alpha_i32 = _mm_cvtps_epi32(scaled_alpha_f32);
    debugout += &format!(
        "scaled_alpha_u32: {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi32(scaled_alpha_i32, 0) as u32,
        _mm_extract_epi32(scaled_alpha_i32, 1) as u32,
        _mm_extract_epi32(scaled_alpha_i32, 2) as u32,
        _mm_extract_epi32(scaled_alpha_i32, 3) as u32,
    );
    debugout += &format!(
        "scaled_alpha_u32: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi8(scaled_alpha_i32, 0) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 1) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 2) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 3) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 4) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 5) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 6) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 7) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 8) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 9) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 10) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 11) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 12) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 13) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 14) as u8,
        _mm_extract_epi8(scaled_alpha_i32, 15) as u8,
    );
    let mma0 = _mm_shuffle_epi8(scaled_alpha_i32, shuffle1);
    let mma1 = _mm_shuffle_epi8(scaled_alpha_i32, shuffle2);

    let pix0 = _mm_unpacklo_epi8(zero, src_pixels);
    let pix1 = _mm_unpackhi_epi8(zero, src_pixels);

    let pix0 = _mm_mulhi_epu16(pix0, mma0);
    debugout += &format!(
        "pix0: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(pix0, 0) as u16,
        _mm_extract_epi16(pix0, 1) as u16,
        _mm_extract_epi16(pix0, 2) as u16,
        _mm_extract_epi16(pix0, 3) as u16,
        _mm_extract_epi16(pix0, 4) as u16,
        _mm_extract_epi16(pix0, 5) as u16,
        _mm_extract_epi16(pix0, 6) as u16,
        _mm_extract_epi16(pix0, 7) as u16,
    );
    let pix1 = _mm_mulhi_epu16(pix1, mma1);
    debugout += &format!(
        "pix1: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi16(pix1, 0) as u16,
        _mm_extract_epi16(pix1, 1) as u16,
        _mm_extract_epi16(pix1, 2) as u16,
        _mm_extract_epi16(pix1, 3) as u16,
        _mm_extract_epi16(pix1, 4) as u16,
        _mm_extract_epi16(pix1, 5) as u16,
        _mm_extract_epi16(pix1, 6) as u16,
        _mm_extract_epi16(pix1, 7) as u16,
    );

    let alpha = _mm_and_si128(src_pixels, alpha_mask);
    debugout += &format!(
        "alpha: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi8(alpha, 0) as u8,
        _mm_extract_epi8(alpha, 1) as u8,
        _mm_extract_epi8(alpha, 2) as u8,
        _mm_extract_epi8(alpha, 3) as u8,
        _mm_extract_epi8(alpha, 4) as u8,
        _mm_extract_epi8(alpha, 5) as u8,
        _mm_extract_epi8(alpha, 6) as u8,
        _mm_extract_epi8(alpha, 7) as u8,
        _mm_extract_epi8(alpha, 8) as u8,
        _mm_extract_epi8(alpha, 9) as u8,
        _mm_extract_epi8(alpha, 10) as u8,
        _mm_extract_epi8(alpha, 11) as u8,
        _mm_extract_epi8(alpha, 12) as u8,
        _mm_extract_epi8(alpha, 13) as u8,
        _mm_extract_epi8(alpha, 14) as u8,
        _mm_extract_epi8(alpha, 15) as u8,
    );
    let rgb = _mm_packus_epi16(pix0, pix1);
    debugout += &format!(
        "rgb: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n",
        _mm_extract_epi8(rgb, 0) as u8,
        _mm_extract_epi8(rgb, 1) as u8,
        _mm_extract_epi8(rgb, 2) as u8,
        _mm_extract_epi8(rgb, 3) as u8,
        _mm_extract_epi8(rgb, 4) as u8,
        _mm_extract_epi8(rgb, 5) as u8,
        _mm_extract_epi8(rgb, 6) as u8,
        _mm_extract_epi8(rgb, 7) as u8,
        _mm_extract_epi8(rgb, 8) as u8,
        _mm_extract_epi8(rgb, 9) as u8,
        _mm_extract_epi8(rgb, 10) as u8,
        _mm_extract_epi8(rgb, 11) as u8,
        _mm_extract_epi8(rgb, 12) as u8,
        _mm_extract_epi8(rgb, 13) as u8,
        _mm_extract_epi8(rgb, 14) as u8,
        _mm_extract_epi8(rgb, 15) as u8,
    );
    fs::write(file, debugout).unwrap();

    _mm_blendv_epi8(rgb, alpha, alpha_mask)
}
