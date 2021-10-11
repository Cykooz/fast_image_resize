use std::arch::x86_64::*;

use crate::alpha::native;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;
use crate::simd_utils;

pub(crate) fn divide_alpha_avx2(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let width = src_image.width().get();
    let src_rows = src_image.iter_rows(0, src_image.height().get());
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            divide_alpha_row_avx2(src_row, dst_row, width as usize);
        }
    }
}

pub(crate) fn divide_alpha_inplace_avx2(mut image: TypedImageViewMut<U8x4>) {
    let width = image.width().get() as usize;
    for dst_row in image.iter_rows_mut() {
        unsafe {
            let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
            divide_alpha_row_avx2(src_row, dst_row, width);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_row_avx2(src_row: &[u32], dst_row: &mut [u32], width: usize) {
    let mut x: usize = 0;
    let zero = _mm256_setzero_si256();
    #[rustfmt::skip]
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

    while x < width.saturating_sub(7) {
        let mut source = simd_utils::loadu_si256(src_row, x);

        let alpha_f32 = _mm256_cvtepi32_ps(_mm256_srli_epi32::<24>(source));
        let scaled_alpha_f32 = _mm256_mul_ps(alpha_scale, _mm256_rcp_ps(alpha_f32));
        let scaled_alpha_i32 = _mm256_cvtps_epi32(scaled_alpha_f32);
        let mma0 = _mm256_shuffle_epi8(scaled_alpha_i32, shuffle1);
        let mma1 = _mm256_shuffle_epi8(scaled_alpha_i32, shuffle2);

        let mut pix0 = _mm256_unpacklo_epi8(zero, source);
        let mut pix1 = _mm256_unpackhi_epi8(zero, source);

        pix0 = _mm256_mulhi_epu16(pix0, mma0);
        pix1 = _mm256_mulhi_epu16(pix1, mma1);

        let alpha = _mm256_and_si256(source, alpha_mask);
        source = _mm256_packus_epi16(pix0, pix1);
        source = _mm256_blendv_epi8(source, alpha, alpha_mask);

        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, source);

        x += 8;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    native::divide_alpha_row_native(src_tail, dst_tail);
}
