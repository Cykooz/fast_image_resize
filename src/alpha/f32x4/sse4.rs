use std::arch::x86_64::*;

use crate::pixels::F32x4;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
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
#[target_feature(enable = "sse4.1")]
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
#[target_feature(enable = "sse4.1")]
unsafe fn multiply_alpha_4_pixels(pixels: [__m128; 4], dst_chunk: &mut [F32x4]) {
    let r_f32x4 = _mm_mul_ps(pixels[0], pixels[3]);
    let g_f32x4 = _mm_mul_ps(pixels[1], pixels[3]);
    let b_f32x4 = _mm_mul_ps(pixels[2], pixels[3]);
    store_4_pixels([r_f32x4, g_f32x4, b_f32x4, pixels[3]], dst_chunk);
}

// Divide

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "sse4.1")]
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

#[target_feature(enable = "sse4.1")]
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
#[target_feature(enable = "sse4.1")]
unsafe fn divide_alpha_4_pixels(pixels: [__m128; 4], dst_chunk: &mut [F32x4]) {
    let mut r_f32x4 = _mm_div_ps(pixels[0], pixels[3]);
    let mut g_f32x4 = _mm_div_ps(pixels[1], pixels[3]);
    let mut b_f32x4 = _mm_div_ps(pixels[2], pixels[3]);
    let zero = _mm_setzero_ps();
    let mask_zero = _mm_cmpneq_ps(pixels[3], zero);
    r_f32x4 = _mm_and_ps(mask_zero, r_f32x4);
    g_f32x4 = _mm_and_ps(mask_zero, g_f32x4);
    b_f32x4 = _mm_and_ps(mask_zero, b_f32x4);

    store_4_pixels([r_f32x4, g_f32x4, b_f32x4, pixels[3]], dst_chunk);
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn load_4_pixels(pixels: &[F32x4]) -> [__m128; 4] {
    let ptr = pixels.as_ptr() as *const f32;
    cols_into_rows([
        _mm_loadu_ps(ptr),
        _mm_loadu_ps(ptr.add(4)),
        _mm_loadu_ps(ptr.add(8)),
        _mm_loadu_ps(ptr.add(12)),
    ])
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn store_4_pixels(pixels: [__m128; 4], dst_chunk: &mut [F32x4]) {
    let pixels = cols_into_rows(pixels);
    let mut dst_ptr = dst_chunk.as_mut_ptr() as *mut f32;
    for rgba in pixels {
        _mm_storeu_ps(dst_ptr, rgba);
        dst_ptr = dst_ptr.add(4)
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn cols_into_rows(pixels: [__m128; 4]) -> [__m128; 4] {
    let rrgg01 = _mm_unpacklo_ps(pixels[0], pixels[1]);
    let rrgg23 = _mm_unpacklo_ps(pixels[2], pixels[3]);
    let r0123 = _mm_castsi128_ps(_mm_unpacklo_epi64(
        _mm_castps_si128(rrgg01),
        _mm_castps_si128(rrgg23),
    ));
    let g0123 = _mm_castsi128_ps(_mm_unpackhi_epi64(
        _mm_castps_si128(rrgg01),
        _mm_castps_si128(rrgg23),
    ));

    let bbaa01 = _mm_unpackhi_ps(pixels[0], pixels[1]);
    let bbaa23 = _mm_unpackhi_ps(pixels[2], pixels[3]);
    let b0123 = _mm_castsi128_ps(_mm_unpacklo_epi64(
        _mm_castps_si128(bbaa01),
        _mm_castps_si128(bbaa23),
    ));
    let a0123 = _mm_castsi128_ps(_mm_unpackhi_epi64(
        _mm_castps_si128(bbaa01),
        _mm_castps_si128(bbaa23),
    ));
    [r0123, g0123, b0123, a0123]
}
