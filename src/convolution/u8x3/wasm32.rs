use std::arch::wasm32::*;
use std::intrinsics::transmute;

use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U8x3;
use crate::wasm32_utils;
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
#[inline]
unsafe fn horiz_convolution_8u4x(
    src_rows: [&[U8x3]; 4],
    dst_rows: [&mut &mut [U8x3]; 4],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    const ZERO: v128 = i64x2(0, 0);
    let initial = i32x4_splat(1 << (precision - 1));
    let src_width = src_rows[0].len();

    /*
        |R  G  B | |R  G  B | |R  G  B | |R  G  B | |R  G  B | |R |
        |00 01 02| |03 04 05| |06 07 08| |09 10 11| |12 13 14| |15|

        Ignore 12-15 bytes in register and
        shuffle other components with converting from u8 into i16:

        x: |-1 -1| |-1 -1|
        B: |-1 05| |-1 02|
        G: |-1 04| |-1 01|
        R: |-1 03| |-1 00|
    */
    #[rustfmt::skip]
    const SH_LO: v128 = i8x16(
        0, -1, 3, -1, 1, -1, 4, -1, 2, -1, 5, -1, -1, -1, -1, -1
    );
    /*
        x: |-1 -1| |-1 -1|
        B: |-1 11| |-1 08|
        G: |-1 10| |-1 07|
        R: |-1 09| |-1 06|
    */
    #[rustfmt::skip]
    const SH_HI: v128 = i8x16(
        6, -1, 9, -1, 7, -1, 10, -1, 8, -1, 11, -1, -1, -1, -1, -1
    );

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let x_start = coeffs_chunk.start as usize;
        let mut x = x_start;

        let mut sss_a = [initial; 4];
        let mut coeffs = coeffs_chunk.values;

        // Next block of code will be load source pixels by 16 bytes per time.
        // We must guarantee what this process will not go beyond
        // the one row of image.
        // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 byte
        let max_x = src_width.saturating_sub(5);
        if x < max_x {
            let coeffs_by_4 = coeffs.chunks_exact(4);

            for k in coeffs_by_4 {
                let mmk0 = wasm32_utils::ptr_i16_to_set1_i32(k, 0);
                let mmk1 = wasm32_utils::ptr_i16_to_set1_i32(k, 2);
                for i in 0..4 {
                    let source = wasm32_utils::load_v128(src_rows[i], x);
                    let pix = i8x16_swizzle(source, SH_LO);
                    let mut sss = sss_a[i];
                    sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk0));
                    let pix = i8x16_swizzle(source, SH_HI);
                    sss_a[i] = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk1));
                }

                x += 4;
                if x >= max_x {
                    break;
                }
            }
        }

        // Next block of code will be load source pixels by 8 bytes per time.
        // We must guarantee what this process will not go beyond
        // the one row of image.
        // (8 bytes) / (3 bytes per pixel) = 2 whole pixels + 2 bytes
        let max_x = src_width.saturating_sub(2);
        if x < max_x {
            let coeffs_by_2 = coeffs[x - x_start..].chunks_exact(2);

            for k in coeffs_by_2 {
                let mmk = wasm32_utils::ptr_i16_to_set1_i32(k, 0);

                for i in 0..4 {
                    let source = wasm32_utils::loadl_i64(src_rows[i], x);
                    let pix = i8x16_swizzle(source, SH_LO);
                    sss_a[i] = i32x4_add(sss_a[i], i32x4_dot_i16x8(pix, mmk));
                }

                x += 2;
                if x >= max_x {
                    break;
                }
            }
        }

        coeffs = coeffs.split_at(x - x_start).1;
        for &k in coeffs {
            let mmk = i32x4_splat(k as i32);
            for i in 0..4 {
                let pix = wasm32_utils::i32x4_extend_low_ptr_u8x3(src_rows[i], x);
                sss_a[i] = i32x4_add(sss_a[i], i32x4_dot_i16x8(pix, mmk));
            }

            x += 1;
        }
        macro_rules! call {
            ($imm8:expr) => {{
                sss_a[0] = i32x4_shr(sss_a[0], $imm8);
                sss_a[1] = i32x4_shr(sss_a[1], $imm8);
                sss_a[2] = i32x4_shr(sss_a[2], $imm8);
                sss_a[3] = i32x4_shr(sss_a[3], $imm8);
            }};
        }
        constify_imm8!(precision, call);

        for i in 0..4 {
            let sss = i16x8_narrow_i32x4(sss_a[i], ZERO);
            let pixel: u32 = transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss, ZERO)));
            let bytes = pixel.to_le_bytes();
            dst_rows[i].get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
