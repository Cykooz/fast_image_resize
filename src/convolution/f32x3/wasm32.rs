use core::arch::wasm32::*;

use crate::convolution::{Coefficients, CoefficientsChunk};
use crate::pixels::{F32x3, InnerPixel};
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = F32x3>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x3>,
    offset: u32,
    coeffs: &Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_2_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_2_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_rows(src_rows, dst_rows, &coefficients_chunks);
        }
    }

    let yy = dst_height - dst_height % 2;
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
    src_rows: [&[F32x3]; ROWS_COUNT],
    dst_rows: [&mut [F32x3]; ROWS_COUNT],
    coefficients_chunks: &[CoefficientsChunk],
) {
    /*
        |R0 G0 B0| |R1|
        |00 01 02| |03|

        |G1 B1| |R2 G2|
        |00 01| |02 03|

        |B2| |R3 G3 B3|
        |00| |01 02 03|
    */

    let mut rg_buf = [0f64; 2];
    let mut br_buf = [0f64; 2];
    let mut gb_buf = [0f64; 2];

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rg_sums = [f64x2_splat(0.); ROWS_COUNT];
        let mut br_sums = [f64x2_splat(0.); ROWS_COUNT];
        let mut gb_sums = [f64x2_splat(0.); ROWS_COUNT];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeff00_f64x2 = f64x2_splat(k[0]);
            let coeff01_f64x2 = f64x2(k[0], k[1]);
            let coeff11_f64x2 = f64x2_splat(k[1]);
            let coeff22_f64x2 = f64x2_splat(k[2]);
            let coeff23_f64x2 = f64x2(k[2], k[3]);
            let coeff33_f64x2 = f64x2_splat(k[3]);

            for i in 0..ROWS_COUNT {
                let c = x * 3;
                let components = F32x3::components(src_rows[i]);

                let rgb0r1 = wasm32_utils::load_v128(components, c);
                let rg0_f64x2 = f64x2_promote_low_f32x4(rgb0r1);
                rg_sums[i] = wasm32_utils::f64x2_mul_add(rg_sums[i], rg0_f64x2, coeff00_f64x2);
                let b0r1_f64x2 = wasm32_utils::f64x2_promote_high_f32x4(rgb0r1);
                br_sums[i] = wasm32_utils::f64x2_mul_add(br_sums[i], b0r1_f64x2, coeff01_f64x2);

                let gb1rg2 = wasm32_utils::load_v128(components, c + 4);
                let gb1_f64x2 = f64x2_promote_low_f32x4(gb1rg2);
                gb_sums[i] = wasm32_utils::f64x2_mul_add(gb_sums[i], gb1_f64x2, coeff11_f64x2);
                let rg2_f64x2 = wasm32_utils::f64x2_promote_high_f32x4(gb1rg2);
                rg_sums[i] = wasm32_utils::f64x2_mul_add(rg_sums[i], rg2_f64x2, coeff22_f64x2);

                let b2rgb3 = wasm32_utils::load_v128(components, c + 8);
                let b2r3_f64x2 = f64x2_promote_low_f32x4(b2rgb3);
                br_sums[i] = wasm32_utils::f64x2_mul_add(br_sums[i], b2r3_f64x2, coeff23_f64x2);
                let gb3_f64x2 = wasm32_utils::f64x2_promote_high_f32x4(b2rgb3);
                gb_sums[i] = wasm32_utils::f64x2_mul_add(gb_sums[i], gb3_f64x2, coeff33_f64x2);
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let coeff00_f64x2 = f64x2_splat(k[0]);
            let coeff01_f64x2 = f64x2(k[0], k[1]);
            let coeff11_f64x2 = f64x2_splat(k[1]);

            for i in 0..ROWS_COUNT {
                let c = x * 3;
                let components = F32x3::components(src_rows[i]);

                let rgb0r1 = wasm32_utils::load_v128(components, c);
                let rg0_f64x2 = f64x2_promote_low_f32x4(rgb0r1);
                rg_sums[i] = wasm32_utils::f64x2_mul_add(rg_sums[i], rg0_f64x2, coeff00_f64x2);
                let b0r1_f64x2 = wasm32_utils::f64x2_promote_high_f32x4(rgb0r1);
                br_sums[i] = wasm32_utils::f64x2_mul_add(br_sums[i], b0r1_f64x2, coeff01_f64x2);

                let g1 = *components.get_unchecked(c + 4);
                let b1 = *components.get_unchecked(c + 5);
                let gb1_f64x2 = f64x2(g1 as f64, b1 as f64);
                gb_sums[i] = wasm32_utils::f64x2_mul_add(gb_sums[i], gb1_f64x2, coeff11_f64x2);
            }
            x += 2;
        }

        for &k in coeffs {
            let coeff00_f64x2 = f64x2_splat(k);
            let coeff0x_f64x2 = f64x2(k, 0.);

            for i in 0..ROWS_COUNT {
                let pixel = src_rows[i].get_unchecked(x);
                let rgb0x = f32x4(pixel.0[0], pixel.0[1], pixel.0[2], 0.);

                let rg0_f64x2 = f64x2_promote_low_f32x4(rgb0x);
                rg_sums[i] = wasm32_utils::f64x2_mul_add(rg_sums[i], rg0_f64x2, coeff00_f64x2);

                let b0x_f64x2 = wasm32_utils::f64x2_promote_high_f32x4(rgb0x);
                br_sums[i] = wasm32_utils::f64x2_mul_add(br_sums[i], b0x_f64x2, coeff0x_f64x2);
            }
            x += 1;
        }

        for i in 0..ROWS_COUNT {
            v128_store(rg_buf.as_mut_ptr() as *mut v128, rg_sums[i]);
            v128_store(br_buf.as_mut_ptr() as *mut v128, br_sums[i]);
            v128_store(gb_buf.as_mut_ptr() as *mut v128, gb_sums[i]);
            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = [
                (rg_buf[0] + br_buf[1]) as f32,
                (rg_buf[1] + gb_buf[0]) as f32,
                (br_buf[0] + gb_buf[1]) as f32,
            ];
        }
    }
}
