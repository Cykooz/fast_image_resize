use std::arch::x86_64::*;

use crate::alpha::native;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_sse4(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row_sse4(src_row, dst_row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace_sse4(mut image: TypedImageViewMut<U8x4>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        multiply_alpha_row_sse4(src_row, dst_row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn multiply_alpha_row_sse4(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    let zero = _mm_setzero_si128();
    let half = _mm_set1_epi16(128);

    const MAX_A: i32 = 0xff000000u32 as i32;
    let max_alpha = _mm_set1_epi32(MAX_A);
    let factor_mask = _mm_set_epi8(15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3);

    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src, dst) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = _mm_loadu_si128(src.as_ptr() as *const __m128i);

        let factor_pixels = _mm_shuffle_epi8(src_pixels, factor_mask);
        let factor_pixels = _mm_or_si128(factor_pixels, max_alpha);

        let pix1 = _mm_unpacklo_epi8(src_pixels, zero);
        let factors = _mm_unpacklo_epi8(factor_pixels, zero);
        let pix1 = _mm_add_epi16(_mm_mullo_epi16(pix1, factors), half);
        let pix1 = _mm_add_epi16(pix1, _mm_srli_epi16::<8>(pix1));
        let pix1 = _mm_srli_epi16::<8>(pix1);

        let pix2 = _mm_unpackhi_epi8(src_pixels, zero);
        let factors = _mm_unpackhi_epi8(factor_pixels, zero);
        let pix2 = _mm_add_epi16(_mm_mullo_epi16(pix2, factors), half);
        let pix2 = _mm_add_epi16(pix2, _mm_srli_epi16::<8>(pix2));
        let pix2 = _mm_srli_epi16::<8>(pix2);

        let dst_pixels = _mm_packus_epi16(pix1, pix2);

        _mm_storeu_si128(dst.as_mut_ptr() as *mut __m128i, dst_pixels);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::mul::multiply_alpha_row_native(src_remainder, dst_reminder);
    }
}
