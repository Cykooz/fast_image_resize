use std::arch::x86_64::*;

use crate::pixels::U16x4;
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
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        multiply_alpha_row(src_row, dst_row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
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

    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = _mm256_loadu_si256(src.as_ptr() as *const __m256i);

        let factor_pixels = _mm256_shuffle_epi8(src_pixels, factor_mask);
        let factor_pixels = _mm256_or_si256(factor_pixels, max_alpha);

        let src_i32_lo = _mm256_unpacklo_epi16(src_pixels, zero);
        let factors = _mm256_unpacklo_epi16(factor_pixels, zero);
        let src_i32_lo = _mm256_add_epi32(_mm256_mullo_epi32(src_i32_lo, factors), half);
        let dst_i32_lo = _mm256_add_epi32(src_i32_lo, _mm256_srli_epi32::<16>(src_i32_lo));
        let dst_i32_lo = _mm256_srli_epi32::<16>(dst_i32_lo);

        let src_i32_hi = _mm256_unpackhi_epi16(src_pixels, zero);
        let factors = _mm256_unpackhi_epi16(factor_pixels, zero);
        let src_i32_hi = _mm256_add_epi32(_mm256_mullo_epi32(src_i32_hi, factors), half);
        let dst_i32_hi = _mm256_add_epi32(src_i32_hi, _mm256_srli_epi32::<16>(src_i32_hi));
        let dst_i32_hi = _mm256_srli_epi32::<16>(dst_i32_hi);

        let dst_pixels = _mm256_packus_epi32(dst_i32_lo, dst_i32_hi);

        _mm256_storeu_si256(dst.as_mut_ptr() as *mut __m256i, dst_pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::multiply_alpha_row(src_remainder, dst_reminder);
    }
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
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        divide_alpha_four_pixels(src.as_ptr(), dst.as_mut_ptr());
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U16x4::new([0, 0, 0, 0]); 4];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U16x4::new([0, 0, 0, 0]); 4];
        divide_alpha_four_pixels(src_pixels.as_ptr(), dst_pixels.as_mut_ptr());

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_four_pixels(src: *const U16x4, dst: *mut U16x4) {
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

    let src_pixels = _mm256_loadu_si256(src as *const __m256i);

    let alpha0_f32x8 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(src_pixels, alpha32_sh0));
    let alpha1_f32x8 = _mm256_cvtepi32_ps(_mm256_shuffle_epi8(src_pixels, alpha32_sh1));

    let pix0_f32x8 = _mm256_cvtepi32_ps(_mm256_unpacklo_epi16(src_pixels, zero));
    let pix1_f32x8 = _mm256_cvtepi32_ps(_mm256_unpacklo_epi16(src_pixels, zero));

    let scaled_pix0_f32x8 = _mm256_mul_ps(pix0_f32x8, alpha_max);
    let scaled_pix1_f32x8 = _mm256_mul_ps(pix1_f32x8, alpha_max);

    let divided_pix0_i32x8 = _mm256_cvtps_epi32(_mm256_div_ps(scaled_pix0_f32x8, alpha0_f32x8));
    let divided_pix1_i32x8 = _mm256_cvtps_epi32(_mm256_div_ps(scaled_pix1_f32x8, alpha1_f32x8));

    let two_pixels_i16x16 = _mm256_packus_epi32(divided_pix0_i32x8, divided_pix1_i32x8);
    let alpha = _mm256_and_si256(src_pixels, alpha_mask);
    let dst_pixels = _mm256_blendv_epi8(two_pixels_i16x16, alpha, alpha_mask);

    _mm256_storeu_si256(dst as *mut __m256i, dst_pixels);
}
