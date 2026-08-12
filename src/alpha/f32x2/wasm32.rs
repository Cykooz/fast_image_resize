use core::arch::wasm32::*;

use super::native;
use crate::pixels::F32x2;
use crate::{ImageView, ImageViewMut};

#[target_feature(enable = "simd128")]
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

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_ptr = src_chunk.as_ptr() as *const v128;
        let src_pixels01 = v128_load(src_ptr);
        let src_pixels23 = v128_load(src_ptr.add(1));
        multiply_alpha_4_pixels(src_pixels01, src_pixels23, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [F32x2]) {
    let mut chunks = row.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let src_ptr = chunk.as_ptr() as *const v128;
        let src_pixels01 = v128_load(src_ptr);
        let src_pixels23 = v128_load(src_ptr.add(1));
        multiply_alpha_4_pixels(src_pixels01, src_pixels23, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_4_pixels(pixels01: v128, pixels23: v128, dst_chunk: &mut [F32x2]) {
    let luma03 = i32x4_shuffle::<0, 2, 4, 6>(pixels01, pixels23);
    let alpha03 = i32x4_shuffle::<1, 3, 5, 7>(pixels01, pixels23);
    let multiplied_luma03 = f32x4_mul(luma03, alpha03);

    let dst_pixel01 = i32x4_shuffle::<0, 4, 1, 5>(multiplied_luma03, alpha03);
    let dst_pixel23 = i32x4_shuffle::<2, 6, 3, 7>(multiplied_luma03, alpha03);
    let dst_ptr = dst_chunk.as_mut_ptr() as *mut v128;
    v128_store(dst_ptr, dst_pixel01);
    v128_store(dst_ptr.add(1), dst_pixel23);
}

// Divide

#[target_feature(enable = "simd128")]
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

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_ptr = src_chunk.as_ptr() as *const v128;
        let src_pixels01 = v128_load(src_ptr);
        let src_pixels23 = v128_load(src_ptr.add(1));
        divide_alpha_4_pixels(src_pixels01, src_pixels23, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [F32x2]) {
    let mut chunks = row.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let src_ptr = chunk.as_ptr() as *const v128;
        let src_pixels01 = v128_load(src_ptr);
        let src_pixels23 = v128_load(src_ptr.add(1));
        divide_alpha_4_pixels(src_pixels01, src_pixels23, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::divide_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_4_pixels(pixels01: v128, pixels23: v128, dst_chunk: &mut [F32x2]) {
    let zero = f32x4_splat(0.);

    let luma03 = i32x4_shuffle::<0, 2, 4, 6>(pixels01, pixels23);
    let alpha03 = i32x4_shuffle::<1, 3, 5, 7>(pixels01, pixels23);
    let mut multiplied_luma03 = f32x4_div(luma03, alpha03);

    let mask_zero = f32x4_ne(alpha03, zero);
    multiplied_luma03 = v128_and(mask_zero, multiplied_luma03);

    let dst_pixel01 = i32x4_shuffle::<0, 4, 1, 5>(multiplied_luma03, alpha03);
    let dst_pixel23 = i32x4_shuffle::<2, 6, 3, 7>(multiplied_luma03, alpha03);
    let dst_ptr = dst_chunk.as_mut_ptr() as *mut v128;
    v128_store(dst_ptr, dst_pixel01);
    v128_store(dst_ptr.add(1), dst_pixel23);
}
