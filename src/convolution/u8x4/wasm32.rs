use std::arch::wasm32::*;
use std::intrinsics::transmute;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U8x4;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

// This code is based on C-implementation from Pillow-SIMD package for Python
// https://github.com/uploadcare/pillow-simd

#[inline]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8x4>,
    dst_image: &mut ImageViewMut<U8x4>,
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
            horiz_convolution_8u4x(src_rows, dst_rows, &coefficients_chunks, precision);
        }
    }

    let mut yy = dst_height - dst_height % 4;
    while yy < dst_height {
        unsafe {
            horiz_convolution_8u(
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
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_8u4x(
    src_rows: [&[U8x4]; 4],
    dst_rows: [&mut &mut [U8x4]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    let initial = i32x4_splat(1 << (precision - 1));
    const MASK_LO: v128 = i8x16(0, -1, 4, -1, 1, -1, 5, -1, 2, -1, 6, -1, 3, -1, 7, -1);
    const MASK_HI: v128 = i8x16(8, -1, 12, -1, 9, -1, 13, -1, 10, -1, 14, -1, 11, -1, 15, -1);
    const MASK: v128 = i8x16(0, -1, 4, -1, 1, -1, 5, -1, 2, -1, 6, -1, 3, -1, 7, -1);

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;

        let mut sss0 = initial;
        let mut sss1 = initial;
        let mut sss2 = initial;
        let mut sss3 = initial;

        let coeffs = coeffs_chunk.values;
        let coeffs_by_4 = coeffs.chunks_exact(4);
        let reminder1 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let mmk_lo = wasm32_utils::ptr_i16_to_set1_i32(k, 0);
            let mmk_hi = wasm32_utils::ptr_i16_to_set1_i32(k, 2);

            // [8] a3 b3 g3 r3 a2 b2 g2 r2 a1 b1 g1 r1 a0 b0 g0 r0
            let mut source = wasm32_utils::load_v128(src_rows[0], x);
            // [16] a1 a0 b1 b0 g1 g0 r1 r0
            let mut pix = i8x16_swizzle(source, MASK_LO);
            sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk_lo));
            // [16] a3 a2 b3 b2 g3 g2 r3 r2
            pix = i8x16_swizzle(source, MASK_HI);
            sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk_hi));

            source = wasm32_utils::load_v128(src_rows[1], x);
            pix = i8x16_swizzle(source, MASK_LO);
            sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk_lo));
            pix = i8x16_swizzle(source, MASK_HI);
            sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk_hi));

            source = wasm32_utils::load_v128(src_rows[2], x);
            pix = i8x16_swizzle(source, MASK_LO);
            sss2 = i32x4_add(sss2, i32x4_dot_i16x8(pix, mmk_lo));
            pix = i8x16_swizzle(source, MASK_HI);
            sss2 = i32x4_add(sss2, i32x4_dot_i16x8(pix, mmk_hi));

            source = wasm32_utils::load_v128(src_rows[3], x);
            pix = i8x16_swizzle(source, MASK_LO);
            sss3 = i32x4_add(sss3, i32x4_dot_i16x8(pix, mmk_lo));
            pix = i8x16_swizzle(source, MASK_HI);
            sss3 = i32x4_add(sss3, i32x4_dot_i16x8(pix, mmk_hi));
            x += 4;
        }

        let coeffs_by_2 = reminder1.chunks_exact(2);
        let reminder2 = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            // [16] k1 k0 k1 k0 k1 k0 k1 k0
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(k, 0);

            // [8] x x x x x x x x a1 b1 g1 r1 a0 b0 g0 r0
            let mut pix = wasm32_utils::loadl_i64(src_rows[0], x);
            // [16] a1 a0 b1 b0 g1 g0 r1 r0
            pix = i8x16_swizzle(pix, MASK);
            sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk));

            pix = wasm32_utils::loadl_i64(src_rows[1], x);
            pix = i8x16_swizzle(pix, MASK);
            sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk));

            pix = wasm32_utils::loadl_i64(src_rows[2], x);
            pix = i8x16_swizzle(pix, MASK);
            sss2 = i32x4_add(sss2, i32x4_dot_i16x8(pix, mmk));

            pix = wasm32_utils::loadl_i64(src_rows[3], x);
            pix = i8x16_swizzle(pix, MASK);
            sss3 = i32x4_add(sss3, i32x4_dot_i16x8(pix, mmk));

            x += 2;
        }

        if let Some(&k) = reminder2.first() {
            // [16] xx k0 xx k0 xx k0 xx k0
            let mmk = i32x4_splat(k as i32);
            // [16] xx a0 xx b0 xx g0 xx r0
            let mut pix = wasm32_utils::i32x4_extend_low_ptr_u8x4(src_rows[0], x);
            sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk));

            pix = wasm32_utils::i32x4_extend_low_ptr_u8x4(src_rows[1], x);
            sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk));

            pix = wasm32_utils::i32x4_extend_low_ptr_u8x4(src_rows[2], x);
            sss2 = i32x4_add(sss2, i32x4_dot_i16x8(pix, mmk));

            pix = wasm32_utils::i32x4_extend_low_ptr_u8x4(src_rows[3], x);
            sss3 = i32x4_add(sss3, i32x4_dot_i16x8(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss0 = i32x4_shr(sss0, $imm8);
                sss1 = i32x4_shr(sss1, $imm8);
                sss2 = i32x4_shr(sss2, $imm8);
                sss3 = i32x4_shr(sss3, $imm8);
            }};
        }
        constify_imm8!(precision, call);

        sss0 = i16x8_narrow_i32x4(sss0, sss0);
        sss1 = i16x8_narrow_i32x4(sss1, sss1);
        sss2 = i16x8_narrow_i32x4(sss2, sss2);
        sss3 = i16x8_narrow_i32x4(sss3, sss3);
        *dst_rows[0].get_unchecked_mut(dst_x) =
            transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss0, sss0)));
        *dst_rows[1].get_unchecked_mut(dst_x) =
            transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss1, sss1)));
        *dst_rows[2].get_unchecked_mut(dst_x) =
            transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss2, sss2)));
        *dst_rows[3].get_unchecked_mut(dst_x) =
            transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss3, sss3)));
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "simd128")]
unsafe fn horiz_convolution_8u(
    src_row: &[U8x4],
    dst_row: &mut [U8x4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    let initial = i32x4_splat(1 << (precision - 1));
    const SH1: v128 = i8x16(0, -1, 8, -1, 1, -1, 9, -1, 2, -1, 10, -1, 3, -1, 11, -1);
    const SH2: v128 = i8x16(0, 1, 4, 5, 0, 1, 4, 5, 0, 1, 4, 5, 0, 1, 4, 5);
    const SH3: v128 = i8x16(4, -1, 12, -1, 5, -1, 13, -1, 6, -1, 14, -1, 7, -1, 15, -1);
    const SH4: v128 = i8x16(2, 3, 6, 7, 2, 3, 6, 7, 2, 3, 6, 7, 2, 3, 6, 7);
    const SH5: v128 = i8x16(8, 9, 12, 13, 8, 9, 12, 13, 8, 9, 12, 13, 8, 9, 12, 13);
    const SH6: v128 = i8x16(
        10, 11, 14, 15, 10, 11, 14, 15, 10, 11, 14, 15, 10, 11, 14, 15,
    );
    const SH7: v128 = i8x16(0, -1, 4, -1, 1, -1, 5, -1, 2, -1, 6, -1, 3, -1, 7, -1);

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x: usize = coeffs_chunk.start as usize;
        let mut sss = initial;

        let coeffs_by_8 = coeffs_chunk.values.chunks_exact(8);
        let reminder8 = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let ksource = wasm32_utils::load_v128(k, 0);

            let mut source = wasm32_utils::load_v128(src_row, x);

            let mut pix = i8x16_swizzle(source, SH1);
            let mut mmk = i8x16_swizzle(ksource, SH2);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            pix = i8x16_swizzle(source, SH3);
            mmk = i8x16_swizzle(ksource, SH4);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            source = wasm32_utils::load_v128(src_row, x + 4);

            pix = i8x16_swizzle(source, SH1);
            mmk = i8x16_swizzle(ksource, SH5);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            pix = i8x16_swizzle(source, SH3);
            mmk = i8x16_swizzle(ksource, SH6);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            x += 8;
        }

        let coeffs_by_4 = reminder8.chunks_exact(4);
        let reminder4 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let source = wasm32_utils::load_v128(src_row, x);
            let ksource = wasm32_utils::loadl_i64(k, 0);

            let mut pix = i8x16_swizzle(source, SH1);
            let mut mmk = i8x16_swizzle(ksource, SH2);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            pix = i8x16_swizzle(source, SH3);
            mmk = i8x16_swizzle(ksource, SH4);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            x += 4;
        }

        let coeffs_by_2 = reminder4.chunks_exact(2);
        let reminder2 = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(k, 0);
            let source = wasm32_utils::loadl_i64(src_row, x);
            let pix = i8x16_swizzle(source, SH7);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            x += 2
        }

        if let Some(&k) = reminder2.first() {
            let pix = wasm32_utils::i32x4_extend_low_ptr_u8x4(src_row, x);
            let mmk = i32x4_splat(k as i32);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss = i32x4_shr(sss, $imm8);
            }};
        }
        constify_imm8!(precision, call);

        sss = i16x8_narrow_i32x4(sss, sss);
        *dst_row.get_unchecked_mut(dst_x) =
            transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss, sss)));
    }
}
