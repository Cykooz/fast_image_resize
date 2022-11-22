use std::arch::aarch64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::neon_utils;
use crate::pixels::U16x3;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U16x3>,
    dst_image: &mut ImageViewMut<U16x3>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let precision = normalizer.precision();
    let coefficients_chunks = normalizer.normalized_chunks();

    let src_iter = src_image.iter_rows(offset);
    let dst_iter = dst_image.iter_rows_mut();
    for (src_row, dst_row) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_row(src_row, dst_row, &coefficients_chunks, precision);
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
    src_row: &[U16x3],
    dst_row: &mut [U16x3],
    coefficients_chunks: &[optimisations::CoefficientsI32Chunk],
    precision: u8,
) {
    let initial = vdupq_n_s64(1i64 << (precision - 2));
    let zero_u16x8 = vdupq_n_u16(0);
    let zero_u16x4 = vdup_n_u16(0);

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = [initial; 3];
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i32x4x2 = neon_utils::load_i32x4x2(k, 0);
            let source = neon_utils::load_deintrel_u16x8x3(src_row, x);

            sss[0] = conv_8_comp(sss[0], source.0, coeffs_i32x4x2, zero_u16x8);
            sss[1] = conv_8_comp(sss[1], source.1, coeffs_i32x4x2, zero_u16x8);
            sss[2] = conv_8_comp(sss[2], source.2, coeffs_i32x4x2, zero_u16x8);

            x += 8;
        }

        let mut coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i32x4 = neon_utils::load_i32x4(k, 0);
            let source = neon_utils::load_deintrel_u16x4x3(src_row, x);

            sss[0] = conv_4_comp(sss[0], source.0, coeffs_i32x4, zero_u16x4);
            sss[1] = conv_4_comp(sss[1], source.1, coeffs_i32x4, zero_u16x4);
            sss[2] = conv_4_comp(sss[2], source.2, coeffs_i32x4, zero_u16x4);

            x += 4;
        }

        let mut coeffs_by_2 = coeffs.chunks_exact(2);
        coeffs = coeffs_by_2.remainder();
        if let Some(k) = coeffs_by_2.next() {
            let coeffs_i32x2 = neon_utils::load_i32x2(k, 0);
            let source = neon_utils::load_deintrel_u16x2x3(src_row, x);

            sss[0] = conv_2_comp(sss[0], source.0, coeffs_i32x2, zero_u16x4);
            sss[1] = conv_2_comp(sss[1], source.1, coeffs_i32x2, zero_u16x4);
            sss[2] = conv_2_comp(sss[2], source.2, coeffs_i32x2, zero_u16x4);

            x += 2;
        }

        if !coeffs.is_empty() {
            let coeffs_i32x2 = neon_utils::load_i32x1(coeffs, 0);
            let source = neon_utils::load_deintrel_u16x1x3(src_row, x);
            sss[0] = conv_2_comp(sss[0], source.0, coeffs_i32x2, zero_u16x4);
            sss[1] = conv_2_comp(sss[1], source.1, coeffs_i32x2, zero_u16x4);
            sss[2] = conv_2_comp(sss[2], source.2, coeffs_i32x2, zero_u16x4);
        }

        let mut sss_i64 = [
            vadd_s64(vget_low_s64(sss[0]), vget_high_s64(sss[0])),
            vadd_s64(vget_low_s64(sss[1]), vget_high_s64(sss[1])),
            vadd_s64(vget_low_s64(sss[2]), vget_high_s64(sss[2])),
        ];
        macro_rules! call {
            ($imm8:expr) => {{
                sss_i64[0] = vshr_n_s64::<$imm8>(sss_i64[0]);
                sss_i64[1] = vshr_n_s64::<$imm8>(sss_i64[1]);
                sss_i64[2] = vshr_n_s64::<$imm8>(sss_i64[2]);
            }};
        }
        constify_64_imm8!(precision, call);

        dst_row.get_unchecked_mut(dst_x).0 = [
            vqmovns_u32(vqmovund_s64(vdupd_lane_s64::<0>(sss_i64[0]))),
            vqmovns_u32(vqmovund_s64(vdupd_lane_s64::<0>(sss_i64[1]))),
            vqmovns_u32(vqmovund_s64(vdupd_lane_s64::<0>(sss_i64[2]))),
        ];
    }
}

#[inline(always)]
unsafe fn conv_8_comp(
    mut sss: int64x2_t,
    source: uint16x8_t,
    coeffs: int32x4x2_t,
    zero_u16x8: uint16x8_t,
) -> int64x2_t {
    let pix_i32 = vreinterpretq_s32_u16(vzip1q_u16(source, zero_u16x8));
    sss = vmlal_s32(sss, vget_low_s32(pix_i32), vget_low_s32(coeffs.0));
    sss = vmlal_s32(sss, vget_high_s32(pix_i32), vget_high_s32(coeffs.0));

    let pix_i32 = vreinterpretq_s32_u16(vzip2q_u16(source, zero_u16x8));
    sss = vmlal_s32(sss, vget_low_s32(pix_i32), vget_low_s32(coeffs.1));
    sss = vmlal_s32(sss, vget_high_s32(pix_i32), vget_high_s32(coeffs.1));

    sss
}

#[inline(always)]
unsafe fn conv_4_comp(
    mut sss: int64x2_t,
    source: uint16x4_t,
    coeffs: int32x4_t,
    zero_u16x4: uint16x4_t,
) -> int64x2_t {
    let pix_i32 = vreinterpret_s32_u16(vzip1_u16(source, zero_u16x4));
    sss = vmlal_s32(sss, pix_i32, vget_low_s32(coeffs));
    let pix_i32 = vreinterpret_s32_u16(vzip2_u16(source, zero_u16x4));
    sss = vmlal_s32(sss, pix_i32, vget_high_s32(coeffs));
    sss
}

#[inline(always)]
unsafe fn conv_2_comp(
    mut sss: int64x2_t,
    source: uint16x4_t,
    coeffs: int32x2_t,
    zero_u16x4: uint16x4_t,
) -> int64x2_t {
    let pix_i32 = vreinterpret_s32_u16(vzip1_u16(source, zero_u16x4));
    sss = vmlal_s32(sss, pix_i32, coeffs);
    sss
}
