use std::arch::aarch64::*;

use crate::convolution::optimisations::{CoefficientsI16Chunk, Normalizer16};
use crate::neon_utils;
use crate::pixels::U8x4;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U8x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x4>,
    offset: u32,
    normalizer: &Normalizer16,
) {
    let precision = normalizer.precision();

    macro_rules! call {
        ($imm8:expr) => {{
            horiz_convolution_p::<$imm8>(src_view, dst_view, offset, normalizer);
        }};
    }
    constify_imm8!(precision, call);
}

fn horiz_convolution_p<const PRECISION: i32>(
    src_view: &impl ImageView<Pixel = U8x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x4>,
    offset: u32,
    normalizer: &Normalizer16,
) {
    let coefficients_chunks = normalizer.chunks();
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows::<PRECISION>(src_rows, dst_rows, &coefficients_chunks);
        }
    }

    let yy = dst_height - dst_height % 4;
    let src_rows = src_view.iter_rows(yy + offset);
    let dst_rows = dst_view.iter_rows_mut(yy);
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            horiz_convolution_one_row::<PRECISION>(src_row, dst_row, &coefficients_chunks);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "neon")]
unsafe fn horiz_convolution_four_rows<const PRECISION: i32>(
    src_rows: [&[U8x4]; 4],
    dst_rows: [&mut [U8x4]; 4],
    coefficients_chunks: &[CoefficientsI16Chunk],
) {
    let initial = vdupq_n_s32(1 << (PRECISION - 1));
    let zero_u8x16 = vdupq_n_u8(0);
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;

        let mut sss_a = [initial; 4];

        let mut coeffs = coeffs_chunk.values();

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            let coeff0 = vdup_laneq_s16::<0>(coeffs_i16x8);
            let coeff1 = vdup_laneq_s16::<1>(coeffs_i16x8);
            let coeff2 = vdup_laneq_s16::<2>(coeffs_i16x8);
            let coeff3 = vdup_laneq_s16::<3>(coeffs_i16x8);
            let coeff4 = vdup_laneq_s16::<4>(coeffs_i16x8);
            let coeff5 = vdup_laneq_s16::<5>(coeffs_i16x8);
            let coeff6 = vdup_laneq_s16::<6>(coeffs_i16x8);
            let coeff7 = vdup_laneq_s16::<7>(coeffs_i16x8);

            for i in 0..4 {
                let source = neon_utils::load_u8x16(src_rows[i], x);
                let mut sss = sss_a[i];

                let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
                let pix = vget_low_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff0);
                let pix = vget_high_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff1);

                let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
                let pix = vget_low_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff2);
                let pix = vget_high_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff3);

                let source = neon_utils::load_u8x16(src_rows[i], x + 4);
                let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
                let pix = vget_low_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff4);
                let pix = vget_high_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff5);

                let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
                let pix = vget_low_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff6);
                let pix = vget_high_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff7);

                sss_a[i] = sss;
            }

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            let coeff0 = vdup_lane_s16::<0>(coeffs_i16x4);
            let coeff1 = vdup_lane_s16::<1>(coeffs_i16x4);
            let coeff2 = vdup_lane_s16::<2>(coeffs_i16x4);
            let coeff3 = vdup_lane_s16::<3>(coeffs_i16x4);

            for i in 0..4 {
                let source = neon_utils::load_u8x16(src_rows[i], x);
                let mut sss = sss_a[i];

                let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
                let pix = vget_low_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff0);
                let pix = vget_high_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff1);

                let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
                let pix = vget_low_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff2);
                let pix = vget_high_s16(source_i16);
                sss = vmlal_s16(sss, pix, coeff3);

                sss_a[i] = sss;
            }
            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let coeff0 = vdup_n_s16(k[0]);
            let coeff1 = vdup_n_s16(k[1]);

            for i in 0..4 {
                let source = neon_utils::load_u8x8(src_rows[i], x);
                let mut sss = sss_a[i];

                let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff0);
                let pix = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
                sss = vmlal_s16(sss, pix, coeff1);

                sss_a[i] = sss;
            }
            x += 2
        }

        if let Some(&k) = coeffs.first() {
            let coeff = vdup_n_s16(k);
            for i in 0..4 {
                let source = neon_utils::load_u8x4(src_rows[i], x);
                let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
                sss_a[i] = vmlal_s16(sss_a[i], pix, coeff);
            }
        }

        sss_a[0] = vshrq_n_s32::<PRECISION>(sss_a[0]);
        sss_a[1] = vshrq_n_s32::<PRECISION>(sss_a[1]);
        sss_a[2] = vshrq_n_s32::<PRECISION>(sss_a[2]);
        sss_a[3] = vshrq_n_s32::<PRECISION>(sss_a[3]);

        for i in 0..4 {
            let s = vqmovun_s16(vcombine_s16(vqmovn_s32(sss_a[i]), vdup_n_s16(0)));
            let s = vreinterpret_u32_u8(s);
            dst_rows[i].get_unchecked_mut(dst_x).0 = vget_lane_u32::<0>(s).to_le_bytes();
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "neon")]
unsafe fn horiz_convolution_one_row<const PRECISION: i32>(
    src_row: &[U8x4],
    dst_row: &mut [U8x4],
    coefficients_chunks: &[CoefficientsI16Chunk],
) {
    let initial = vdupq_n_s32(1 << (PRECISION - 1));
    let zero_u8x16 = vdupq_n_u8(0);
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = initial;
        let mut coeffs = coeffs_chunk.values();

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            let source = neon_utils::load_u8x16(src_row, x);

            let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
            let pix = vget_low_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<0>(coeffs_i16x8));
            let pix = vget_high_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<1>(coeffs_i16x8));

            let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
            let pix = vget_low_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<2>(coeffs_i16x8));
            let pix = vget_high_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<3>(coeffs_i16x8));

            let source = neon_utils::load_u8x16(src_row, x + 4);
            let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
            let pix = vget_low_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<4>(coeffs_i16x8));
            let pix = vget_high_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<5>(coeffs_i16x8));

            let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
            let pix = vget_low_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<6>(coeffs_i16x8));
            let pix = vget_high_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_laneq_s16::<7>(coeffs_i16x8));

            x += 8;
        }

        let coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            let source = neon_utils::load_u8x16(src_row, x);

            let source_i16 = vreinterpretq_s16_u8(vzip1q_u8(source, zero_u8x16));
            let pix = vget_low_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_lane_s16::<0>(coeffs_i16x4));
            let pix = vget_high_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_lane_s16::<1>(coeffs_i16x4));

            let source_i16 = vreinterpretq_s16_u8(vzip2q_u8(source, zero_u8x16));
            let pix = vget_low_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_lane_s16::<2>(coeffs_i16x4));
            let pix = vget_high_s16(source_i16);
            sss = vmlal_s16(sss, pix, vdup_lane_s16::<3>(coeffs_i16x4));

            x += 4;
        }

        let coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let source = neon_utils::load_u8x8(src_row, x);

            let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
            sss = vmlal_s16(sss, pix, vdup_n_s16(k[0]));
            let pix = vreinterpret_s16_u8(vzip2_u8(source, zero_u8x8));
            sss = vmlal_s16(sss, pix, vdup_n_s16(k[1]));

            x += 2
        }

        if let Some(&k) = coeffs.first() {
            let source = neon_utils::load_u8x4(src_row, x);
            let pix = vreinterpret_s16_u8(vzip1_u8(source, zero_u8x8));
            sss = vmlal_s16(sss, pix, vdup_n_s16(k));
        }

        sss = vshrq_n_s32::<PRECISION>(sss);

        let s = vqmovun_s16(vcombine_s16(vqmovn_s32(sss), vdup_n_s16(0)));
        let s = vreinterpret_u32_u8(s);
        dst_row.get_unchecked_mut(dst_x).0 = vget_lane_u32::<0>(s).to_le_bytes();
    }
}
