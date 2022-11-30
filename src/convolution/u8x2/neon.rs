use std::arch::aarch64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::neon_utils;
use crate::pixels::U8x2;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8x2>,
    dst_image: &mut ImageViewMut<U8x2>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let precision = normalizer.precision();
    let coefficients_chunks = normalizer.normalized_chunks();
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows(src_rows, dst_rows, &coefficients_chunks, precision);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_row(
                src_image.get_row(yy + offset).unwrap(),
                dst_image.get_row_mut(yy).unwrap(),
                &coefficients_chunks,
                precision,
            );
        }
        yy += 1;
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "neon")]
unsafe fn horiz_convolution_four_rows(
    src_rows: [&[U8x2]; 4],
    dst_rows: [&mut &mut [U8x2]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    let initial = vdupq_n_s32(1 << (precision - 2));
    let zero_u8x16 = vdupq_n_u8(0);
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss_a = [initial; 4];
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            let coeff01 = vzip1q_s16(coeffs_i16x8, coeffs_i16x8);
            let coeff23 = vzip2q_s16(coeffs_i16x8, coeffs_i16x8);
            let coeff0 = vget_low_s16(coeff01);
            let coeff1 = vget_high_s16(coeff01);
            let coeff2 = vget_low_s16(coeff23);
            let coeff3 = vget_high_s16(coeff23);

            for i in 0..4 {
                let source = neon_utils::load_u8x16(src_rows[i], x);
                let mut sss = sss_a[i];

                let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
                sss = vmlal_s16(sss, vget_low_s16(source_i16), coeff0);
                sss = vmlal_s16(sss, vget_high_s16(source_i16), coeff1);

                let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
                sss = vmlal_s16(sss, vget_low_s16(source_i16), coeff2);
                sss = vmlal_s16(sss, vget_high_s16(source_i16), coeff3);

                sss_a[i] = sss;
            }

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            let coeff0 = vzip1_s16(coeffs_i16x4, coeffs_i16x4);
            let coeff1 = vzip2_s16(coeffs_i16x4, coeffs_i16x4);

            for i in 0..4 {
                let source = neon_utils::load_u8x8(src_rows[i], x);
                let mut sss = sss_a[i];

                let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff0);
                let pix = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff1);

                sss_a[i] = sss;
            }
            x += 4;
        }

        if !coeffs.is_empty() {
            let mut four_coeffs = [0i16; 4];
            four_coeffs
                .iter_mut()
                .zip(coeffs)
                .for_each(|(d, s)| *d = *s);
            let coeffs_i16x4 = neon_utils::load_i16x4(&four_coeffs, 0);
            let coeff0 = vzip1_s16(coeffs_i16x4, coeffs_i16x4);
            let coeff1 = vzip2_s16(coeffs_i16x4, coeffs_i16x4);

            let mut four_pixels = [U8x2::new(0); 4];

            for i in 0..4 {
                four_pixels
                    .iter_mut()
                    .zip(src_rows[i].get_unchecked(x..))
                    .for_each(|(d, s)| *d = *s);
                let source = neon_utils::load_u8x8(&four_pixels, 0);
                let mut sss = sss_a[i];

                let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff0);
                let pix = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff1);

                sss_a[i] = sss;
            }
        }

        let mut res_i32x2x4 = sss_a.map(|sss| vadd_s32(vget_low_s32(sss), vget_high_s32(sss)));

        macro_rules! call {
            ($imm8:expr) => {{
                res_i32x2x4[0] = vshr_n_s32::<$imm8>(res_i32x2x4[0]);
                res_i32x2x4[1] = vshr_n_s32::<$imm8>(res_i32x2x4[1]);
                res_i32x2x4[2] = vshr_n_s32::<$imm8>(res_i32x2x4[2]);
                res_i32x2x4[3] = vshr_n_s32::<$imm8>(res_i32x2x4[3]);
            }};
        }
        constify_imm8!(precision, call);

        for i in 0..4 {
            let sss = vcombine_s32(res_i32x2x4[i], vreinterpret_s32_u8(zero_u8x8));
            let s = vreinterpret_u16_u8(vqmovun_s16(vcombine_s16(
                vqmovn_s32(sss),
                vreinterpret_s16_u8(zero_u8x8),
            )));
            dst_rows[i].get_unchecked_mut(dst_x).0 = vget_lane_u16::<0>(s);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "neon")]
unsafe fn horiz_convolution_row(
    src_row: &[U8x2],
    dst_row: &mut [U8x2],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    let initial = vdupq_n_s32(1 << (precision - 2));
    let zero_u8x16 = vdupq_n_u8(0);
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = initial;
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            let coeff01 = vzip1q_s16(coeffs_i16x8, coeffs_i16x8);
            let coeff23 = vzip2q_s16(coeffs_i16x8, coeffs_i16x8);
            let source = neon_utils::load_u8x16(src_row, x);

            let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
            sss = vmlal_s16(sss, vget_low_s16(source_i16), vget_low_s16(coeff01));
            sss = vmlal_s16(sss, vget_high_s16(source_i16), vget_high_s16(coeff01));

            let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
            sss = vmlal_s16(sss, vget_low_s16(source_i16), vget_low_s16(coeff23));
            sss = vmlal_s16(sss, vget_high_s16(source_i16), vget_high_s16(coeff23));

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            sss = conv_four_pixels(sss, k, src_row, x, zero_u8x8);
            x += 4;
        }

        if !coeffs.is_empty() {
            let mut four_coeffs = [0i16; 4];
            four_coeffs
                .iter_mut()
                .zip(coeffs)
                .for_each(|(d, s)| *d = *s);

            let mut four_pixels = [U8x2::new(0); 4];
            four_pixels
                .iter_mut()
                .zip(src_row.get_unchecked(x..))
                .for_each(|(d, s)| *d = *s);

            sss = conv_four_pixels(sss, &four_coeffs, &four_pixels, 0, zero_u8x8);
        }

        let mut res_i32x2 = vadd_s32(vget_low_s32(sss), vget_high_s32(sss));

        macro_rules! call {
            ($imm8:expr) => {{
                res_i32x2 = vshr_n_s32::<$imm8>(res_i32x2);
            }};
        }
        constify_imm8!(precision, call);

        let sss = vcombine_s32(res_i32x2, vreinterpret_s32_u8(zero_u8x8));
        let s = vreinterpret_u16_u8(vqmovun_s16(vcombine_s16(
            vqmovn_s32(sss),
            vreinterpret_s16_u8(zero_u8x8),
        )));
        dst_row.get_unchecked_mut(dst_x).0 = vget_lane_u16::<0>(s);
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn conv_four_pixels(
    mut sss: int32x4_t,
    coeffs: &[i16],
    src_row: &[U8x2],
    x: usize,
    zero_u8x8: uint8x8_t,
) -> int32x4_t {
    let coeffs_i16x4 = neon_utils::load_i16x4(coeffs, 0);
    let coeff0 = vzip1_s16(coeffs_i16x4, coeffs_i16x4);
    let coeff1 = vzip2_s16(coeffs_i16x4, coeffs_i16x4);
    let source = neon_utils::load_u8x8(src_row, x);

    let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
    sss = vmlal_s16(sss, pix, coeff0);
    let pix = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
    sss = vmlal_s16(sss, pix, coeff1);

    sss
}
