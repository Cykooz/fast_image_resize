use std::arch::wasm32::*;

use crate::convolution::optimisations::{CoefficientsI16Chunk, Normalizer16};
use crate::convolution::vertical_u8::native;
use crate::pixels::InnerPixel;
use crate::wasm32_utils;
use crate::{ImageView, ImageViewMut};

#[inline]
pub(crate) fn vert_convolution<T: InnerPixel<Component = u8>>(
    src_view: &impl ImageView<Pixel = T>,
    dst_view: &mut impl ImageViewMut<Pixel = T>,
    offset: u32,
    normalizer: &Normalizer16,
) {
    let coefficients_chunks = normalizer.chunks();
    let src_x = offset as usize * T::count_of_components();

    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, coeffs_chunk) in dst_rows.zip(coefficients_chunks) {
        unsafe {
            vert_convolution_into_one_row_u8(src_view, dst_row, src_x, coeffs_chunk, normalizer);
        }
    }
}

#[inline]
#[target_feature(enable = "simd128")]
unsafe fn vert_convolution_into_one_row_u8<T: InnerPixel<Component = u8>>(
    src_view: &impl ImageView<Pixel = T>,
    dst_row: &mut [T],
    mut src_x: usize,
    coeffs_chunk: &CoefficientsI16Chunk,
    normalizer: &Normalizer16,
) {
    const ZERO: v128 = i64x2(0, 0);
    let y_start = coeffs_chunk.start;
    let coeffs = coeffs_chunk.values();
    let max_rows = coeffs.len() as u32;
    let precision = normalizer.precision() as u32;
    let mut dst_u8 = T::components_mut(dst_row);

    let initial = i32x4_splat(1 << (precision - 1));

    let mut dst_chunks_32 = dst_u8.chunks_exact_mut(32);
    for dst_chunk in &mut dst_chunks_32 {
        let mut sss0 = initial;
        let mut sss1 = initial;
        let mut sss2 = initial;
        let mut sss3 = initial;
        let mut sss4 = initial;
        let mut sss5 = initial;
        let mut sss6 = initial;
        let mut sss7 = initial;

        let mut y: u32 = 0;

        for src_rows in src_view.iter_2_rows(y_start, max_rows) {
            let components1 = T::components(src_rows[0]);
            let components2 = T::components(src_rows[1]);

            // Load two coefficients at once
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(coeffs, y as usize);

            let source1 = wasm32_utils::load_v128(components1, src_x); // top line
            let source2 = wasm32_utils::load_v128(components2, src_x); // bottom line

            let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                source1, source2,
            );
            let pix = i16x8_extend_low_u8x16(source);
            sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk));
            let pix = i16x8_extend_high_u8x16(source);
            sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk));

            let source =
                i8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(
                    source1, source2,
                );
            let pix = i16x8_extend_low_u8x16(source);
            sss2 = i32x4_add(sss2, i32x4_dot_i16x8(pix, mmk));
            let pix = i16x8_extend_high_u8x16(source);
            sss3 = i32x4_add(sss3, i32x4_dot_i16x8(pix, mmk));

            let source1 = wasm32_utils::load_v128(components1, src_x + 16); // top line
            let source2 = wasm32_utils::load_v128(components2, src_x + 16); // bottom line

            let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                source1, source2,
            );
            let pix = i16x8_extend_low_u8x16(source);
            sss4 = i32x4_add(sss4, i32x4_dot_i16x8(pix, mmk));
            let pix = i16x8_extend_high_u8x16(source);
            sss5 = i32x4_add(sss5, i32x4_dot_i16x8(pix, mmk));

            let source =
                i8x16_shuffle::<8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31>(
                    source1, source2,
                );
            let pix = i16x8_extend_low_u8x16(source);
            sss6 = i32x4_add(sss6, i32x4_dot_i16x8(pix, mmk));
            let pix = i16x8_extend_high_u8x16(source);
            sss7 = i32x4_add(sss7, i32x4_dot_i16x8(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            if let Some(s_row) = src_view.iter_rows(y_start + y).next() {
                let components = T::components(s_row);
                let mmk = i32x4_splat(k as i32);

                let source1 = wasm32_utils::load_v128(components, src_x); // top line

                let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                    source1, ZERO,
                );
                let pix = i16x8_extend_low_u8x16(source);
                sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk));
                let pix = i16x8_extend_high_u8x16(source);
                sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk));

                let source = i16x8_extend_high_u8x16(source1);
                let pix = i16x8_extend_low_u8x16(source);
                sss2 = i32x4_add(sss2, i32x4_dot_i16x8(pix, mmk));
                let pix = i16x8_extend_high_u8x16(source);
                sss3 = i32x4_add(sss3, i32x4_dot_i16x8(pix, mmk));

                let source1 = wasm32_utils::load_v128(components, src_x + 16); // top line

                let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                    source1, ZERO,
                );
                let pix = i16x8_extend_low_u8x16(source);
                sss4 = i32x4_add(sss4, i32x4_dot_i16x8(pix, mmk));
                let pix = i16x8_extend_high_u8x16(source);
                sss5 = i32x4_add(sss5, i32x4_dot_i16x8(pix, mmk));

                let source = i16x8_extend_high_u8x16(source1);
                let pix = i16x8_extend_low_u8x16(source);
                sss6 = i32x4_add(sss6, i32x4_dot_i16x8(pix, mmk));
                let pix = i16x8_extend_high_u8x16(source);
                sss7 = i32x4_add(sss7, i32x4_dot_i16x8(pix, mmk));
            }
        }

        sss0 = i32x4_shr(sss0, precision);
        sss1 = i32x4_shr(sss1, precision);
        sss2 = i32x4_shr(sss2, precision);
        sss3 = i32x4_shr(sss3, precision);
        sss4 = i32x4_shr(sss4, precision);
        sss5 = i32x4_shr(sss5, precision);
        sss6 = i32x4_shr(sss6, precision);
        sss7 = i32x4_shr(sss7, precision);

        sss0 = i16x8_narrow_i32x4(sss0, sss1);
        sss2 = i16x8_narrow_i32x4(sss2, sss3);
        sss0 = u8x16_narrow_i16x8(sss0, sss2);
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut v128;
        v128_store(dst_ptr, sss0);
        sss4 = i16x8_narrow_i32x4(sss4, sss5);
        sss6 = i16x8_narrow_i32x4(sss6, sss7);
        sss4 = u8x16_narrow_i16x8(sss4, sss6);
        let dst_ptr = dst_ptr.add(1);
        v128_store(dst_ptr, sss4);

        src_x += 32;
    }

    dst_u8 = dst_chunks_32.into_remainder();
    let mut dst_chunks_8 = dst_u8.chunks_exact_mut(8);
    for dst_chunk in &mut dst_chunks_8 {
        let mut sss0 = initial; // left row
        let mut sss1 = initial; // right row
        let mut y: u32 = 0;

        for src_rows in src_view.iter_2_rows(y_start, max_rows) {
            let components1 = T::components(src_rows[0]);
            let components2 = T::components(src_rows[1]);
            // Load two coefficients at once
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(coeffs, y as usize);

            let source1 = wasm32_utils::loadl_i64(components1, src_x); // top line
            let source2 = wasm32_utils::loadl_i64(components2, src_x); // bottom line

            let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                source1, source2,
            );
            let pix = i16x8_extend_low_u8x16(source);
            sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk));
            let pix = i16x8_extend_high_u8x16(source);
            sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            if let Some(s_row) = src_view.iter_rows(y_start + y).next() {
                let components = T::components(s_row);
                let mmk = i32x4_splat(k as i32);

                let source1 = wasm32_utils::loadl_i64(components, src_x); // top line

                let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                    source1, ZERO,
                );
                let pix = i16x8_extend_low_u8x16(source);
                sss0 = i32x4_add(sss0, i32x4_dot_i16x8(pix, mmk));
                let pix = i16x8_extend_high_u8x16(source);
                sss1 = i32x4_add(sss1, i32x4_dot_i16x8(pix, mmk));
            }
        }

        sss0 = i32x4_shr(sss0, precision);
        sss1 = i32x4_shr(sss1, precision);

        sss0 = i16x8_narrow_i32x4(sss0, sss1);
        sss0 = u8x16_narrow_i16x8(sss0, sss0);
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut i64;
        dst_ptr.write_unaligned(i64x2_extract_lane::<0>(sss0));

        src_x += 8;
    }

    dst_u8 = dst_chunks_8.into_remainder();
    let mut dst_chunks_4 = dst_u8.chunks_exact_mut(4);
    if let Some(dst_chunk) = dst_chunks_4.next() {
        let mut sss = initial;
        let mut y: u32 = 0;

        for src_rows in src_view.iter_2_rows(y_start, max_rows) {
            let components1 = T::components(src_rows[0]);
            let components2 = T::components(src_rows[1]);
            // Load two coefficients at once
            let mmk = wasm32_utils::ptr_i16_to_set1_i32(coeffs, y as usize);

            let source1 = wasm32_utils::i32x4_v128_from_u8(components1, src_x); // top line
            let source2 = wasm32_utils::i32x4_v128_from_u8(components2, src_x); // bottom line

            let source = i8x16_shuffle::<0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23>(
                source1, source2,
            );
            let pix = i16x8_extend_low_u8x16(source);
            sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));

            y += 2;
        }

        if let Some(&k) = coeffs.get(y as usize) {
            if let Some(s_row) = src_view.iter_rows(y_start + y).next() {
                let components = T::components(s_row);
                let pix = wasm32_utils::i32x4_extend_low_ptr_u8(components, src_x);
                let mmk = i32x4_splat(k as i32);
                sss = i32x4_add(sss, i32x4_dot_i16x8(pix, mmk));
            }
        }

        sss = i32x4_shr(sss, precision);

        sss = i16x8_narrow_i32x4(sss, sss);
        let dst_ptr = dst_chunk.as_mut_ptr() as *mut i32;
        dst_ptr.write_unaligned(i32x4_extract_lane::<0>(u8x16_narrow_i16x8(sss, sss)));

        src_x += 4;
    }

    dst_u8 = dst_chunks_4.into_remainder();
    if !dst_u8.is_empty() {
        native::convolution_by_u8(
            src_view,
            normalizer,
            1 << (precision - 1),
            dst_u8,
            src_x,
            y_start,
            coeffs,
        );
    }
}
