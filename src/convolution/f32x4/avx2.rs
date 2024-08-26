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
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_rows<const ROWS_COUNT: usize>(
    src_rows: [&[F32x4]; ROWS_COUNT],
    dst_rows: [&mut [F32x4]; ROWS_COUNT],
    coefficients_chunks: &[CoefficientsChunk],
) {
    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rgba_sums = [_mm256_set1_pd(0.); ROWS_COUNT];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let coeff0_f64x4 = _mm256_set1_pd(k[0]);
            let coeff1_f64x4 = _mm256_set1_pd(k[1]);

            for r in 0..ROWS_COUNT {
                let pixel01 = simd_utils::loadu_ps256(src_rows[r], x);

                let pixel0_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<0>(pixel01));
                rgba_sums[r] =
                    _mm256_add_pd(rgba_sums[r], _mm256_mul_pd(pixel0_f64x4, coeff0_f64x4));

                let pixels1_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<1>(pixel01));
                rgba_sums[r] =
                    _mm256_add_pd(rgba_sums[r], _mm256_mul_pd(pixels1_f64x4, coeff1_f64x4));
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_f64x4 = _mm256_set1_pd(k);

            for r in 0..ROWS_COUNT {
                let pixel0 = simd_utils::loadu_ps(src_rows[r], x);

                let pixel0_f64x4 = _mm256_cvtps_pd(pixel0);
                rgba_sums[r] =
                    _mm256_add_pd(rgba_sums[r], _mm256_mul_pd(pixel0_f64x4, coeff0_f64x4));
            }
        }

        for r in 0..ROWS_COUNT {
            let dst_pixel = dst_rows[r].get_unchecked_mut(dst_x);
            let rgba_f32x4 = _mm256_cvtpd_ps(rgba_sums[r]);
            _mm_storeu_ps(dst_pixel.0.as_mut_ptr(), rgba_f32x4);
        }
    }
}
