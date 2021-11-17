use std::arch::x86_64::*;

use crate::alpha::native;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;
use crate::simd_utils;

#[allow(dead_code)]
pub(crate) fn multiply_alpha_sse2(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let width = src_image.width().get() as usize;
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            multiply_alpha_row_sse2(src_row, dst_row, width);
        }
    }
}

/// https://github.com/Wizermil/premultiply_alpha/blob/master/premultiply_alpha/premultiply_alpha.hpp#L108
/// This implementation is twice slowly than native version.
#[allow(dead_code)]
#[target_feature(enable = "sse2")]
unsafe fn multiply_alpha_row_sse2(src_row: &[U8x4], dst_row: &mut [U8x4], width: usize) {
    let mask_alpha_color_odd_255 = _mm_set1_epi32(0xff000000u32 as i32);
    let div_255 = _mm_set1_epi16(0x8081u16 as i16);

    let mask_shuffle_alpha =
        _mm_set_epi8(15, -1, 15, -1, 11, -1, 11, -1, 7, -1, 7, -1, 3, -1, 3, -1);
    let mask_shuffle_color_odd =
        _mm_set_epi8(-1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1);

    let mut x: usize = 0;
    while x < width.saturating_sub(3) {
        let mut color = simd_utils::loadu_si128(src_row, x);
        let alpha = _mm_shuffle_epi8(color, mask_shuffle_alpha);

        let mut color_even = _mm_slli_epi16::<8>(color);
        let mut color_odd = _mm_shuffle_epi8(color, mask_shuffle_color_odd);
        color_odd = _mm_or_si128(color_odd, mask_alpha_color_odd_255);

        color_odd = _mm_mulhi_epu16(color_odd, alpha);
        color_even = _mm_mulhi_epu16(color_even, alpha);

        color_odd = _mm_srli_epi16::<7>(_mm_mulhi_epu16(color_odd, div_255));
        color_even = _mm_srli_epi16::<7>(_mm_mulhi_epu16(color_even, div_255));

        color = _mm_or_si128(color_even, _mm_slli_epi16::<8>(color_odd));

        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m128i;
        _mm_storeu_si128(dst_ptr, color);

        x += 4;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    native::multiply_alpha_row_native(src_tail, dst_tail);
}
