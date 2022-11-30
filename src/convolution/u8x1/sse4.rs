use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U8;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8>,
    dst_image: &mut ImageViewMut<U8>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows(src_rows, dst_rows, &coefficients_chunks, &normalizer);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_row(
                src_image.get_row(yy + offset).unwrap(),
                dst_image.get_row_mut(yy).unwrap(),
                &coefficients_chunks,
                &normalizer,
            );
        }
        yy += 1;
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U8]; 4],
    dst_rows: [&mut &mut [U8]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let zero = _mm_setzero_si128();
    let initial = 1 << (normalizer.precision() - 1);
    let mut buf = [0, 0, 0, 0, initial];

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let coeffs = coeffs_chunk.values;
        let mut x = coeffs_chunk.start as usize;
        let mut result_i32x4 = [zero, zero, zero, zero];

        let coeffs_by_8 = coeffs.chunks_exact(8);
        let reminder8 = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i16x8 = _mm_loadu_si128(k.as_ptr() as *const __m128i);
            for i in 0..4 {
                let pixels_u8x8 = simd_utils::loadl_epi64(src_rows[i], x);
                let pixels_i16x8 = _mm_cvtepu8_epi16(pixels_u8x8);
                result_i32x4[i] =
                    _mm_add_epi32(result_i32x4[i], _mm_madd_epi16(pixels_i16x8, coeffs_i16x8));
            }
            x += 8;
        }

        let mut coeffs_by_4 = reminder8.chunks_exact(4);
        let reminder4 = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i16x4 = simd_utils::loadl_epi64(k, 0);
            for i in 0..4 {
                let pixels_u8x4 = simd_utils::loadl_epi32(src_rows[i], x);
                let pixels_i16x4 = _mm_cvtepu8_epi16(pixels_u8x4);
                result_i32x4[i] =
                    _mm_add_epi32(result_i32x4[i], _mm_madd_epi16(pixels_i16x4, coeffs_i16x4));
            }
            x += 4;
        }

        let mut result_i32x4 = result_i32x4.map(|v| {
            _mm_storeu_si128(buf.as_mut_ptr() as *mut __m128i, v);
            buf.iter().sum()
        });

        for &coeff in reminder4 {
            let coeff_i32 = coeff as i32;
            for i in 0..4 {
                result_i32x4[i] += src_rows[i].get_unchecked(x).0.to_owned() as i32 * coeff_i32;
            }
            x += 1;
        }

        let result_u8x4 = result_i32x4.map(|v| normalizer.clip(v));
        for i in 0..4 {
            dst_rows[i].get_unchecked_mut(dst_x).0 = result_u8x4[i];
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_row(
    src_row: &[U8],
    dst_row: &mut [U8],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let zero = _mm_setzero_si128();
    let initial = 1 << (normalizer.precision() - 1);
    let mut buf = [0, 0, 0, 0, initial];

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let coeffs = coeffs_chunk.values;
        let mut x = coeffs_chunk.start as usize;
        let mut result_i32x4 = zero;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        let reminder8 = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i16x8 = _mm_loadu_si128(k.as_ptr() as *const __m128i);
            let pixels_u8x8 = simd_utils::loadl_epi64(src_row, x);
            let pixels_i16x8 = _mm_cvtepu8_epi16(pixels_u8x8);
            result_i32x4 = _mm_add_epi32(result_i32x4, _mm_madd_epi16(pixels_i16x8, coeffs_i16x8));
            x += 8;
        }

        let mut coeffs_by_4 = reminder8.chunks_exact(4);
        let reminder4 = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i16x4 = simd_utils::loadl_epi64(k, 0);
            let pixels_u8x4 = simd_utils::loadl_epi32(src_row, x);
            let pixels_i16x4 = _mm_cvtepu8_epi16(pixels_u8x4);
            result_i32x4 = _mm_add_epi32(result_i32x4, _mm_madd_epi16(pixels_i16x4, coeffs_i16x4));
            x += 4;
        }

        _mm_storeu_si128(buf.as_mut_ptr() as *mut __m128i, result_i32x4);
        let mut result_i32 = buf.iter().sum();

        for &coeff in reminder4 {
            let coeff_i32 = coeff as i32;
            result_i32 += src_row.get_unchecked(x).0 as i32 * coeff_i32;
            x += 1;
        }

        dst_row.get_unchecked_mut(dst_x).0 = normalizer.clip(result_i32);
    }
}
