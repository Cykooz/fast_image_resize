use std::arch::x86_64::*;

use crate::convolution::optimisations::{CoefficientsI16Chunk, NormalizerGuard16};
use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::Pixel;
use crate::simd_utils;

#[inline]
pub(crate) fn vert_convolution<T>(
    src_image: TypedImageView<T>,
    mut dst_image: TypedImageViewMut<T>,
    coeffs: Coefficients,
) where
    T: Pixel<Component = u8>,
{
    let (values, window_size, bounds_per_pixel) =
        (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard16::new(values);
    let coefficients_chunks = normalizer_guard.normalized_chunks(window_size, &bounds_per_pixel);

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u8(&src_image, dst_row, coeffs_chunk, &normalizer_guard);
        }
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn vert_convolution_into_one_row_u8<T>(
    src_img: &TypedImageView<T>,
    dst_row: &mut [T],
    coeffs_chunk: CoefficientsI16Chunk,
    normalizer_guard: &NormalizerGuard16,
) where
    T: Pixel<Component = u8>,
{
    let src_width = src_img.width().get() as usize * T::components_count();
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let max_y = y_start + coeffs.len() as u32;
    let precision = normalizer_guard.precision();

    let initial = _mm_set1_epi32(1 << (precision - 1));
    let initial_256 = _mm256_set1_epi32(1 << (precision - 1));

    let mut x_in_bytes: usize = 0;
    let dst_ptr_u8 = T::components_mut(dst_row).as_mut_ptr() as *mut u8;

    // 32 components in one register - 1 = 31
    while x_in_bytes < src_width.saturating_sub(31) {
        let mut sss0 = initial_256;
        let mut sss1 = initial_256;
        let mut sss2 = initial_256;
        let mut sss3 = initial_256;

        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(s_row1);
            let components2 = T::components(s_row2);

            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_256set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadu_si256(components1, x_in_bytes); // top line
            let source2 = simd_utils::loadu_si256(components2, x_in_bytes); // bottom line

            let source = _mm256_unpacklo_epi8(source1, source2);
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_unpackhi_epi8(source1, source2);
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let mmk = _mm256_set1_epi32(k as i32);

            let source1 = simd_utils::loadu_si256(components, x_in_bytes); // top line
            let source2 = _mm256_setzero_si256(); // bottom line is empty

            let source = _mm256_unpacklo_epi8(source1, source2);
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_unpackhi_epi8(source1, _mm256_setzero_si256());
            let pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
            sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(pix, mmk));
            let pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
            sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(pix, mmk));
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

        let dst_ptr = dst_ptr_u8.add(x_in_bytes) as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, sss0);

        x_in_bytes += 32;
    }

    // 8 components in half of SSE register - 1 = 7
    while x_in_bytes < src_width.saturating_sub(7) {
        let mut sss0 = initial; // left row
        let mut sss1 = initial; // right row
        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(s_row1);
            let components2 = T::components(s_row2);
            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadl_epi64(components1, x_in_bytes); // top line
            let source2 = simd_utils::loadl_epi64(components2, x_in_bytes); // bottom line

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let mmk = _mm_set1_epi32(k as i32);

            let source1 = simd_utils::loadl_epi64(components, x_in_bytes); // top line
            let source2 = _mm_setzero_si128(); // bottom line is empty

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));
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

        let dst_ptr = dst_ptr_u8.add(x_in_bytes) as *mut __m128i;
        _mm_storel_epi64(dst_ptr, sss0);

        x_in_bytes += 8;
    }

    while x_in_bytes < src_width.saturating_sub(3) {
        let mut sss = initial;
        let mut y: u32 = 0;
        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(s_row1);
            let components2 = T::components(s_row2);
            // Load two coefficients at once
            let two_coeffs = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let row1 = simd_utils::mm_cvtsi32_si128_from_u8(components1, x_in_bytes); // top line
            let row2 = simd_utils::mm_cvtsi32_si128_from_u8(components2, x_in_bytes); // bottom line

            let pixels_u8 = _mm_unpacklo_epi8(row1, row2);
            let pixels_i16 = _mm_unpacklo_epi8(pixels_u8, _mm_setzero_si128());
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pixels_i16, two_coeffs));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let pix = simd_utils::mm_cvtepu8_epi32_from_u8(components, x_in_bytes);
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
        let dst_ptr_i32 = dst_ptr_u8.add(x_in_bytes) as *mut i32;
        *dst_ptr_i32 = _mm_cvtsi128_si32(_mm_packus_epi16(sss, sss));

        x_in_bytes += 4;
    }

    if x_in_bytes < src_width {
        let dst_u8 =
            std::slice::from_raw_parts_mut(dst_ptr_u8.add(x_in_bytes), src_width - x_in_bytes);

        for dst_pixel in dst_u8 {
            let mut ss0 = 1 << (precision - 1);
            for (dy, &k) in coeffs.iter().enumerate() {
                if let Some(src_row) = src_img.get_row(y_start + dy as u32) {
                    let src_ptr = src_row.as_ptr() as *const u8;
                    let src_component = *src_ptr.add(x_in_bytes);
                    ss0 += src_component as i32 * (k as i32);
                }
            }
            *dst_pixel = normalizer_guard.clip(ss0);
            x_in_bytes += 1;
        }
    }
}
