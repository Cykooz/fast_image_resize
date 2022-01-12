use std::arch::x86_64::*;

use crate::alpha::native;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;
use crate::simd_utils;

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_avx2(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let width = src_image.width().get() as usize;
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row_avx2(src_row, dst_row, width);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace_avx2(mut image: TypedImageViewMut<U8x4>) {
    let width = image.width().get() as usize;
    for dst_row in image.iter_rows_mut() {
        let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
        multiply_alpha_row_avx2(src_row, dst_row, width);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_row_avx2(src_row: &[U8x4], dst_row: &mut [U8x4], width: usize) {
    let zero = _mm256_setzero_si256();
    let half = _mm256_set1_epi16(128);

    const MAX_A: i32 = 0xff000000u32 as i32;
    let max_alpha = _mm256_set1_epi32(MAX_A);
    #[rustfmt::skip]
    let factor_mask = _mm256_set_epi8(
        15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3, 
        15, 15, 15, 15, 11, 11, 11, 11, 7, 7, 7, 7, 3, 3, 3, 3,
    );

    let mut x: usize = 0;
    while x < width.saturating_sub(7) {
        let src_pixels = simd_utils::loadu_si256(src_row, x);

        let factor_pixels = _mm256_shuffle_epi8(src_pixels, factor_mask);
        let factor_pixels = _mm256_or_si256(factor_pixels, max_alpha);

        let pix1 = _mm256_unpacklo_epi8(src_pixels, zero);
        let factors = _mm256_unpacklo_epi8(factor_pixels, zero);
        let pix1 = _mm256_add_epi16(_mm256_mullo_epi16(pix1, factors), half);
        let pix1 = _mm256_add_epi16(pix1, _mm256_srli_epi16::<8>(pix1));
        let pix1 = _mm256_srli_epi16::<8>(pix1);

        let pix2 = _mm256_unpackhi_epi8(src_pixels, zero);
        let factors = _mm256_unpackhi_epi8(factor_pixels, zero);
        let pix2 = _mm256_add_epi16(_mm256_mullo_epi16(pix2, factors), half);
        let pix2 = _mm256_add_epi16(pix2, _mm256_srli_epi16::<8>(pix2));
        let pix2 = _mm256_srli_epi16::<8>(pix2);

        let dst_pixels = _mm256_packus_epi16(pix1, pix2);

        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, dst_pixels);

        x += 8;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    native::mul::multiply_alpha_row_native(src_tail, dst_tail);
}
