use std::arch::x86_64::*;

use crate::convolution::vertical_u8::native;
use crate::convolution::{optimisations, Coefficients};
use crate::pixels::PixelExt;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn vert_convolution<T>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    offset: u32,
    coeffs: Coefficients,
) where
    T: PixelExt<Component = u8>,
{
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let src_x = offset as usize * T::count_of_components();

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u8(src_image, dst_row, src_x, coeffs_chunk, &normalizer);
        }
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn vert_convolution_into_one_row_u8<T>(
    src_img: &ImageView<T>,
    dst_row: &mut [T],
    mut src_x: usize,
    coeffs_chunk: optimisations::CoefficientsI16Chunk,
    normalizer: &optimisations::Normalizer16,
) where
    T: PixelExt<Component = u8>,
{
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let max_y = y_start + coeffs.len() as u32;
    let precision = normalizer.precision();

    let initial = _mm_set1_epi32(1 << (precision - 1));
    let initial_256 = _mm256_set1_epi32(1 << (precision - 1));

    let mut dst_u8 = T::components_mut(dst_row);

    // 32 components in one register
    let mut dst_chunks_32 = dst_u8.chunks_exact_mut(32);
    for dst_chunk in &mut dst_chunks_32 {
        let mut sss0 = initial_256;
        let mut sss1 = initial_256;
        let mut sss2 = initial_256;
        let mut sss3 = initial_256;

        let mut y: u32 = 0;

        for src_rows in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(src_rows[0]);
            let components2 = T::components(src_rows[1]);

            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_256set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadu_si256(components1, src_x); // top line
            let source2 = simd_utils::loadu_si256(components2, src_x); // bottom line

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

            let source1 = simd_utils::loadu_si256(components, src_x); // top line
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

        let dst_ptr = dst_chunk.as_mut_ptr() as *mut __m256i;
        _mm256_storeu_si256(dst_ptr, sss0);

        src_x += 32;
    }

    // 8 components in half of SSE register
    dst_u8 = dst_chunks_32.into_remainder();
    let mut dst_chunks_8 = dst_u8.chunks_exact_mut(8);
    for dst_chunk in &mut dst_chunks_8 {
        let mut sss0 = initial; // left row
        let mut sss1 = initial; // right row
        let mut y: u32 = 0;

        for src_rows in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(src_rows[0]);
            let components2 = T::components(src_rows[1]);
            // Load two coefficients at once
            let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let source1 = simd_utils::loadl_epi64(components1, src_x); // top line
            let source2 = simd_utils::loadl_epi64(components2, src_x); // bottom line

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

            let source1 = simd_utils::loadl_epi64(components, src_x); // top line
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

        let dst_ptr = dst_chunk.as_mut_ptr() as *mut __m128i;
        _mm_storel_epi64(dst_ptr, sss0);

        src_x += 8;
    }

    dst_u8 = dst_chunks_8.into_remainder();
    let mut dst_chunks_4 = dst_u8.chunks_exact_mut(4);
    if let Some(dst_chunk) = dst_chunks_4.next() {
        let mut sss = initial;
        let mut y: u32 = 0;
        for src_rows in src_img.iter_2_rows(y_start, max_y) {
            let components1 = T::components(src_rows[0]);
            let components2 = T::components(src_rows[1]);
            // Load two coefficients at once
            let two_coeffs = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

            let row1 = simd_utils::mm_cvtsi32_si128_from_u8(components1, src_x); // top line
            let row2 = simd_utils::mm_cvtsi32_si128_from_u8(components2, src_x); // bottom line

            let pixels_u8 = _mm_unpacklo_epi8(row1, row2);
            let pixels_i16 = _mm_unpacklo_epi8(pixels_u8, _mm_setzero_si128());
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pixels_i16, two_coeffs));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            let s_row = src_img.get_row(y_start + y).unwrap();
            let components = T::components(s_row);
            let pix = simd_utils::mm_cvtepu8_epi32_from_u8(components, src_x);
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
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut i32;
        dst_ptr.write_unaligned(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));

        src_x += 4;
    }

    dst_u8 = dst_chunks_4.into_remainder();
    if !dst_u8.is_empty() {
        native::convolution_by_u8(
            src_img,
            normalizer,
            1 << (precision - 1),
            dst_u8,
            src_x,
            y_start,
            coeffs,
        );
    }
}
