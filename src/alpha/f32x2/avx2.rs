use std::arch::x86_64::*;

use crate::pixels::F32x2;
use crate::{ImageView, ImageViewMut};

use super::sse4;

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha(
    src_view: &impl ImageView<Pixel = F32x2>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_ptr = src_chunk.as_ptr() as *const f32;
        let src_pixels03 = _mm256_loadu_ps(src_ptr);
        let src_pixels47 = _mm256_loadu_ps(src_ptr.add(8));
        multiply_alpha_8_pixels(src_pixels03, src_pixels47, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [F32x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let src_ptr = chunk.as_ptr() as *const f32;
        let src_pixels01 = _mm256_loadu_ps(src_ptr);
        let src_pixels23 = _mm256_loadu_ps(src_ptr.add(8));
        multiply_alpha_8_pixels(src_pixels01, src_pixels23, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        sse4::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_8_pixels(pixels03: __m256, pixels47: __m256, dst_chunk: &mut [F32x2]) {
    let luma07 = _mm256_shuffle_ps::<0b10_00_10_00>(pixels03, pixels47);
    let alpha07 = _mm256_shuffle_ps::<0b11_01_11_01>(pixels03, pixels47);
    let multiplied_luma07 = _mm256_mul_ps(luma07, alpha07);

    let dst_pixel03 = _mm256_unpacklo_ps(multiplied_luma07, alpha07);
    let dst_pixel47 = _mm256_unpackhi_ps(multiplied_luma07, alpha07);
    let dst_ptr = dst_chunk.as_mut_ptr() as *mut f32;
    _mm256_storeu_ps(dst_ptr, dst_pixel03);
    _mm256_storeu_ps(dst_ptr.add(8), dst_pixel47);
}

// Divide

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha(
    src_view: &impl ImageView<Pixel = F32x2>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_ptr = src_chunk.as_ptr() as *const f32;
        let src_pixels03 = _mm256_loadu_ps(src_ptr);
        let src_pixels47 = _mm256_loadu_ps(src_ptr.add(8));
        divide_alpha_8_pixels(src_pixels03, src_pixels47, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        sse4::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [F32x2]) {
    let mut chunks = row.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let src_ptr = chunk.as_ptr() as *const f32;
        let src_pixels01 = _mm256_loadu_ps(src_ptr);
        let src_pixels23 = _mm256_loadu_ps(src_ptr.add(8));
        divide_alpha_8_pixels(src_pixels01, src_pixels23, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        sse4::divide_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_8_pixels(pixels03: __m256, pixels47: __m256, dst_chunk: &mut [F32x2]) {
    let zero = _mm256_set1_ps(0.);

    let luma07 = _mm256_shuffle_ps::<0b10_00_10_00>(pixels03, pixels47);
    let alpha07 = _mm256_shuffle_ps::<0b11_01_11_01>(pixels03, pixels47);
    let mut multiplied_luma07 = _mm256_div_ps(luma07, alpha07);

    let mask_zero = _mm256_cmp_ps::<_CMP_NEQ_UQ>(alpha07, zero);
    multiplied_luma07 = _mm256_and_ps(mask_zero, multiplied_luma07);

    let dst_pixel03 = _mm256_unpacklo_ps(multiplied_luma07, alpha07);
    let dst_pixel47 = _mm256_unpackhi_ps(multiplied_luma07, alpha07);
    let dst_ptr = dst_chunk.as_mut_ptr() as *mut f32;
    _mm256_storeu_ps(dst_ptr, dst_pixel03);
    _mm256_storeu_ps(dst_ptr.add(8), dst_pixel47);
}
