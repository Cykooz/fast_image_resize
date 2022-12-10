use std::arch::x86_64::*;

use crate::pixels::U16x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

use super::sse4;

#[target_feature(enable = "avx2")]
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

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm256_loadu_si256(src.as_ptr() as *const __m256i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_4_pixels(pixels);
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
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = multiply_alpha_4_pixels(pixels);
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
unsafe fn multiply_alpha_4_pixels(pixels: __m256i) -> __m256i {
    let zero = _mm256_setzero_si256();
    let half = _mm256_set1_epi32(0x8000);

    const MAX_A: i64 = 0xffff000000000000u64 as i64;
    let max_alpha = _mm256_set1_epi64x(MAX_A);
    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  | |R0   G0   B0   A0  | |R1   G1   B1   A1 |
       |0001 0203 0405 0607| |0809 1011 1213 1415| |0001 0203 0405 0607| |0809 1011 1213 1415|
    */
    let factor_mask = _mm256_set_m128i(
        _mm_set_epi8(15, 14, 15, 14, 15, 14, 15, 14, 7, 6, 7, 6, 7, 6, 7, 6),
        _mm_set_epi8(15, 14, 15, 14, 15, 14, 15, 14, 7, 6, 7, 6, 7, 6, 7, 6),
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
    src_image: &ImageView<U16x4>,
    dst_image: &mut ImageViewMut<U16x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    let src_dst = src_chunks.zip(&mut dst_chunks);
    foreach_with_pre_reading(
        src_dst,
        |(src, dst)| {
            let pixels = _mm256_loadu_si256(src.as_ptr() as *const __m256i);
            let dst_ptr = dst.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_4_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U16x4::new([0, 0, 0, 0]); 4];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 4];
        let mut pixels = _mm256_loadu_si256(src_pixels.as_ptr() as *const __m256i);
        pixels = divide_alpha_4_pixels(pixels);
        _mm256_storeu_si256(dst_pixels.as_mut_ptr() as *mut __m256i, pixels);

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [U16x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    foreach_with_pre_reading(
        &mut chunks,
        |chunk| {
            let pixels = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let dst_ptr = chunk.as_mut_ptr() as *mut __m256i;
            (pixels, dst_ptr)
        },
        |(mut pixels, dst_ptr)| {
            pixels = divide_alpha_4_pixels(pixels);
            _mm256_storeu_si256(dst_ptr, pixels);
        },
    );

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        let mut src_pixels = [U16x4::new([0, 0, 0, 0]); 4];
        src_pixels
            .iter_mut()
            .zip(reminder.iter())
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 4];
        let mut pixels = _mm256_loadu_si256(src_pixels.as_ptr() as *const __m256i);
        pixels = divide_alpha_4_pixels(pixels);
        _mm256_storeu_si256(dst_pixels.as_mut_ptr() as *mut __m256i, pixels);

        dst_pixels.iter().zip(reminder).for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_4_pixels(pixels: __m256i) -> __m256i {
    let zero = _mm256_setzero_si256();
    let alpha_mask = _mm256_set1_epi64x(0xffff000000000000u64 as i64);
    let alpha_max = _mm256_set1_ps(65535.0);
    /*
       |R0   G0   B0   A0  | |R1   G1   B1   A1  | |R0   G0   B0   A0  | |R1   G1   B1   A1 |
       |0001 0203 0405 0607| |0809 1011 1213 1415| |0001 0203 0405 0607| |0809 1011 1213 1415|
    */
    let alpha32_sh0 = _mm256_set_m128i(
        _mm_set_epi8(-1, -1, 7, 6, -1, -1, 7, 6, -1, -1, 7, 6, -1, -1, 7, 6),
        _mm_set_epi8(-1, -1, 7, 6, -1, -1, 7, 6, -1, -1, 7, 6, -1, -1, 7, 6),
    );
    let alpha32_sh1 = _mm256_set_m128i(
        _mm_set_epi8(
            -1, -1, 15, 14, -1, -1, 15, 14, -1, -1, 15, 14, -1, -1, 15, 14,
        ),
        _mm_set_epi8(
            -1, -1, 15, 14, -1, -1, 15, 14, -1, -1, 15, 14, -1, -1, 15, 14,
        ),
    );

    let alpha0_f32x8 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(pixels, alpha32_sh0));
    let alpha1_f32x8 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(pixels, alpha32_sh1));

    let pix0_f32x8 = _mm256_cvtepi32_ps(_mm256_unpacklo_epi16(pixels, zero));
    let pix1_f32x8 = _mm256_cvtepi32_ps(_mm256_unpackhi_epi16(pixels, zero));

    let scaled_pix0_f32x8 = _mm256_mul_ps(pix0_f32x8, alpha_max);
    let scaled_pix1_f32x8 = _mm256_mul_ps(pix1_f32x8, alpha_max);

    let divided_pix0_i32x8 = _mm256_cvtps_epi32(_mm256_div_ps(scaled_pix0_f32x8, alpha0_f32x8));
    let divided_pix1_i32x8 = _mm256_cvtps_epi32(_mm256_div_ps(scaled_pix1_f32x8, alpha1_f32x8));

    let two_pixels_i16x16 = _mm256_packus_epi32(divided_pix0_i32x8, divided_pix1_i32x8);
    let alpha = _mm256_and_si256(pixels, alpha_mask);
    _mm256_blendv_epi8(two_pixels_i16x16, alpha, alpha_mask)
}
