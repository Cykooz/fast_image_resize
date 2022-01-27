use std::arch::x86_64::*;

use crate::convolution::optimisations::{CoefficientsI16Chunk, NormalizerGuard16};
use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{FourRows, FourRowsMut, TypedImageView, TypedImageViewMut};
use crate::pixels::U8;
use crate::simd_utils;

#[inline]
pub(crate) fn horiz_convolution(
    src_image: TypedImageView<U8>,
    mut dst_image: TypedImageViewMut<U8>,
    offset: u32,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds_per_pixel) =
        (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard16::new(values);
    let coefficients_chunks = normalizer_guard.normalized_chunks(window_size, &bounds_per_pixel);
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_8u4x(src_rows, dst_rows, &coefficients_chunks, &normalizer_guard);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_8u(
                src_image.get_row(yy + offset).unwrap(),
                dst_image.get_row_mut(yy).unwrap(),
                &coefficients_chunks,
                &normalizer_guard,
            );
        }
        yy += 1;
    }
}

#[inline]
pub(crate) fn vert_convolution(
    src_image: TypedImageView<U8>,
    mut dst_image: TypedImageViewMut<U8>,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds_per_pixel) =
        (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard16::new(values);
    let coefficients_chunks = normalizer_guard.normalized_chunks(window_size, &bounds_per_pixel);

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_8u(&src_image, dst_row, coeffs_chunk, &normalizer_guard);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_8u4x(
    src_rows: FourRows<U8>,
    dst_rows: FourRowsMut<U8>,
    coefficients_chunks: &[CoefficientsI16Chunk],
    normalizer_guard: &NormalizerGuard16,
) {
    let s_rows = [src_rows.0, src_rows.1, src_rows.2, src_rows.3];
    let d_rows = [dst_rows.0, dst_rows.1, dst_rows.2, dst_rows.3];
    let zero = _mm_setzero_si128();
    // 8 components will be added, use only 1/8 of the error
    let initial = _mm256_set1_epi32(1 << (normalizer_guard.precision() - 4));

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let coeffs = coeffs_chunk.values;
        let mut x = coeffs_chunk.start as usize;
        let mut result_i32x8 = [initial, initial, initial, initial];

        let coeffs_by_16 = coeffs.chunks_exact(16);
        let reminder16 = coeffs_by_16.remainder();
        for k in coeffs_by_16 {
            let coeffs_i16x16 = _mm256_loadu_si256(k.as_ptr() as *const __m256i);
            for i in 0..4 {
                let pixels_u8x16 = simd_utils::loadu_si128(s_rows[i], x);
                let pixels_i16x16 = _mm256_cvtepu8_epi16(pixels_u8x16);
                result_i32x8[i] = _mm256_add_epi32(
                    result_i32x8[i],
                    _mm256_madd_epi16(pixels_i16x16, coeffs_i16x16),
                );
            }
            x += 16;
        }

        let mut coeffs_by_8 = reminder16.chunks_exact(8);
        let reminder8 = coeffs_by_8.remainder();
        if let Some(k) = coeffs_by_8.next() {
            let coeffs_i16x8 = _mm_loadu_si128(k.as_ptr() as *const __m128i);
            for i in 0..4 {
                let pixels_u8x8 = simd_utils::loadl_epi64(s_rows[i], x);
                let pixels_i16x8 = _mm_cvtepu8_epi16(pixels_u8x8);
                result_i32x8[i] = _mm256_add_epi32(
                    result_i32x8[i],
                    _mm256_set_m128i(zero, _mm_madd_epi16(pixels_i16x8, coeffs_i16x8)),
                );
            }
            x += 8;
        }

        let mut result_i32x4 = result_i32x8.map(|v| hsum_i32x8_avx2(v));

        for &coeff in reminder8 {
            let coeff_i32 = coeff as i32;
            for i in 0..4 {
                result_i32x4[i] += s_rows[i].get_unchecked(x).0.to_owned() as i32 * coeff_i32;
            }
            x += 1;
        }

        let result_u8x4 = result_i32x4.map(|v| normalizer_guard.clip(v));
        for i in 0..4 {
            d_rows[i].get_unchecked_mut(dst_x).0 = result_u8x4[i];
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_8u(
    src_row: &[U8],
    dst_row: &mut [U8],
    coefficients_chunks: &[CoefficientsI16Chunk],
    normalizer_guard: &NormalizerGuard16,
) {
    let zero = _mm_setzero_si128();
    // 8 components will be added, use only 1/8 of the error
    let initial = _mm256_set1_epi32(1 << (normalizer_guard.precision() - 4));

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let coeffs = coeffs_chunk.values;
        let mut x = coeffs_chunk.start as usize;
        let mut result_i32x8 = initial;

        let coeffs_by_16 = coeffs.chunks_exact(16);
        let reminder16 = coeffs_by_16.remainder();
        for k in coeffs_by_16 {
            let coeffs_i16x16 = _mm256_loadu_si256(k.as_ptr() as *const __m256i);
            let pixels_u8x16 = simd_utils::loadu_si128(src_row, x);
            let pixels_i16x16 = _mm256_cvtepu8_epi16(pixels_u8x16);
            result_i32x8 = _mm256_add_epi32(
                result_i32x8,
                _mm256_madd_epi16(pixels_i16x16, coeffs_i16x16),
            );
            x += 16;
        }

        let mut coeffs_by_8 = reminder16.chunks_exact(8);
        let reminder8 = coeffs_by_8.remainder();
        if let Some(k) = coeffs_by_8.next() {
            let coeffs_i16x8 = _mm_loadu_si128(k.as_ptr() as *const __m128i);
            let pixels_u8x8 = simd_utils::loadl_epi64(src_row, x);
            let pixels_i16x8 = _mm_cvtepu8_epi16(pixels_u8x8);
            result_i32x8 = _mm256_add_epi32(
                result_i32x8,
                _mm256_set_m128i(zero, _mm_madd_epi16(pixels_i16x8, coeffs_i16x8)),
            );

            x += 8;
        }

        let mut result_i32 = hsum_i32x8_avx2(result_i32x8);

        for &coeff in reminder8 {
            let coeff_i32 = coeff as i32;
            result_i32 += src_row.get_unchecked(x).0 as i32 * coeff_i32;
            x += 1;
        }

        dst_row.get_unchecked_mut(dst_x).0 = normalizer_guard.clip(result_i32);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn vert_convolution_8u(
    src_img: &TypedImageView<U8>,
    dst_row: &mut [U8],
    coeffs_chunk: CoefficientsI16Chunk,
    normalizer_guard: &NormalizerGuard16,
) {
    let src_width = src_img.width().get() as usize;
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let max_y = y_start + coeffs.len() as u32;
    let precision = normalizer_guard.precision();

    let initial = _mm_set1_epi32(1 << (precision - 1));
    let initial_256 = _mm256_set1_epi32(1 << (precision - 1));
    let zero_128 = _mm_setzero_si128();
    let zero_256: __m256i = _mm256_setzero_si256();

    let mut x: usize = 0;
    while x < src_width.saturating_sub(31) {
        let mut sss0 = initial_256;
        let mut sss1 = initial_256;
        let mut sss2 = initial_256;
        let mut sss3 = initial_256;

        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            // Load two coefficients at once
            let two_coeffs = simd_utils::ptr_i16_to_256set1_epi32(coeffs, y as usize);
            let row1 = simd_utils::loadu_si256(s_row1, x); // top line
            let row2 = simd_utils::loadu_si256(s_row2, x); // bottom line

            let lo_pixels = _mm256_unpacklo_epi8(row1, row2);
            let lo_lo = _mm256_unpacklo_epi8(lo_pixels, zero_256);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(lo_lo, two_coeffs));
            let hi_lo = _mm256_unpackhi_epi8(lo_pixels, zero_256);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(hi_lo, two_coeffs));

            let hi_pixels = _mm256_unpackhi_epi8(row1, row2);
            let lo_hi = _mm256_unpacklo_epi8(hi_pixels, zero_256);
            sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(lo_hi, two_coeffs));
            let hi_hi = _mm256_unpackhi_epi8(hi_pixels, zero_256);
            sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(hi_hi, two_coeffs));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let one_coeff = _mm256_set1_epi32(k as i32);

            let row1 = simd_utils::loadu_si256(s_row, x); // top line
            let row2 = _mm256_setzero_si256(); // bottom line is empty

            let lo_pixels = _mm256_unpacklo_epi8(row1, row2);
            let lo_lo = _mm256_unpacklo_epi8(lo_pixels, zero_256);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(lo_lo, one_coeff));
            let hi_lo = _mm256_unpackhi_epi8(lo_pixels, zero_256);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(hi_lo, one_coeff));

            let hi_pixels = _mm256_unpackhi_epi8(row1, zero_256);
            let lo_hi = _mm256_unpacklo_epi8(hi_pixels, zero_256);
            sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(lo_hi, one_coeff));
            let hi_hi = _mm256_unpackhi_epi8(hi_pixels, zero_256);
            sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(hi_hi, one_coeff));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = _mm256_srai_epi32::<$imm8>(sss0);
                sss1 = _mm256_srai_epi32::<$imm8>(sss1);
                sss2 = _mm256_srai_epi32::<$imm8>(sss2);
                sss3 = _mm256_srai_epi32::<$imm8>(sss3);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = _mm256_packs_epi32(sss0, sss1);
        sss2 = _mm256_packs_epi32(sss2, sss3);
        sss0 = _mm256_packus_epi16(sss0, sss2);
        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, sss0);

        x += 32;
    }

    while x < src_width.saturating_sub(7) {
        let mut sss0 = initial; // left row
        let mut sss1 = initial; // right row
        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            // Load two coefficients at once
            let two_coeffs = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let row1 = simd_utils::loadl_epi64(s_row1, x); // top line
            let row2 = simd_utils::loadl_epi64(s_row2, x); // bottom line

            let pixels = _mm_unpacklo_epi8(row1, row2);
            let lo_pixels = _mm_unpacklo_epi8(pixels, zero_128);
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(lo_pixels, two_coeffs));
            let hi_pixels = _mm_unpackhi_epi8(pixels, zero_128);
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(hi_pixels, two_coeffs));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let one_coeff = _mm_set1_epi32(k as i32);

            let row1 = simd_utils::loadl_epi64(s_row, x); // top line
            let row2 = _mm_setzero_si128(); // bottom line is empty

            let pixels = _mm_unpacklo_epi8(row1, row2);
            let lo_pixels = _mm_unpacklo_epi8(pixels, zero_128);
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(lo_pixels, one_coeff));
            let hi_pixels = _mm_unpackhi_epi8(pixels, zero_128);
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(hi_pixels, one_coeff));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = _mm_srai_epi32::<$imm8>(sss0);
                sss1 = _mm_srai_epi32::<$imm8>(sss1);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = _mm_packs_epi32(sss0, sss1);
        sss0 = _mm_packus_epi16(sss0, sss0);
        let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m128i;
        _mm_storel_epi64(dst_ptr, sss0);

        x += 8;
    }

    while x < src_width.saturating_sub(3) {
        let mut sss = initial;
        let mut y: u32 = 0;
        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            // Load two coefficients at once
            let two_coeffs = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let row1 = simd_utils::mm_cvtsi32_si128_from_u8(s_row1, x); // top line
            let row2 = simd_utils::mm_cvtsi32_si128_from_u8(s_row2, x); // bottom line

            let pixels_u8 = _mm_unpacklo_epi8(row1, row2);
            let pixels_i16 = _mm_unpacklo_epi8(pixels_u8, _mm_setzero_si128());
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pixels_i16, two_coeffs));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let pix = simd_utils::mm_cvtepu8_epi32_from_u8(s_row, x);
            let mmk = _mm_set1_epi32(k as i32);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss = _mm_srai_epi32::<$imm8>(sss);
            }};
        }
        constify_imm8!(precision, call);

        sss = _mm_packs_epi32(sss, sss);
        let u8x4: [u8; 4] = _mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)).to_le_bytes();
        dst_row.get_unchecked_mut(x).0 = u8x4[0];
        dst_row.get_unchecked_mut(x + 1).0 = u8x4[1];
        dst_row.get_unchecked_mut(x + 2).0 = u8x4[2];
        dst_row.get_unchecked_mut(x + 3).0 = u8x4[3];
        x += 4;
    }

    for dst_pixel in dst_row.iter_mut().skip(x) {
        let mut ss0 = 1 << (precision - 1);
        for (dy, &k) in coeffs.iter().enumerate() {
            let src_pixel = src_img.get_pixel(x as u32, y_start + dy as u32);
            ss0 += src_pixel.0 as i32 * (k as i32);
        }
        dst_pixel.0 = normalizer_guard.clip(ss0);
        x += 1;
    }
}

// only needs AVX2
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn hsum_i32x8_avx2(v: __m256i) -> i32 {
    let sum128 = _mm_add_epi32(_mm256_castsi256_si128(v), _mm256_extracti128_si256::<1>(v));
    hsum_epi32_avx(sum128)
}

#[inline(always)]
unsafe fn hsum_epi32_avx(x: __m128i) -> i32 {
    // 3-operand non-destructive AVX lets us save a byte without needing a movdqa
    let hi64 = _mm_unpackhi_epi64(x, x);
    let sum64 = _mm_add_epi32(hi64, x);
    const I: i32 = ((2 << 6) | (3 << 4) | 1) as i32;
    let hi32 = _mm_shuffle_epi32::<I>(sum64); // Swap the low two elements
    let sum32 = _mm_add_epi32(sum64, hi32);
    _mm_cvtsi128_si32(sum32) // movd
}
