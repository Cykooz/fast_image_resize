use core::arch::wasm32::*;

use crate::convolution::{Coefficients, CoefficientsChunk};
use crate::pixels::F32;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = F32>,
    dst_view: &mut impl ImageViewMut<Pixel = F32>,
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
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_rows<const ROWS_COUNT: usize>(
    src_rows: [&[F32]; ROWS_COUNT],
    dst_rows: [&mut [F32]; ROWS_COUNT],
    coefficients_chunks: &[CoefficientsChunk],
) {
    let mut ll_buf = [0f64; 2];

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sums = [f64x2_splat(0.); ROWS_COUNT];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff01_f64x2 = wasm32_utils::load_v128(k, 0);
            let coeff23_f64x2 = wasm32_utils::load_v128(k, 2);

            for i in 0..ROWS_COUNT {
                let mut sum = sums[i];
                let source = wasm32_utils::load_v128(src_rows[i], x);

                let pixel01_f64 = f64x2_promote_low_f32x4(source);
                sum = wasm32_utils::f64x2_mul_add(sum, pixel01_f64, coeff01_f64x2);

                let pixel23_f64 = wasm32_utils::f64x2_promote_high_f32x4(source);
                sum = wasm32_utils::f64x2_mul_add(sum, pixel23_f64, coeff23_f64x2);

                sums[i] = sum;
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let coeff01_f64x2 = wasm32_utils::load_v128(k, 0);

            for i in 0..ROWS_COUNT {
                let pixel0 = src_rows[i].get_unchecked(x).0;
                let pixel1 = src_rows[i].get_unchecked(x + 1).0;
                let pixel01_f64 = f64x2(pixel0 as f64, pixel1 as f64);
                sums[i] = wasm32_utils::f64x2_mul_add(sums[i], pixel01_f64, coeff01_f64x2);
            }
            x += 2;
        }

        if let Some(&k) = coeffs.first() {
            let coeff0_f64x2 = f64x2_splat(k);

            for i in 0..ROWS_COUNT {
                let pixel0 = src_rows[i].get_unchecked(x).0;
                let pixel0_f64 = f64x2(pixel0 as f64, 0.);
                sums[i] = wasm32_utils::f64x2_mul_add(sums[i], pixel0_f64, coeff0_f64x2);
            }
        }

        for i in 0..ROWS_COUNT {
            v128_store(ll_buf.as_mut_ptr() as *mut v128, sums[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = (ll_buf[0] + ll_buf[1]) as f32;
        }
    }
}
