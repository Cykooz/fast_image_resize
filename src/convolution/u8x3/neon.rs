use std::arch::aarch64::*;

use crate::convolution::{optimisations, Coefficients};
use crate::neon_utils;
use crate::pixels::U8x3;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8x3>,
    dst_image: &mut ImageViewMut<U8x3>,
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
    src_rows: [&[U8x3]; 4],
    dst_rows: [&mut &mut [U8x3]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    let initial = vdupq_n_s32(1 << (precision - 1));
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss_a = [initial; 4];
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();
        for k in coeffs_by_8 {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            for i in 0..4 {
                sss_a[i] = conv_8_pixels(sss_a[i], coeffs_i16x8, src_rows[i], x, zero_u8x8);
            }
            x += 8;
        }

        let mut coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            for i in 0..4 {
                sss_a[i] = conv_4_pixels(sss_a[i], coeffs_i16x4, src_rows[i], x, zero_u8x8);
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

            let mut four_pixels = [U8x3::new([0, 0, 0]); 4];

            for i in 0..4 {
                four_pixels
                    .iter_mut()
                    .zip(src_rows[i].get_unchecked(x..))
                    .for_each(|(d, s)| *d = *s);
                sss_a[i] = conv_4_pixels(sss_a[i], coeffs_i16x4, &four_pixels, 0, zero_u8x8);
            }
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss_a[0] = vshrq_n_s32::<$imm8>(sss_a[0]);
                sss_a[1] = vshrq_n_s32::<$imm8>(sss_a[1]);
                sss_a[2] = vshrq_n_s32::<$imm8>(sss_a[2]);
                sss_a[3] = vshrq_n_s32::<$imm8>(sss_a[3]);
            }};
        }
        constify_imm8!(precision, call);

        for i in 0..4 {
            store_pixel(sss_a[i], dst_rows[i], dst_x, zero_u8x8);
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
    src_row: &[U8x3],
    dst_row: &mut [U8x3],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    let initial = vdupq_n_s32(1 << (precision - 1));
    let zero_u8x8 = vdup_n_u8(0);

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = initial;
        let mut coeffs = coeffs_chunk.values;

        let coeffs_by_8 = coeffs.chunks_exact(8);
        coeffs = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let coeffs_i16x8 = neon_utils::load_i16x8(k, 0);
            sss = conv_8_pixels(sss, coeffs_i16x8, src_row, x, zero_u8x8);
            x += 8;
        }

        let mut coeffs_by_4 = coeffs.chunks_exact(4);
        coeffs = coeffs_by_4.remainder();
        if let Some(k) = coeffs_by_4.next() {
            let coeffs_i16x4 = neon_utils::load_i16x4(k, 0);
            sss = conv_4_pixels(sss, coeffs_i16x4, src_row, x, zero_u8x8);
            x += 4;
        }

        if !coeffs.is_empty() {
            let mut four_coeffs = [0i16; 4];
            four_coeffs
                .iter_mut()
                .zip(coeffs)
                .for_each(|(d, s)| *d = *s);
            let coeffs_i16x4 = neon_utils::load_i16x4(&four_coeffs, 0);

            let mut four_pixels = [U8x3::new([0, 0, 0]); 4];
            four_pixels
                .iter_mut()
                .zip(src_row.get_unchecked(x..))
                .for_each(|(d, s)| *d = *s);

            sss = conv_4_pixels(sss, coeffs_i16x4, &four_pixels, 0, zero_u8x8);
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss = vshrq_n_s32::<$imm8>(sss);
            }};
        }
        constify_imm8!(precision, call);

        store_pixel(sss, dst_row, dst_x, zero_u8x8);
    }
}

#[inline]
unsafe fn store_pixel(sss: int32x4_t, dst_row: &mut [U8x3], dst_x: usize, zero_u8x8: uint8x8_t) {
    let res_i16x4 = vmovn_s32(sss);
    let res_u8x8 = vqmovun_s16(vcombine_s16(res_i16x4, vreinterpret_s16_u8(zero_u8x8)));
    let res_u32 = vget_lane_u32::<0>(vreinterpret_u32_u8(res_u8x8));
    let rgbx = res_u32.to_le_bytes();
    dst_row.get_unchecked_mut(dst_x).0 = [rgbx[0], rgbx[1], rgbx[2]];
}

#[inline]
unsafe fn conv_8_pixels(
    mut sss: int32x4_t,
    coeffs_i16x8: int16x8_t,
    src_row: &[U8x3],
    x: usize,
    zero_u8x8: uint8x8_t,
) -> int32x4_t {
    let source = neon_utils::load_u8x8x3(src_row, x);

    // pixel 0
    let pix_i16x4 = vreinterpret_s16_u8(vzip1_u8(source.0, zero_u8x8));
    let coeff = vdup_laneq_s16::<0>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 1
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source.0,
        vcreate_u8(u64::from_le_bytes([3, 255, 4, 255, 5, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<1>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 2
    let pix_i16x4 = vreinterpret_s16_u8(vtbl2_u8(
        uint8x8x2_t(source.0, source.1),
        vcreate_u8(u64::from_le_bytes([6, 255, 7, 255, 8, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<2>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 3
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source.1,
        vcreate_u8(u64::from_le_bytes([1, 255, 2, 255, 3, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<3>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 4
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source.1,
        vcreate_u8(u64::from_le_bytes([4, 255, 5, 255, 6, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<4>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 5
    let pix_i16x4 = vreinterpret_s16_u8(vtbl2_u8(
        uint8x8x2_t(source.1, source.2),
        vcreate_u8(u64::from_le_bytes([7, 255, 8, 255, 9, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<5>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 6
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source.2,
        vcreate_u8(u64::from_le_bytes([2, 255, 3, 255, 4, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<6>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 7
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source.2,
        vcreate_u8(u64::from_le_bytes([5, 255, 6, 255, 7, 255, 255, 255])),
    ));
    let coeff = vdup_laneq_s16::<7>(coeffs_i16x8);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    sss
}

#[inline]
unsafe fn conv_4_pixels(
    mut sss: int32x4_t,
    coeffs_i16x4: int16x4_t,
    src_row: &[U8x3],
    x: usize,
    zero_u8x8: uint8x8_t,
) -> int32x4_t {
    // |R0 G0 B0 R1 G1 B1 R2 G2|
    let source0 = neon_utils::load_u8x8(src_row, x);
    // |G1 B1 R2 G2 B2 R3 G3 B3|
    let source1 = vld1_u8((src_row.get_unchecked(x..).as_ptr() as *const u8).add(4));

    // pixel 0
    let pix_i16x4 = vreinterpret_s16_u8(vzip1_u8(source0, zero_u8x8));
    let coeff = vdup_lane_s16::<0>(coeffs_i16x4);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 1
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source0,
        vcreate_u8(u64::from_le_bytes([3, 255, 4, 255, 5, 255, 255, 255])),
    ));
    let coeff = vdup_lane_s16::<1>(coeffs_i16x4);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 2
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source1,
        vcreate_u8(u64::from_le_bytes([2, 255, 3, 255, 4, 255, 255, 255])),
    ));
    let coeff = vdup_lane_s16::<2>(coeffs_i16x4);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    // pixel 3
    let pix_i16x4 = vreinterpret_s16_u8(vtbl1_u8(
        source1,
        vcreate_u8(u64::from_le_bytes([5, 255, 6, 255, 7, 255, 255, 255])),
    ));
    let coeff = vdup_lane_s16::<3>(coeffs_i16x4);
    sss = vmlal_s16(sss, pix_i16x4, coeff);

    sss
}
