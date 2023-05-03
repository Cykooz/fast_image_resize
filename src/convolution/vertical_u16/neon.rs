use std::arch::aarch64::*;
use std::mem::transmute;

use crate::convolution::{optimisations, Coefficients};
use crate::neon_utils;
use crate::pixels::PixelExt;
use crate::{ImageView, ImageViewMut};

pub(crate) fn vert_convolution<T: PixelExt<Component = u16>>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let precision = normalizer.precision();
    let initial = 1i64 << (precision - 1);
    let start_src_x = offset as usize * T::count_of_components();

    let mut tmp_dst = vec![0i64; dst_image.width().get() as usize * T::count_of_components()];
    let tmp_buf = tmp_dst.as_mut_slice();
    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        tmp_buf.fill(initial);
        unsafe {
            vert_convolution_into_one_row_i64(src_image, tmp_buf, start_src_x, coeffs_chunk);
            let dst_comp = T::components_mut(dst_row);
            macro_rules! call {
                ($imm8:expr) => {{
                    store_tmp_buf_into_dst_row::<$imm8>(tmp_buf, dst_comp, &normalizer);
                }};
            }
            constify_64_imm8!(precision as i64, call);
        }
    }
}

#[target_feature(enable = "neon")]
unsafe fn vert_convolution_into_one_row_i64<T: PixelExt<Component = u16>>(
    src_img: &ImageView<T>,
    dst_buf: &mut [i64],
    start_src_x: usize,
    coeffs_chunk: optimisations::CoefficientsI32Chunk,
) {
    let width = dst_buf.len();
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values;

    let zero_u16x8 = vdupq_n_u16(0);
    let zero_u16x4 = vdup_n_u16(0);

    for (s_row, &coeff) in src_img.iter_rows(y_start).zip(coeffs) {
        let components = T::components(s_row);
        let coeff_i32x2 = vdup_n_s32(coeff);

        let mut dst_x: usize = 0;
        let mut src_x = start_src_x;
        while dst_x < width.saturating_sub(31) {
            let source = neon_utils::load_u16x8x4(components, src_x);

            for s in [source.0, source.1, source.2, source.3] {
                let mut accum = neon_utils::load_i64x2x4(dst_buf, dst_x);
                let pix = vreinterpretq_s32_u16(vzip1q_u16(s, zero_u16x8));
                accum.0 = vmlal_s32(accum.0, vget_low_s32(pix), coeff_i32x2);
                accum.1 = vmlal_s32(accum.1, vget_high_s32(pix), coeff_i32x2);
                let pix = vreinterpretq_s32_u16(vzip2q_u16(s, zero_u16x8));
                accum.2 = vmlal_s32(accum.2, vget_low_s32(pix), coeff_i32x2);
                accum.3 = vmlal_s32(accum.3, vget_high_s32(pix), coeff_i32x2);
                neon_utils::store_i64x2x4(dst_buf, dst_x, accum);
                dst_x += 8;
                src_x += 8;
            }
        }

        if dst_x < width.saturating_sub(15) {
            let source = neon_utils::load_u16x8x2(components, src_x);

            for s in [source.0, source.1] {
                let mut accum = neon_utils::load_i64x2x4(dst_buf, dst_x);
                let pix = vreinterpretq_s32_u16(vzip1q_u16(s, zero_u16x8));
                accum.0 = vmlal_s32(accum.0, vget_low_s32(pix), coeff_i32x2);
                accum.1 = vmlal_s32(accum.1, vget_high_s32(pix), coeff_i32x2);
                let pix = vreinterpretq_s32_u16(vzip2q_u16(s, zero_u16x8));
                accum.2 = vmlal_s32(accum.2, vget_low_s32(pix), coeff_i32x2);
                accum.3 = vmlal_s32(accum.3, vget_high_s32(pix), coeff_i32x2);
                neon_utils::store_i64x2x4(dst_buf, dst_x, accum);
                dst_x += 8;
                src_x += 8;
            }
        }

        if dst_x < width.saturating_sub(7) {
            let s = neon_utils::load_u16x8(components, src_x);
            let mut accum = neon_utils::load_i64x2x4(dst_buf, dst_x);
            let pix = vreinterpretq_s32_u16(vzip1q_u16(s, zero_u16x8));
            accum.0 = vmlal_s32(accum.0, vget_low_s32(pix), coeff_i32x2);
            accum.1 = vmlal_s32(accum.1, vget_high_s32(pix), coeff_i32x2);
            let pix = vreinterpretq_s32_u16(vzip2q_u16(s, zero_u16x8));
            accum.2 = vmlal_s32(accum.2, vget_low_s32(pix), coeff_i32x2);
            accum.3 = vmlal_s32(accum.3, vget_high_s32(pix), coeff_i32x2);
            neon_utils::store_i64x2x4(dst_buf, dst_x, accum);
            dst_x += 8;
            src_x += 8;
        }

        if dst_x < width.saturating_sub(3) {
            let s = vcombine_u16(neon_utils::load_u16x4(components, src_x), zero_u16x4);
            let mut accum = neon_utils::load_i64x2x2(dst_buf, dst_x);
            let pix = vreinterpretq_s32_u16(vzip1q_u16(s, zero_u16x8));
            accum.0 = vmlal_s32(accum.0, vget_low_s32(pix), coeff_i32x2);
            accum.1 = vmlal_s32(accum.1, vget_high_s32(pix), coeff_i32x2);
            neon_utils::store_i64x2x2(dst_buf, dst_x, accum);
            dst_x += 4;
            src_x += 4;
        }

        let coeff = coeff as i64;
        let tmp_tail = dst_buf.iter_mut().skip(dst_x);
        let comp_tail = components.iter().skip(src_x);
        for (accum, &comp) in tmp_tail.zip(comp_tail) {
            *accum += coeff * comp as i64;
        }
    }
}

