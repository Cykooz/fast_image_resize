use std::arch::x86_64::*;

use crate::alpha::sse4;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_avx2(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row_avx2(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace_avx2(mut image: TypedImageViewMut<U8x4>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        divide_alpha_row_avx2(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_row_avx2(src_row: &[U8x4], dst_row: &mut [U8x4]) {
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

    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = _mm256_loadu_si256(src.as_ptr() as *const __m256i);

        let alpha_f32 = _mm256_cvtepi32_ps(_mm256_srli_epi32::<24>(src_pixels));
        let scaled_alpha_f32 = _mm256_div_ps(alpha_scale, alpha_f32);
        let scaled_alpha_i32 = _mm256_cvtps_epi32(scaled_alpha_f32);
        let mma0 = _mm256_shuffle_epi8(scaled_alpha_i32, shuffle1);
        let mma1 = _mm256_shuffle_epi8(scaled_alpha_i32, shuffle2);

        let pix0 = _mm256_unpacklo_epi8(zero, src_pixels);
        let pix1 = _mm256_unpackhi_epi8(zero, src_pixels);

        let pix0 = _mm256_mulhi_epu16(pix0, mma0);
        let pix1 = _mm256_mulhi_epu16(pix1, mma1);

        let alpha = _mm256_and_si256(src_pixels, alpha_mask);
        let rgb = _mm256_packus_epi16(pix0, pix1);
        let dst_pixels = _mm256_blendv_epi8(rgb, alpha, alpha_mask);

        _mm256_storeu_si256(dst.as_mut_ptr() as *mut __m256i, dst_pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::div::divide_alpha_row_sse4(src_remainder, dst_reminder);
    }
}
