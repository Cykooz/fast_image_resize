use std::arch::x86_64::*;

use crate::alpha::native;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;
use crate::simd_utils;

pub(crate) fn divide_alpha_sse2(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let width = src_image.width().get() as usize;
    let src_rows = src_image.iter_rows(0, src_image.height().get());
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            divide_alpha_row_sse2(src_row, dst_row, width);
        }
    }
}

pub(crate) fn divide_alpha_inplace_sse2(mut image: TypedImageViewMut<U8x4>) {
    let width = image.width().get() as usize;
    for dst_row in image.iter_rows_mut() {
        unsafe {
            let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
            divide_alpha_row_sse2(src_row, dst_row, width);
        }
    }
}

#[target_feature(enable = "sse2")]
unsafe fn divide_alpha_row_sse2(src_row: &[u32], dst_row: &mut [u32], width: usize) {
    let zero = _mm_setzero_si128();
    let alpha_mask = _mm_set1_epi32(0xff000000u32 as i32);
    let shuffle0 = _mm_set_epi8(5, 4, 5, 4, 5, 4, 5, 4, 1, 0, 1, 0, 1, 0, 1, 0);
    let shuffle1 = _mm_set_epi8(13, 12, 13, 12, 13, 12, 13, 12, 9, 8, 9, 8, 9, 8, 9, 8);
    let alpha_scale = _mm_set1_ps(255.0 * 256.0);

    let mut x: usize = 0;
    while x < width.saturating_sub(3) {
        let mut source = simd_utils::loadu_si128(src_row, x);

        let alpha = _mm_and_si128(source, alpha_mask);
        let alpha_f32 = _mm_cvtepi32_ps(_mm_srli_epi32::<24>(source));
        let scaled_recip_alpha_f32 = _mm_mul_ps(alpha_scale, _mm_rcp_ps(alpha_f32));
        let scaled_recip_alpha_i32 = _mm_cvtps_epi32(scaled_recip_alpha_f32);
        let mma0 = _mm_shuffle_epi8(scaled_recip_alpha_i32, shuffle0);
        let mma1 = _mm_shuffle_epi8(scaled_recip_alpha_i32, shuffle1);

        let mut pix0 = _mm_unpacklo_epi8(zero, source);
        let mut pix1 = _mm_unpackhi_epi8(zero, source);

        pix0 = _mm_mulhi_epu16(pix0, mma0);
        pix1 = _mm_mulhi_epu16(pix1, mma1);

        source = _mm_packus_epi16(pix0, pix1);
        source = _mm_blendv_epi8(source, alpha, alpha_mask);
        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m128i;
        _mm_storeu_si128(dst_ptr, source);

        x += 4;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    native::divide_alpha_row_native(src_tail, dst_tail);
}
