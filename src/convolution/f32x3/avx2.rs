use std::arch::x86_64::*;

use crate::convolution::{Coefficients, CoefficientsChunk};
use crate::pixels::{F32x3, InnerPixel};
use crate::{simd_utils, ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = F32x3>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x3>,
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
    src_rows: [&[F32x3]; ROWS_COUNT],
    dst_rows: [&mut [F32x3]; ROWS_COUNT],
    coefficients_chunks: &[CoefficientsChunk],
) {
    /*
        |R0 G0 B0| |R1 G1 B1| |R2 G2|
        |00 01 02| |03 04 05| |06 07|

        |B2| |R3 G3 B3| |R4 G4 B4| |R5|
        |00| |01 02 03| |04 05 06| |07|

        |G5 B5| |R6 G6 B6| |R7 G7 B7|
        |00 01| |02 03 04| |05 06 07|
    */

    let mut rg_buf = [0f64; 2];
    let mut br_buf = [0f64; 2];
    let mut gb_buf = [0f64; 2];

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut rgbr_sums = [_mm256_set1_pd(0.); ROWS_COUNT];
        let mut gbrg_sums = [_mm256_set1_pd(0.); ROWS_COUNT];
        let mut brgb_sums = [_mm256_set1_pd(0.); ROWS_COUNT];

        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeff0001_f64x4 = _mm256_set_pd(k[1], k[0], k[0], k[0]);
            let coeff1122_f64x4 = _mm256_set_pd(k[2], k[2], k[1], k[1]);
            let coeff2333_f64x4 = _mm256_set_pd(k[3], k[3], k[3], k[2]);
            let coeff4445_f64x4 = _mm256_set_pd(k[5], k[4], k[4], k[4]);
            let coeff5566_f64x4 = _mm256_set_pd(k[6], k[6], k[5], k[5]);
            let coeff6777_f64x4 = _mm256_set_pd(k[7], k[7], k[7], k[6]);

            for i in 0..ROWS_COUNT {
                let c = x * 3;
                let components = F32x3::components(src_rows[i]);

                let rgb0rgb1rg2 = simd_utils::loadu_ps256(components, c);
                let rgb0r1_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<0>(rgb0rgb1rg2));
                rgbr_sums[i] =
                    _mm256_add_pd(rgbr_sums[i], _mm256_mul_pd(rgb0r1_f64x4, coeff0001_f64x4));
                let gb1rg2_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<1>(rgb0rgb1rg2));
                gbrg_sums[i] =
                    _mm256_add_pd(gbrg_sums[i], _mm256_mul_pd(gb1rg2_f64x4, coeff1122_f64x4));

                let b2rgb3rgb4r5 = simd_utils::loadu_ps256(components, c + 8);
                let b2rgb3_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<0>(b2rgb3rgb4r5));
                brgb_sums[i] =
                    _mm256_add_pd(brgb_sums[i], _mm256_mul_pd(b2rgb3_f64x4, coeff2333_f64x4));
                let rgb4r5_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<1>(b2rgb3rgb4r5));
                rgbr_sums[i] =
                    _mm256_add_pd(rgbr_sums[i], _mm256_mul_pd(rgb4r5_f64x4, coeff4445_f64x4));

                let gb5rgb6rgb7 = simd_utils::loadu_ps256(components, c + 16);
                let gb5rg6_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<0>(gb5rgb6rgb7));
                gbrg_sums[i] =
                    _mm256_add_pd(gbrg_sums[i], _mm256_mul_pd(gb5rg6_f64x4, coeff5566_f64x4));
                let b6rgb7_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<1>(gb5rgb6rgb7));
                brgb_sums[i] =
                    _mm256_add_pd(brgb_sums[i], _mm256_mul_pd(b6rgb7_f64x4, coeff6777_f64x4));
            }
            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        for k in coeffs_by_4 {
            let coeff0001_f64x4 = _mm256_set_pd(k[1], k[0], k[0], k[0]);
            let coeff1122_f64x4 = _mm256_set_pd(k[2], k[2], k[1], k[1]);
            let coeff2333_f64x4 = _mm256_set_pd(k[3], k[3], k[3], k[2]);

            for i in 0..ROWS_COUNT {
                let c = x * 3;
                let components = F32x3::components(src_rows[i]);

                let rgb0rgb1rg2 = simd_utils::loadu_ps256(components, c);
                let rgb0r1_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<0>(rgb0rgb1rg2));
                rgbr_sums[i] =
                    _mm256_add_pd(rgbr_sums[i], _mm256_mul_pd(rgb0r1_f64x4, coeff0001_f64x4));
                let gb1rg2_f64x4 = _mm256_cvtps_pd(_mm256_extractf128_ps::<1>(rgb0rgb1rg2));
                gbrg_sums[i] =
                    _mm256_add_pd(gbrg_sums[i], _mm256_mul_pd(gb1rg2_f64x4, coeff1122_f64x4));

                let b2rgb3 = simd_utils::loadu_ps(components, c + 8);
                let b2rgb3_f64x4 = _mm256_cvtps_pd(b2rgb3);
                brgb_sums[i] =
                    _mm256_add_pd(brgb_sums[i], _mm256_mul_pd(b2rgb3_f64x4, coeff2333_f64x4));
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        for k in coeffs_by_2 {
            let coeff0001_f64x4 = _mm256_set_pd(k[1], k[0], k[0], k[0]);
            let coeff11xx_f64x4 = _mm256_set_pd(0., 0., k[1], k[1]);

            for i in 0..ROWS_COUNT {
                let c = x * 3;
                let components = F32x3::components(src_rows[i]);

                let rgb0r1 = simd_utils::loadu_ps(components, c);
                let rgb0r1_f64x4 = _mm256_cvtps_pd(rgb0r1);
                rgbr_sums[i] =
                    _mm256_add_pd(rgbr_sums[i], _mm256_mul_pd(rgb0r1_f64x4, coeff0001_f64x4));

                let g1 = *components.get_unchecked(c + 4);
                let b1 = *components.get_unchecked(c + 5);
                let gb1xx = _mm_set_ps(0., 0., b1, g1);
                let gb1xx_f64x4 = _mm256_cvtps_pd(gb1xx);
                gbrg_sums[i] =
                    _mm256_add_pd(gbrg_sums[i], _mm256_mul_pd(gb1xx_f64x4, coeff11xx_f64x4));
            }
            x += 2;
        }

        for &k in coeffs {
            let coeff0000_f64x2 = _mm256_set1_pd(k);

            for i in 0..ROWS_COUNT {
                let pixel = src_rows[i].get_unchecked(x);
                let rgb0x = _mm_set_ps(0., pixel.0[2], pixel.0[1], pixel.0[0]);
                let rgb0x_f64x4 = _mm256_cvtps_pd(rgb0x);
                rgbr_sums[i] =
                    _mm256_add_pd(rgbr_sums[i], _mm256_mul_pd(rgb0x_f64x4, coeff0000_f64x2));
            }
            x += 1;
        }

        for i in 0..ROWS_COUNT {
            let rg0_f64x2 = _mm256_extractf128_pd::<0>(rgbr_sums[i]);
            let rg1_f64x2 = _mm256_extractf128_pd::<1>(gbrg_sums[i]);
            let rg_f64x2 = _mm_add_pd(rg0_f64x2, rg1_f64x2);
            _mm_storeu_pd(rg_buf.as_mut_ptr(), rg_f64x2);

            let br0_f64x2 = _mm256_extractf128_pd::<1>(rgbr_sums[i]);
            let br1_f64x2 = _mm256_extractf128_pd::<0>(brgb_sums[i]);
            let br_f64x2 = _mm_add_pd(br0_f64x2, br1_f64x2);
            _mm_storeu_pd(br_buf.as_mut_ptr(), br_f64x2);

            let gb0_f64x2 = _mm256_extractf128_pd::<0>(gbrg_sums[i]);
            let gb1_f64x2 = _mm256_extractf128_pd::<1>(brgb_sums[i]);
            let gb_f64x2 = _mm_add_pd(gb0_f64x2, gb1_f64x2);
            _mm_storeu_pd(gb_buf.as_mut_ptr(), gb_f64x2);

            let dst_pixel = dst_rows[i].get_unchecked_mut(dst_x);
            dst_pixel.0 = [
                (rg_buf[0] + br_buf[1]) as f32,
                (rg_buf[1] + gb_buf[0]) as f32,
                (br_buf[0] + gb_buf[1]) as f32,
            ];
        }
    }
}
