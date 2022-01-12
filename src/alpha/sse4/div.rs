use std::arch::x86_64::*;

use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_sse4(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row_sse4(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace_sse4(mut image: TypedImageViewMut<U8x4>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        divide_alpha_row_sse4(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row_sse4(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        divide_alpha(src.as_ptr(), dst.as_mut_ptr());
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        let mut src_pixels = [U8x4(0); 4];
        src_pixels
            .iter_mut()
            .zip(src_remainder)
            .for_each(|(d, s)| *d = *s);

        let mut dst_pixels = [U8x4(0); 4];
        divide_alpha(src_pixels.as_ptr(), dst_pixels.as_mut_ptr());

        dst_pixels
            .iter()
            .zip(dst_reminder)
            .for_each(|(s, d)| *d = *s);
    }
}

#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha(src: *const U8x4, dst: *mut U8x4) {
    let zero = _mm_setzero_si128();
    let alpha_mask = _mm_set1_epi32(0xff000000u32 as i32);
    let shuffle1 = _mm_set_epi8(5, 4, 5, 4, 5, 4, 5, 4, 1, 0, 1, 0, 1, 0, 1, 0);
    let shuffle2 = _mm_set_epi8(13, 12, 13, 12, 13, 12, 13, 12, 9, 8, 9, 8, 9, 8, 9, 8);
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let src_pixels = _mm_loadu_si128(src as *const __m128i);

    let alpha_f32 = _mm_cvtepi32_ps(_mm_srli_epi32::<24>(src_pixels));
    let scaled_alpha_f32 = _mm_div_ps(alpha_scale, alpha_f32);
    // let scaled_alpha_f32 = _mm_mul_ps(alpha_scale, _mm_rcp_ps(alpha_f32));
    let scaled_alpha_i32 = _mm_cvtps_epi32(scaled_alpha_f32);
    let mma0 = _mm_shuffle_epi8(scaled_alpha_i32, shuffle1);
    let mma1 = _mm_shuffle_epi8(scaled_alpha_i32, shuffle2);

    let pix0 = _mm_unpacklo_epi8(zero, src_pixels);
    let pix1 = _mm_unpackhi_epi8(zero, src_pixels);

    let pix0 = _mm_mulhi_epu16(pix0, mma0);
    let pix1 = _mm_mulhi_epu16(pix1, mma1);

    let alpha = _mm_and_si128(src_pixels, alpha_mask);
    let rgb = _mm_packus_epi16(pix0, pix1);
    let dst_pixels = _mm_blendv_epi8(rgb, alpha, alpha_mask);

    _mm_storeu_si128(dst as *mut __m128i, dst_pixels);
}
