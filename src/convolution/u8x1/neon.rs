use std::arch::aarch64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::neon_utils;
use crate::pixels::U8;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8>,
    dst_image: &mut ImageViewMut<U8>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let dst_height = dst_image.height().get();

    let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_image.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows(src_rows, dst_rows, &coefficients_chunks, &normalizer);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_row(
                src_image.get_row(yy + offset).unwrap(),
                dst_image.get_row_mut(yy).unwrap(),
                &coefficients_chunks,
                &normalizer,
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
    src_rows: [&[U8]; 4],
    dst_rows: [&mut &mut [U8]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let precision = normalizer.precision();
    let initial = vdupq_n_s32(1 << (precision - 3));
    let zero_u8x16 = vdupq_n_u8(0);
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss_a = [initial; 4];
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_16 = coeffs.chunks_exact(16);
        coeffs = coeffs_by_16.remainder();
        for k in coeffs_by_16 {
            let coeffs_i16x8x2 = neon_utils::load_i16x8x2(k, 0);
            let coeff0 = vget_low_s16(coeffs_i16x8x2.0);
            let coeff1 = vget_high_s16(coeffs_i16x8x2.0);
            let coeff2 = vget_low_s16(coeffs_i16x8x2.1);
            let coeff3 = vget_high_s16(coeffs_i16x8x2.1);

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

            x += 16;
        }

        let mut coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        if let Some(k) = coeffs_by_8.next() {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            let coeff0 = vget_low_s16(coeffs_i16x8);
            let coeff1 = vget_high_s16(coeffs_i16x8);

            for i in 0..4 {
                let source = neon_utils::load_u8x8(src_rows[i], x);
                let mut sss = sss_a[i];

                let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff0);
                let pix = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff1);

                sss_a[i] = sss;
            }
            x += 8;
        }

        let mut coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            for i in 0..4 {
                let source = neon_utils::load_u8x4(src_rows[i], x);
                sss_a[i] = conv_4_pixels(sss_a[i], coeffs_i16x4, source, zero_u8x8);
            }
            x += 4;
        }

        let mut coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        if let Some(k) = coeffs_by_2.next() {
            let coeffs_i16x4 = neon_utils::load_i16x2(k, 0);
            for i in 0..4 {
                let source = neon_utils::load_u8x2(src_rows[i], x);
                sss_a[i] = conv_4_pixels(sss_a[i], coeffs_i16x4, source, zero_u8x8);
            }
            x += 2;
        }

        if !coeffs.is_empty() {
            let coeffs_i16x4 = neon_utils::load_i16x1(coeffs, 0);
            for i in 0..4 {
                let source = neon_utils::load_u8x1(src_rows[i], x);
                sss_a[i] = conv_4_pixels(sss_a[i], coeffs_i16x4, source, zero_u8x8);
            }
        }

        for i in 0..4 {
            let sss = sss_a[i];
            let res_i32x2 = vadd_s32(vget_low_s32(sss), vget_high_s32(sss));
            let res = vget_lane_s32::<0>(res_i32x2) + vget_lane_s32::<1>(res_i32x2);
            dst_rows[i].get_unchecked_mut(dst_x).0 = normalizer.clip(res);
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
    src_row: &[U8],
    dst_row: &mut [U8],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    normalizer: &optimisations::Normalizer16,
) {
    let precision = normalizer.precision();
    let initial = vdupq_n_s32(1 << (precision - 3));
    let zero_u8x16 = vdupq_n_u8(0);
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = initial;
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_16 = coeffs.chunks_exact(16);
        coeffs = coeffs_by_16.remainder();

        for k in coeffs_by_16 {
            let coeffs_i16x8x2 = neon_utils::load_i16x8x2(k, 0);
            let source = neon_utils::load_u8x16(src_row, x);

            let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
            sss = vmlal_s16(
                sss,
                vget_low_s16(source_i16),
                vget_low_s16(coeffs_i16x8x2.0),
            );
            sss = vmlal_s16(
                sss,
                vget_high_s16(source_i16),
                vget_high_s16(coeffs_i16x8x2.0),
            );

            let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
            sss = vmlal_s16(
                sss,
                vget_low_s16(source_i16),
                vget_low_s16(coeffs_i16x8x2.1),
            );
            sss = vmlal_s16(
                sss,
                vget_high_s16(source_i16),
                vget_high_s16(coeffs_i16x8x2.1),
            );

            x += 16;
        }

        let mut coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        if let Some(k) = coeffs_by_8.next() {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            let source = neon_utils::load_u8x8(src_row, x);

            let source_i16 = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
            sss = vmlal_s16(sss, source_i16, vget_low_s16(coeffs_i16x8));
            let source_i16 = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
            sss = vmlal_s16(sss, source_i16, vget_high_s16(coeffs_i16x8));

            x += 8;
        }

        let mut coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            let source = neon_utils::load_u8x4(src_row, x);
            sss = conv_4_pixels(sss, coeffs_i16x4, source, zero_u8x8);
            x += 4;
        }

        let mut coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        if let Some(k) = coeffs_by_2.next() {
            let coeffs_i16x4 = neon_utils::load_i16x2(k, 0);
            let source = neon_utils::load_u8x2(src_row, x);
            sss = conv_4_pixels(sss, coeffs_i16x4, source, zero_u8x8);
            x += 2;
        }

        if !coeffs.is_empty() {
            let coeffs_i16x4 = neon_utils::load_i16x1(coeffs, 0);
            let source = neon_utils::load_u8x1(src_row, x);
            sss = conv_4_pixels(sss, coeffs_i16x4, source, zero_u8x8);
        }

        let res_i32x2 = vadd_s32(vget_low_s32(sss), vget_high_s32(sss));
        let res = vget_lane_s32::<0>(res_i32x2) + vget_lane_s32::<1>(res_i32x2);
        dst_row.get_unchecked_mut(dst_x).0 = normalizer.clip(res);
    }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn conv_4_pixels(
    sss: int32x4_t,
    coeffs_i16x4: int16x4_t,
    source: uint8x8_t,
    zero_u8x8: uint8x8_t,
) -> int32x4_t {
    let source_i16 = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
    vmlal_s16(sss, source_i16, coeffs_i16x4)
}
