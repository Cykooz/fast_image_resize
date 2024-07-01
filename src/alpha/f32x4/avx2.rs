use std::arch::x86_64::*;

use crate::pixels::F32x4;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "avx2")]
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

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row(src_row: &[F32x4], dst_row: &mut [F32x4]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);
    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = load_8_pixels(src_chunk);
        multiply_alpha_8_pixels(src_pixels, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::multiply_alpha_row(src_remainder, dst_reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn multiply_alpha_row_inplace(row: &mut [F32x4]) {
    let mut chunks = row.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let src_pixels = load_8_pixels(chunk);
        multiply_alpha_8_pixels(src_pixels, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::multiply_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn multiply_alpha_8_pixels(pixels: [__m256; 4], dst_chunk: &mut [F32x4]) {
    let r_f32x8 = _mm256_mul_ps(pixels[0], pixels[3]);
    let g_f32x8 = _mm256_mul_ps(pixels[1], pixels[3]);
    let b_f32x8 = _mm256_mul_ps(pixels[2], pixels[3]);
    store_8_pixels([r_f32x8, g_f32x8, b_f32x8, pixels[3]], dst_chunk);
}

// Divide

#[target_feature(enable = "avx2")]
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

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row(src_row: &[F32x4], dst_row: &mut [F32x4]) {
    let src_chunks = src_row.chunks_exact(8);
    let src_remainder = src_chunks.remainder();
    let mut dst_chunks = dst_row.chunks_exact_mut(8);

    for (src_chunk, dst_chunk) in src_chunks.zip(&mut dst_chunks) {
        let src_pixels = load_8_pixels(src_chunk);
        divide_alpha_8_pixels(src_pixels, dst_chunk);
    }

    if !src_remainder.is_empty() {
        let dst_reminder = dst_chunks.into_remainder();
        native::divide_alpha_row(src_remainder, dst_reminder);
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn divide_alpha_row_inplace(row: &mut [F32x4]) {
    let mut chunks = row.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let src_pixels = load_8_pixels(chunk);
        divide_alpha_8_pixels(src_pixels, chunk);
    }

    let reminder = chunks.into_remainder();
    if !reminder.is_empty() {
        native::divide_alpha_row_inplace(reminder);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn divide_alpha_8_pixels(pixels: [__m256; 4], dst_chunk: &mut [F32x4]) {
    let mut r_f32x8 = _mm256_div_ps(pixels[0], pixels[3]);
    let mut g_f32x8 = _mm256_div_ps(pixels[1], pixels[3]);
    let mut b_f32x8 = _mm256_div_ps(pixels[2], pixels[3]);

    let zero = _mm256_setzero_ps();
    let mask_zero = _mm256_cmp_ps::<_CMP_NEQ_UQ>(pixels[3], zero);
    r_f32x8 = _mm256_and_ps(mask_zero, r_f32x8);
    g_f32x8 = _mm256_and_ps(mask_zero, g_f32x8);
    b_f32x8 = _mm256_and_ps(mask_zero, b_f32x8);

    store_8_pixels([r_f32x8, g_f32x8, b_f32x8, pixels[3]], dst_chunk);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn load_8_pixels(pixels: &[F32x4]) -> [__m256; 4] {
    let ptr = pixels.as_ptr() as *const f32;
    cols_into_rows([
        _mm256_loadu_ps(ptr),
        _mm256_loadu_ps(ptr.add(8)),
        _mm256_loadu_ps(ptr.add(16)),
        _mm256_loadu_ps(ptr.add(24)),
    ])
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn store_8_pixels(pixels: [__m256; 4], dst_chunk: &mut [F32x4]) {
    let pixels = cols_into_rows(pixels);
    let mut dst_ptr = dst_chunk.as_mut_ptr() as *mut f32;
    for rgba in pixels {
        _mm256_storeu_ps(dst_ptr, rgba);
        dst_ptr = dst_ptr.add(8)
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn cols_into_rows(pixels: [__m256; 4]) -> [__m256; 4] {
    let rrgg02_rrgg13 = _mm256_unpacklo_ps(pixels[0], pixels[1]);
    let rrgg46_rrgg57 = _mm256_unpacklo_ps(pixels[2], pixels[3]);
    let r0246_r1357 = _mm256_castsi256_ps(_mm256_unpacklo_epi64(
        _mm256_castps_si256(rrgg02_rrgg13),
        _mm256_castps_si256(rrgg46_rrgg57),
    ));
    let g0246_g1357 = _mm256_castsi256_ps(_mm256_unpackhi_epi64(
        _mm256_castps_si256(rrgg02_rrgg13),
        _mm256_castps_si256(rrgg46_rrgg57),
    ));

    let bbaa02_bbaa13 = _mm256_unpackhi_ps(pixels[0], pixels[1]);
    let bbaa46_bbaa57 = _mm256_unpackhi_ps(pixels[2], pixels[3]);
    let b0246_b1357 = _mm256_castsi256_ps(_mm256_unpacklo_epi64(
        _mm256_castps_si256(bbaa02_bbaa13),
        _mm256_castps_si256(bbaa46_bbaa57),
    ));
    let a0246_a1357 = _mm256_castsi256_ps(_mm256_unpackhi_epi64(
        _mm256_castps_si256(bbaa02_bbaa13),
        _mm256_castps_si256(bbaa46_bbaa57),
    ));
    [r0246_r1357, g0246_g1357, b0246_b1357, a0246_a1357]
}