#[target_feature(enable = "neon")]
unsafe fn store_tmp_buf_into_dst_row<const IMM: i32>(
    mut src_buf: &[i64],
    dst_buf: &mut [u16],
    normalizer: &optimisations::Normalizer32,
) {
    let mut dst_chunks_8 = dst_buf.chunks_exact_mut(8);
    let src_chunks_8 = src_buf.chunks_exact(8);
    src_buf = src_chunks_8.remainder();
    for (dst_chunk, src_chunk) in dst_chunks_8.by_ref().zip(src_chunks_8) {
        let mut accum = neon_utils::load_i64x2x4(src_chunk, 0);
        accum.0 = vshrq_n_s64::<IMM>(accum.0);
        accum.1 = vshrq_n_s64::<IMM>(accum.1);
        accum.2 = vshrq_n_s64::<IMM>(accum.2);
        accum.3 = vshrq_n_s64::<IMM>(accum.3);
        let sss0_i32 = vcombine_s32(vqmovn_s64(accum.0), vqmovn_s64(accum.1));
        let sss1_i32 = vcombine_s32(vqmovn_s64(accum.2), vqmovn_s64(accum.3));
        let sss_u16 = vcombine_u16(vqmovun_s32(sss0_i32), vqmovun_s32(sss1_i32));
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut u128;
        vstrq_p128(dst_ptr, transmute(sss_u16));
    }

    let mut dst_chunks_4 = dst_chunks_8.into_remainder().chunks_exact_mut(4);
    let src_chunks_4 = src_buf.chunks_exact(4);
    src_buf = src_chunks_4.remainder();
    for (dst_chunk, src_chunk) in dst_chunks_4.by_ref().zip(src_chunks_4) {
        let mut accum = neon_utils::load_i64x2x2(src_chunk, 0);
        accum.0 = vshrq_n_s64::<IMM>(accum.0);
        accum.1 = vshrq_n_s64::<IMM>(accum.1);
        let sss_i32 = vcombine_s32(vqmovn_s64(accum.0), vqmovn_s64(accum.1));
        let sss_u16 = vcombine_u16(vqmovun_s32(sss_i32), vqmovun_s32(sss_i32));
        let res = vdupd_laneq_u64::<0>(vreinterpretq_u64_u16(sss_u16));
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut u64;
        dst_ptr.write_unaligned(res);
    }

    let mut dst_chunks_2 = dst_chunks_4.into_remainder().chunks_exact_mut(2);
    let src_chunks_2 = src_buf.chunks_exact(2);
    src_buf = src_chunks_2.remainder();
    for (dst_chunk, src_chunk) in dst_chunks_2.by_ref().zip(src_chunks_2) {
        let mut accum = neon_utils::load_i64x2(src_chunk, 0);
        accum = vshrq_n_s64::<IMM>(accum);
        let sss_i32 = vcombine_s32(vqmovn_s64(accum), vqmovn_s64(accum));
        let sss_u16 = vcombine_u16(vqmovun_s32(sss_i32), vqmovun_s32(sss_i32));
        let res = vdups_laneq_u32::<0>(vreinterpretq_u32_u16(sss_u16));
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut u32;
        dst_ptr.write_unaligned(res);
    }

    let dst_chunk = dst_chunks_2.into_remainder();
    for (dst, &src) in dst_chunk.iter_mut().zip(src_buf) {
        *dst = normalizer.clip(src);
    }
}
