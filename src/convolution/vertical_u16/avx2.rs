use std::arch::x86_64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::PixelExt;
use crate::simd_utils;
use crate::{ImageView, ImageViewMut};

pub(crate) fn vert_convolution<T>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    offset: u32,
    coeffs: Coefficients,
) where
    T: PixelExt<Component = u16>,
{
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let src_x = offset as usize * T::count_of_components();

    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u16(src_image, dst_row, src_x, coeffs_chunk, &normalizer);
        }
    }
}

#[target_feature(enable = "avx2")]
pub(crate) unsafe fn vert_convolution_into_one_row_u16<T>(
    src_img: &ImageView<T>,
    dst_row: &mut [T],
    mut src_x: usize,
    coeffs_chunk: optimisations::CoefficientsI32Chunk,
    normalizer: &optimisations::Normalizer32,
) where
    T: PixelExt<Component = u16>,
{
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;
    let mut dst_u16 = T::components_mut(dst_row);

    /*
        |R    G    B   | |R    G    B   | |R    G   | - |B   | |R    G    B   | |R    G    B   | |R   |
        |0001 0203 0405| |0607 0809 1011| |1213 1415| - |0001| |0203 0405 0607| |0809 1011 1213| |1415|

        Shuffle to extract 0-1 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0
        hi: -1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0

        Shuffle to extract 2-3 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4
        hi: -1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4

        Shuffle to extract 4-5 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8
        hi: -1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8

        Shuffle to extract 6-7 components as i64:
        lo: -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
        hi: -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12
    */

    let shuffles = [
        _mm256_set_m128i(
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 3, 2, -1, -1, -1, -1, -1, -1, 1, 0),
        ),
        _mm256_set_m128i(
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4),
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 7, 6, -1, -1, -1, -1, -1, -1, 5, 4),
        ),
        _mm256_set_m128i(
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, 11, 10, -1, -1, -1, -1, -1, -1, 9, 8),
        ),
        _mm256_set_m128i(
            _mm_set_epi8(
                -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
            ),
            _mm_set_epi8(
                -1, -1, -1, -1, -1, -1, 15, 14, -1, -1, -1, -1, -1, -1, 13, 12,
            ),
        ),
    ];

    let precision = normalizer.precision();
    let initial = _mm256_set1_epi64x(1 << (precision - 1));
    let mut comp_buf = [0i64; 4];

    // 16 components in one register
    let mut dst_chunks_16 = dst_u16.chunks_exact_mut(16);
    for dst_chunk in &mut dst_chunks_16 {
        // 16 components / 4 per register = 4 registers
        let mut sum = [initial; 4];

        for (s_row, &coeff) in src_img.iter_rows(y_start).zip(coeffs) {
            let components = T::components(s_row);
            let coeff_i64x4 = _mm256_set1_epi64x(coeff as i64);
            let source = simd_utils::loadu_si256(components, src_x);
            for i in 0..4 {
                let comp_i64x4 = _mm256_shuffle_epi8(source, shuffles[i]);
                sum[i] = _mm256_add_epi64(sum[i], _mm256_mul_epi32(comp_i64x4, coeff_i64x4));
            }
        }

        for i in 0..4 {
            _mm256_storeu_si256(comp_buf.as_mut_ptr() as *mut __m256i, sum[i]);
            let component = dst_chunk.get_unchecked_mut(i * 2);
            *component = normalizer.clip(comp_buf[0]);
            let component = dst_chunk.get_unchecked_mut(i * 2 + 1);
            *component = normalizer.clip(comp_buf[1]);
            let component = dst_chunk.get_unchecked_mut(i * 2 + 8);
            *component = normalizer.clip(comp_buf[2]);
            let component = dst_chunk.get_unchecked_mut(i * 2 + 9);
            *component = normalizer.clip(comp_buf[3]);
        }

        src_x += 16;
    }

    dst_u16 = dst_chunks_16.into_remainder();
    if !dst_u16.is_empty() {
        // 16 components / 4 per register = 4 registers
        let mut sum = [initial; 4];
        let mut buf = [0u16; 16];

        for (s_row, &coeff) in src_img.iter_rows(y_start).zip(coeffs) {
            let components = T::components(s_row);
            for (i, &v) in components
                .get_unchecked(src_x..)
                .iter()
                .take(dst_u16.len())
                .enumerate()
            {
                buf[i] = v;
            }
            let coeff_i64x4 = _mm256_set1_epi64x(coeff as i64);
            let source = simd_utils::loadu_si256(&buf, 0);
            for i in 0..4 {
                let comp_i64x4 = _mm256_shuffle_epi8(source, shuffles[i]);
                sum[i] = _mm256_add_epi64(sum[i], _mm256_mul_epi32(comp_i64x4, coeff_i64x4));
            }
        }

        for i in 0..4 {
            _mm256_storeu_si256(comp_buf.as_mut_ptr() as *mut __m256i, sum[i]);
            let component = buf.get_unchecked_mut(i * 2);
            *component = normalizer.clip(comp_buf[0]);
            let component = buf.get_unchecked_mut(i * 2 + 1);
            *component = normalizer.clip(comp_buf[1]);
            let component = buf.get_unchecked_mut(i * 2 + 8);
            *component = normalizer.clip(comp_buf[2]);
            let component = buf.get_unchecked_mut(i * 2 + 9);
            *component = normalizer.clip(comp_buf[3]);
        }
        for (i, v) in dst_u16.iter_mut().enumerate() {
            *v = buf[i];
        }
    }
}
