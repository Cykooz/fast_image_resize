use std::arch::x86_64::*;

use crate::convolution::{Coefficients, CoefficientsChunk};
use crate::pixels::InnerPixel;
use crate::{simd_utils, ImageView, ImageViewMut};

use super::native;

pub(crate) fn vert_convolution<T>(
    src_view: &impl ImageView<Pixel = T>,
    dst_view: &mut impl ImageViewMut<Pixel = T>,
    offset: u32,
    coeffs: &Coefficients,
) where
    T: InnerPixel<Component = f32>,
{
    let coefficients_chunks = coeffs.get_chunks();
    let src_x = offset as usize * T::count_of_components();

    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_f32(src_view, dst_row, src_x, coeffs_chunk);
        }
    }
}

#[target_feature(enable = "sse4.1")]
unsafe fn vert_convolution_into_one_row_f32<T: InnerPixel<Component = f32>>(
    src_view: &impl ImageView<Pixel = T>,
    dst_row: &mut [T],
    mut src_x: usize,
    coeffs_chunk: CoefficientsChunk,
) {
    let mut c_buf = [0f64; 2];
    let mut dst_f32 = T::components_mut(dst_row);

    let mut dst_chunks = dst_f32.chunks_exact_mut(16);
    for dst_chunk in &mut dst_chunks {
        multiply_components_of_rows::<_, 8>(src_view, src_x, coeffs_chunk, dst_chunk, &mut c_buf);
        src_x += 16;
    }

    dst_f32 = dst_chunks.into_remainder();
    dst_chunks = dst_f32.chunks_exact_mut(8);
    for dst_chunk in &mut dst_chunks {
        multiply_components_of_rows::<_, 4>(src_view, src_x, coeffs_chunk, dst_chunk, &mut c_buf);
        src_x += 8;
    }

    dst_f32 = dst_chunks.into_remainder();
    dst_chunks = dst_f32.chunks_exact_mut(4);
    if let Some(dst_chunk) = dst_chunks.next() {
        multiply_components_of_rows::<_, 2>(src_view, src_x, coeffs_chunk, dst_chunk, &mut c_buf);
        src_x += 4;
    }

    dst_f32 = dst_chunks.into_remainder();
    if !dst_f32.is_empty() {
        let y_start = coeffs_chunk.start;
        let coeffs = coeffs_chunk.values;
        native::convolution_by_f32(src_view, dst_f32, src_x, y_start, coeffs);
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn multiply_components_of_rows<
    T: InnerPixel<Component = f32>,
    const SUMS_COUNT: usize,
>(
    src_view: &impl ImageView<Pixel = T>,
    src_x: usize,
    coeffs_chunk: CoefficientsChunk,
    dst_chunk: &mut [f32],
    c_buf: &mut [f64; 2],
) {
    let mut sums = [_mm_set1_pd(0.); SUMS_COUNT];
    let y_start = coeffs_chunk.start;
    let mut coeffs = coeffs_chunk.values;
    let mut y: u32 = 0;
    let max_rows = coeffs.len() as u32;

    let coeffs_2 = coeffs.chunks_exact(2);
    coeffs = coeffs_2.remainder();
    for (src_rows, two_coeffs) in src_view.iter_2_rows(y_start, max_rows).zip(coeffs_2) {
        let src_rows = src_rows.map(|row| T::components(row).get_unchecked(src_x..));
        for (&coeff, src_row) in two_coeffs.iter().zip(src_rows) {
            multiply_components_of_row(&mut sums, coeff, src_row);
        }
        y += 2;
    }

    if let Some(&coeff) = coeffs.first() {
        if let Some(s_row) = src_view.iter_rows(y_start + y).next() {
            let src_row = T::components(s_row).get_unchecked(src_x..);
            multiply_components_of_row(&mut sums, coeff, src_row);
        }
    }

    let mut dst_ptr = dst_chunk.as_mut_ptr();
    for sum in sums {
        _mm_storeu_pd(c_buf.as_mut_ptr(), sum);
        for &v in c_buf.iter() {
            *dst_ptr = v as f32;
            dst_ptr = dst_ptr.add(1);
        }
    }
}

#[inline]
#[target_feature(enable = "sse4.1")]
unsafe fn multiply_components_of_row<const SUMS_COUNT: usize>(
    sums: &mut [__m128d; SUMS_COUNT],
    coeff: f64,
    src_row: &[f32],
) {
    let coeff_f64x2 = _mm_set1_pd(coeff);
    let mut i = 0;
    while i < SUMS_COUNT {
        let comp03_f32x4 = simd_utils::loadu_ps(src_row, i * 2);

        let comp01_f64x2 = _mm_cvtps_pd(comp03_f32x4);
        sums[i] = _mm_add_pd(sums[i], _mm_mul_pd(comp01_f64x2, coeff_f64x2));
        i += 1;

        let comp23_f64x2 = _mm_cvtps_pd(_mm_movehl_ps(comp03_f32x4, comp03_f32x4));
        sums[i] = _mm_add_pd(sums[i], _mm_mul_pd(comp23_f64x2, coeff_f64x2));
        i += 1;
    }
}
