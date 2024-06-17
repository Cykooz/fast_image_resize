use std::arch::x86_64::*;

use crate::pixels::F32x2;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_ptr = src_chunk.as_ptr() as *const f32;
        let src_pixels01 = _mm_loadu_ps(src_ptr);
        let src_pixels23 = _mm_loadu_ps(src_ptr.add(4));
        multiply_alpha_4_pixels(src_pixels01, src_pixels23, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [F32x2]) {
    let mut chunks = row.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let src_ptr = chunk.as_ptr() as *const f32;
        let src_pixels01 = _mm_loadu_ps(src_ptr);
        let src_pixels23 = _mm_loadu_ps(src_ptr.add(4));
        multiply_alpha_4_pixels(src_pixels01, src_pixels23, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn multiply_alpha_4_pixels(pixels01: __m128, pixels23: __m128, dst_chunk: &mut [F32x2]) {
    let luma03 = _mm_shuffle_ps::<0b10_00_10_00>(pixels01, pixels23);
    let alpha03 = _mm_shuffle_ps::<0b11_01_11_01>(pixels01, pixels23);
    let multiplied_luma03 = _mm_mul_ps(luma03, alpha03);

    let dst_pixel01 = _mm_unpacklo_ps(multiplied_luma03, alpha03);
    let dst_pixel23 = _mm_unpackhi_ps(multiplied_luma03, alpha03);
    let dst_ptr = dst_chunk.as_mut_ptr() as *mut f32;
    _mm_storeu_ps(dst_ptr, dst_pixel01);
    _mm_storeu_ps(dst_ptr.add(4), dst_pixel23);
}

// Divide

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_ptr = src_chunk.as_ptr() as *const f32;
        let src_pixels01 = _mm_loadu_ps(src_ptr);
        let src_pixels23 = _mm_loadu_ps(src_ptr.add(4));
        divide_alpha_4_pixels(src_pixels01, src_pixels23, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [F32x2]) {
    let mut chunks = row.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let src_ptr = chunk.as_ptr() as *const f32;
        let src_pixels01 = _mm_loadu_ps(src_ptr);
        let src_pixels23 = _mm_loadu_ps(src_ptr.add(4));
        divide_alpha_4_pixels(src_pixels01, src_pixels23, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::divide_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_4_pixels(pixels01: __m128, pixels23: __m128, dst_chunk: &mut [F32x2]) {
    let zero = _mm_set_ps1(0.);

    let luma03 = _mm_shuffle_ps::<0b10_00_10_00>(pixels01, pixels23);
    let alpha03 = _mm_shuffle_ps::<0b11_01_11_01>(pixels01, pixels23);
    let mut multiplied_luma03 = _mm_div_ps(luma03, alpha03);

    let mask_zero = _mm_cmpneq_ps(alpha03, zero);
    multiplied_luma03 = _mm_and_ps(mask_zero, multiplied_luma03);

    let dst_pixel01 = _mm_unpacklo_ps(multiplied_luma03, alpha03);
    let dst_pixel23 = _mm_unpackhi_ps(multiplied_luma03, alpha03);
    let dst_ptr = dst_chunk.as_mut_ptr() as *mut f32;
    _mm_storeu_ps(dst_ptr, dst_pixel01);
    _mm_storeu_ps(dst_ptr.add(4), dst_pixel23);
}
