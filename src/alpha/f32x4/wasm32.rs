use core::arch::wasm32::*;

use super::native;
use crate::pixels::F32x4;
use crate::{ImageView, ImageViewMut};

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha(
    src_view: &impl ImageView<Pixel = F32x4>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x4>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[F32x4], dst_row: &mut [F32x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);
    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = load_4_pixels(src_chunk);
        multiply_alpha_4_pixels(src_pixels, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [F32x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let src_pixels = load_4_pixels(chunk);
        multiply_alpha_4_pixels(src_pixels, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn multiply_alpha_4_pixels(pixels: [v128; 4], dst_chunk: &mut [F32x4]) {
    let r_f32x4 = f32x4_mul(pixels[0], pixels[3]);
    let g_f32x4 = f32x4_mul(pixels[1], pixels[3]);
    let b_f32x4 = f32x4_mul(pixels[2], pixels[3]);
    store_4_pixels([r_f32x4, g_f32x4, b_f32x4, pixels[3]], dst_chunk);
}

// Divide

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha(
    src_view: &impl ImageView<Pixel = F32x4>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x4>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[F32x4], dst_row: &mut [F32x4]) {
    let src_chunks = src_row.chunks_exact(4);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(4);

    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = load_4_pixels(src_chunk);
        divide_alpha_4_pixels(src_pixels, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "simd128")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [F32x4]) {
    let mut chunks = row.chunks_exact_mut(4);
    for chunk in &mut chunks {
        let src_pixels = load_4_pixels(chunk);
        divide_alpha_4_pixels(src_pixels, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::divide_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn divide_alpha_4_pixels(pixels: [v128; 4], dst_chunk: &mut [F32x4]) {
    let mut r_f32x4 = f32x4_div(pixels[0], pixels[3]);
    let mut g_f32x4 = f32x4_div(pixels[1], pixels[3]);
    let mut b_f32x4 = f32x4_div(pixels[2], pixels[3]);
    let zero = f32x4_splat(0.);
    let mask_zero = f32x4_ne(pixels[3], zero);
    r_f32x4 = v128_and(mask_zero, r_f32x4);
    g_f32x4 = v128_and(mask_zero, g_f32x4);
    b_f32x4 = v128_and(mask_zero, b_f32x4);

    store_4_pixels([r_f32x4, g_f32x4, b_f32x4, pixels[3]], dst_chunk);
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn load_4_pixels(pixels: &[F32x4]) -> [v128; 4] {
    let ptr = pixels.as_ptr() as *const v128;
    cols_into_rows([
        v128_load(ptr),
        v128_load(ptr.add(1)),
        v128_load(ptr.add(2)),
        v128_load(ptr.add(3)),
    ])
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn store_4_pixels(pixels: [v128; 4], dst_chunk: &mut [F32x4]) {
    let pixels = cols_into_rows(pixels);
    let mut dst_ptr = dst_chunk.as_mut_ptr() as *mut v128;
    for rgba in pixels {
        v128_store(dst_ptr, rgba);
        dst_ptr = dst_ptr.add(1);
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn cols_into_rows(pixels: [v128; 4]) -> [v128; 4] {
    let rrgg01 = i32x4_shuffle::<0, 4, 1, 5>(pixels[0], pixels[1]);
    let rrgg23 = i32x4_shuffle::<0, 4, 1, 5>(pixels[2], pixels[3]);
    let r0123 = i64x2_shuffle::<0, 2>(rrgg01, rrgg23);
    let g0123 = i64x2_shuffle::<1, 3>(rrgg01, rrgg23);

    let bbaa01 = i32x4_shuffle::<2, 6, 3, 7>(pixels[0], pixels[1]);
    let bbaa23 = i32x4_shuffle::<2, 6, 3, 7>(pixels[2], pixels[3]);
    let b0123 = i64x2_shuffle::<0, 2>(bbaa01, bbaa23);
    let a0123 = i64x2_shuffle::<1, 3>(bbaa01, bbaa23);
    [r0123, g0123, b0123, a0123]
}
