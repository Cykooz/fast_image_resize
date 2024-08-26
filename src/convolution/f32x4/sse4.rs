use std::arch::x86_64::*;

use crate::convolution::{Coefficients, CoefficientsChunk};
use crate::pixels::F32x4;
use crate::{simd_utils, ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = F32x4>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x4>,
    offset: u32,
    coeffs: &Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_rows(src_rows, dst_rows, &coefficients_chunks);
        }
    }

    let yy = dst_height - dst_height % 4;
    let src_rows = src_view.iter_rows(yy + offset);
    let dst_rows = dst_view.iter_rows_mut(yy);
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            horiz_convolution_rows([src_row], [dst_row], &coefficients_chunks);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_rows<const ROWS_COUNT: usize>(
    src_rows: [&[F32x4]; ROWS_COUNT],
    dst_rows: [&mut [F32x4]; ROWS_COUNT],
    coefficients_chunks: &[CoefficientsChunk],
) {
    let mut rg_buf = [0f64; 2];
    let mut ba_buf = [0f64; 2];

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sums = [_mm_set1_pd(0.); ROWS_COUNT];
        let mut ba_sums = [_mm_set1_pd(0.); ROWS_COUNT];

        for &k in coeffs_chunk.values {
            let coeffs_f64x2 = _mm_set1_pd(k);

            for r in 0..ROWS_COUNT {
                let pixel = simd_utils::loadu_ps(src_rows[r], x);
                let rg_f64x2 = _mm_cvtps_pd(pixel);
                rg_sums[r] = _mm_add_pd(rg_sums[r], _mm_mul_pd(rg_f64x2, coeffs_f64x2));
                let ba_f64x2 = _mm_cvtps_pd(_mm_movehl_ps(pixel, pixel));
                ba_sums[r] = _mm_add_pd(ba_sums[r], _mm_mul_pd(ba_f64x2, coeffs_f64x2));
            }
            x += 1;
        }

        for i in 0..ROWS_COUNT {
            _mm_storeu_pd(rg_buf.as_mut_ptr(), rg_sums[i]);
            _mm_storeu_pd(ba_buf.as_mut_ptr(), ba_sums[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = [
                rg_buf[0] as f32,
                rg_buf[1] as f32,
                ba_buf[0] as f32,
                ba_buf[1] as f32,
            ];
        }
    }
}