unsafe fn horiz_convolution_8u(
    src_row: &[U8x3],
    dst_row: &mut [U8x3],
    coefficients_chunks: &[optimisations::CoefficientsI16Chunk],
    precision: u8,
) {
    #[rustfmt::skip]
    const PIX_SH1: v128 = i8x16(
        0, -1, 3, -1, 1, -1, 4, -1, 2, -1, 5, -1, -1, -1, -1, -1
    );
    #[rustfmt::skip]
    const COEF_SH1: v128 = i8x16(
        0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3
    );
    #[rustfmt::skip]
    const PIX_SH2: v128 = i8x16(
        6, -1, 9, -1, 7, -1, 10, -1, 8, -1, 11, -1, -1, -1, -1, -1
    );
    #[rustfmt::skip]
    const COEF_SH2: v128 = i8x16(
        4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7
    );
    /*
        Load 8 bytes from memory into low half of 16-bytes register:
        |R  G  B | |R  G  B | |R  G |
        |00 01 02| |03 04 05| |06 07| 08 09 10 11 12 13 14 15

        Ignore 06-16 bytes in 16-bytes register and
        shuffle other components with converting from u8 into i16:

        x: |-1 -1| |-1 -1|
        B: |-1 05| |-1 02|
        G: |-1 04| |-1 01|
        R: |-1 03| |-1 00|
    */
    let src_width = src_row.len();

    for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let x_start = coeffs_chunk.start as usize;
        let mut x = x_start;
        let mut coeffs = coeffs_chunk.values;
        let mut sss = i32x4_splat(1 << (precision - 1));

        // Next block of code will be load source pixels by 16 bytes per time.
        // We must guarantee what this process will not go beyond
        // the one row of image.
        // (16 bytes) / (3 bytes per pixel) = 5 whole pixels + 1 bytes
        let max_x = src_width.saturating_sub(5);
        if x < max_x {
            let coeffs_by_4 = coeffs.chunks_exact(4);
            for k in coeffs_by_4 {
                let ksource = wasm32_utils::loadl_i64(k, 0);
                let source = wasm32_utils::load_v128(src_row, x);

                let pix = i8x16_swizzle(source, PIX_SH1);
                let mmk = i8x16_swizzle(ksource, COEF_SH1);
                sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

                let pix = i8x16_swizzle(source, PIX_SH2);
                let mmk = i8x16_swizzle(ksource, COEF_SH2);
                sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

                x += 4;
                if x >= max_x {
                    break;
                }
            }
        }

        // Next block of code will be load source pixels by 8 bytes per time.
        // We must guarantee what this process will not go beyond
        // the one row of image.
        // (8 bytes) / (3 bytes per pixel) = 2 whole pixels + 2 bytes
        let max_x = src_width.saturating_sub(2);
        if x < max_x {
            let coeffs_by_2 = coeffs[x - x_start..].chunks_exact(2);

            for k in coeffs_by_2 {
                let mmk = wasm32_utils::ptr_i16_to_set1_i32(k, 0);
                let source = wasm32_utils::loadl_i64(src_row, x);
                let pix = i8x16_swizzle(source, PIX_SH1);
                sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

                x += 2;
                if x >= max_x {
                    break;
                }
            }
        }

        coeffs = coeffs.split_at(x - x_start).1;
        for &k in coeffs {
            let pix = wasm32_utils::i32x4_extend_low_ptr_u8x3(src_row, x);
            let mmk = i32x4_splat(k as i32);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));
            x += 1;
        }

        macro_rules! call {
            ($imm8:expr) => {{
                sss = i32x4_shr(sss, $imm8);
            }};
        }
        constify_imm8!(precision, call);

        sss = i16x8_narrow_i32x4(sss, sss);
        let pixel: u32 = transmute(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss, sss)));
        let bytes = pixel.to_le_bytes();
        dst_row.get_unchecked_mut(dst_x).0 = [bytes[0], bytes[1], bytes[2]];
    }
}
