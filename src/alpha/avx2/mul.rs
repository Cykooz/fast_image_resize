use std::arch::x86_64::*;

use crate::alpha::native;
use crate::{simd_utils, DstImageView, SrcImageView};

pub(crate) fn multiply_alpha_avx2(src_image: &SrcImageView, dst_image: &mut DstImageView) {
    let width = src_image.width().get() as usize;
    let src_rows = src_image.iter_rows(0, src_image.height().get());
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            multiply_alpha_row_avx2(src_row, dst_row, width);
        }
    }
}

pub(crate) fn multiply_alpha_inplace_avx2(image: &mut DstImageView) {
    let width = image.width().get() as usize;
    for dst_row in image.iter_rows_mut() {
        unsafe {
            let src_row = std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len());
            multiply_alpha_row_avx2(src_row, dst_row, width);
        }
    }
}

/// https://github.com/Wizermil/premultiply_alpha/blob/master/premultiply_alpha/premultiply_alpha.hpp#L232
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_row_avx2(src_row: &[u32], dst_row: &mut [u32], width: usize) {
    let mask_alpha_color_odd_255 = _mm256_set1_epi32(0xff000000u32 as i32);
    let div_255 = _mm256_set1_epi16(0x8081u16 as i16);
    #[rustfmt::skip]
    let mask_shuffle_alpha = _mm256_set_epi8(
        15, -1, 15, -1, 11, -1, 11, -1, 7, -1, 7, -1, 3, -1, 3, -1,
        15, -1, 15, -1, 11, -1, 11, -1, 7, -1, 7, -1, 3, -1, 3, -1,
    );
    #[rustfmt::skip]
    let mask_shuffle_color_odd = _mm256_set_epi8(
        -1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1,
        -1, -1, 13, -1, -1, -1, 9, -1, -1, -1, 5, -1, -1, -1, 1, -1,
    );

    let mut x: usize = 0;
    while x < width.saturating_sub(7) {
        let mut color = simd_utils::loadu_si256(src_row, x);

        let alpha = _mm256_shuffle_epi8(color, mask_shuffle_alpha);
        let mut color_even = _mm256_slli_epi16::<8>(color);
        let mut color_odd = _mm256_shuffle_epi8(color, mask_shuffle_color_odd);
        color_odd = _mm256_or_si256(color_odd, mask_alpha_color_odd_255);

        color_odd = _mm256_mulhi_epu16(color_odd, alpha);
        color_even = _mm256_mulhi_epu16(color_even, alpha);

        color_odd = _mm256_srli_epi16::<7>(_mm256_mulhi_epu16(color_odd, div_255));
        color_even = _mm256_srli_epi16::<7>(_mm256_mulhi_epu16(color_even, div_255));

        color = _mm256_or_si256(color_even, _mm256_slli_epi16::<8>(color_odd));

        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, color);

        x += 8;
    }

    let src_tail = &src_row[x..];
    let dst_tail = &mut dst_row[x..];
    native::multiply_alpha_row_native(src_tail, dst_tail);
}
