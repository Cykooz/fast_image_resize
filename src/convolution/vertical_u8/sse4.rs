use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::Pixel;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn vert_convolution<T: Pixel<Component = u8>>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u8(src_image, dst_row, coeffs_chunk, &normalizer);
        }
    }
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn vert_convolution_into_one_row_u8<T: Pixel<Component = u8>>(
    src_img: &ImageView<T>,
    dst_row: &mut [T],
    coeffs_chunk: optimisations::CoefficientsI16Chunk,
    normalizer: &optimisations::Normalizer16,
) {
    let mut xx: usize = 0;
    let src_width = src_img.width().get() as usize * T::count_of_components();
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let max_y = y_start + coeffs.len() as u32;
    let precision = normalizer.precision();
    let dst_ptr_u8 = T::components_mut(dst_row).as_mut_ptr() as *mut u8;

    let initial = _mm_set1_epi32(1 << (precision - 1));

    // 32 components in two registers - 1 = 31
    while xx < src_width.saturating_sub(31) {
        let mut sss0 = initial;
        let mut sss1 = initial;
        let mut sss2 = initial;
        let mut sss3 = initial;
        let mut sss4 = initial;
        let mut sss5 = initial;
        let mut sss6 = initial;
        let mut sss7 = initial;

        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(s_row1);
            let components2 = T::components(s_row2);

            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadu_si128(components1, xx); // top line
            let source2 = simd_utils::loadu_si128(components2, xx); // bottom line

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

            let source = _mm_unpackhi_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

            let source1 = simd_utils::loadu_si128(components1, xx + 16); // top line
            let source2 = simd_utils::loadu_si128(components2, xx + 16); // bottom line

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss4 = _mm_add_epi32(sss4, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss5 = _mm_add_epi32(sss5, _mm_madd_epi16(pix, mmk));

            let source = _mm_unpackhi_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss6 = _mm_add_epi32(sss6, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss7 = _mm_add_epi32(sss7, _mm_madd_epi16(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let mmk = _mm_set1_epi32(k as i32);

            let source1 = simd_utils::loadu_si128(components, xx); // top line

            let source = _mm_unpacklo_epi8(source1, _mm_setzero_si128());
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

            let source = _mm_unpackhi_epi8(source1, _mm_setzero_si128());
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

            let source1 = simd_utils::loadu_si128(components, xx + 16); // top line

            let source = _mm_unpacklo_epi8(source1, _mm_setzero_si128());
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss4 = _mm_add_epi32(sss4, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss5 = _mm_add_epi32(sss5, _mm_madd_epi16(pix, mmk));

            let source = _mm_unpackhi_epi8(source1, _mm_setzero_si128());
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss6 = _mm_add_epi32(sss6, _mm_madd_epi16(pix, mmk));
            let pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
            sss7 = _mm_add_epi32(sss7, _mm_madd_epi16(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = _mm_srai_epi32::<$imm8>(sss0);
                sss1 = _mm_srai_epi32::<$imm8>(sss1);
                sss2 = _mm_srai_epi32::<$imm8>(sss2);
                sss3 = _mm_srai_epi32::<$imm8>(sss3);
                sss4 = _mm_srai_epi32::<$imm8>(sss4);
                sss5 = _mm_srai_epi32::<$imm8>(sss5);
                sss6 = _mm_srai_epi32::<$imm8>(sss6);
                sss7 = _mm_srai_epi32::<$imm8>(sss7);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = _mm_packs_epi32(sss0, sss1);
        sss2 = _mm_packs_epi32(sss2, sss3);
        sss0 = _mm_packus_epi16(sss0, sss2);
        let dst_ptr = dst_ptr_u8.add(xx) as *mut __m128i;
        _mm_storeu_si128(dst_ptr, sss0);
        sss4 = _mm_packs_epi32(sss4, sss5);
        sss6 = _mm_packs_epi32(sss6, sss7);
        sss4 = _mm_packus_epi16(sss4, sss6);
        let dst_ptr = dst_ptr_u8.add(xx + 16) as *mut __m128i;
        _mm_storeu_si128(dst_ptr, sss4);

        xx += 32;
    }

    while xx < src_width.saturating_sub(7) {
        let mut sss0 = initial; // left row
        let mut sss1 = initial; // right row
        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(s_row1);
            let components2 = T::components(s_row2);
            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadl_epi64(components1, xx); // top line
            let source2 = simd_utils::loadl_epi64(components2, xx); // bottom line

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

            let source1 = simd_utils::loadl_epi64(components, xx); // top line

            let source = _mm_unpacklo_epi8(source1, _mm_setzero_si128());
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
        let dst_ptr = dst_ptr_u8.add(xx) as *mut __m128i;
        _mm_storel_epi64(dst_ptr, sss0);

        xx += 8;
    }

    while xx < src_width.saturating_sub(3) {
        let mut sss = initial;
        let mut y: u32 = 0;

        for (s_row1, s_row2) in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(s_row1);
            let components2 = T::components(s_row2);
            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::mm_cvtsi32_si128_from_u8(components1, xx); // top line
            let source2 = simd_utils::mm_cvtsi32_si128_from_u8(components2, xx); // bottom line

            let source = _mm_unpacklo_epi8(source1, source2);
            let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let pix = simd_utils::mm_cvtepu8_epi32_from_u8(components, xx);
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
        let dst_ptr = dst_ptr_u8.add(xx) as *mut i32;
        *dst_ptr = _mm_cvtsi128_si32(_mm_packus_epi16(sss, sss));

        xx += 4;
    }

    if xx < src_width {
        let dst_u8 = std::slice::from_raw_parts_mut(dst_ptr_u8.add(xx), src_width - xx);

        for dst_pixel in dst_u8 {
            let mut ss0 = 1 << (precision - 1);
            for (dy, &k) in coeffs.iter().enumerate() {
                if let Some(src_row) = src_img.get_row(y_start + dy as u32) {
                    let src_ptr = src_row.as_ptr() as *const u8;
                    let src_component = *src_ptr.add(xx);
                    ss0 += src_component as i32 * (k as i32);
                }
            }
            *dst_pixel = normalizer.clip(ss0);
            xx += 1;
        }
    }
}
