use std::arch::x86_64::*;
use std::mem::transmute;

use crate::convolution::optimisations::Normalizer16;
use crate::pixels::U8x4;
use crate::{simd_utils, ImageView, ImageViewMut};

// This code is based on C-implementation from Pillow-SIMD package for Python
// https://github.com/uploadcare/pillow-simd

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
    let dst_height = dst_view.height();

    let src_iter = src_view.iter_4_rows(offset, dst_height + offset);
    let dst_iter = dst_view.iter_4_rows_mut();
    for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
        unsafe {
            horiz_convolution_four_rows::<PRECISION>(src_rows, dst_rows, normalizer);
        }
    }

    let yy = dst_height - dst_height % 4;
    let src_rows = src_view.iter_rows(yy + offset);
    let dst_rows = dst_view.iter_rows_mut(yy);
    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        unsafe {
            horiz_convolution_one_row::<PRECISION>(src_row, dst_row, normalizer);
        }
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - length of all rows in src_rows must be equal
/// - length of all rows in dst_rows must be equal
/// - coefficients_chunks.len() == dst_rows.0.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_four_rows<const PRECISION: i32>(
    src_rows: [&[U8x4]; 4],
    dst_rows: [&mut [U8x4]; 4],
    normalizer: &Normalizer16,
) {
    let initial = _mm_set1_epi32(1 << (PRECISION - 1));
    let mask_lo = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);
    let mask_hi = _mm_set_epi8(-1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8);
    let mask = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);

    for (dst_x, chunk) in normalizer.chunks().iter().enumerate() {
        let mut x = chunk.start as usize;

        let mut sss0 = initial;
        let mut sss1 = initial;
        let mut sss2 = initial;
        let mut sss3 = initial;

        let coeffs_by_4 = chunk.values().chunks_exact(4);
        let reminder1 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let mmk_lo = simd_utils::mm_load_and_clone_i16x2(k);
            let mmk_hi = simd_utils::mm_load_and_clone_i16x2(&k[2..]);

            // [8] a3 b3 g3 r3 a2 b2 g2 r2 a1 b1 g1 r1 a0 b0 g0 r0
            let mut source = simd_utils::loadu_si128(src_rows[0], x);
            // [16] a1 a0 b1 b0 g1 g0 r1 r0
            let mut pix = _mm_shuffle_epi8(source, mask_lo);
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk_lo));
            // [16] a3 a2 b3 b2 g3 g2 r3 r2
            pix = _mm_shuffle_epi8(source, mask_hi);
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk_hi));

            source = simd_utils::loadu_si128(src_rows[1], x);
            pix = _mm_shuffle_epi8(source, mask_lo);
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk_lo));
            pix = _mm_shuffle_epi8(source, mask_hi);
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk_hi));

            source = simd_utils::loadu_si128(src_rows[2], x);
            pix = _mm_shuffle_epi8(source, mask_lo);
            sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk_lo));
            pix = _mm_shuffle_epi8(source, mask_hi);
            sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk_hi));

            source = simd_utils::loadu_si128(src_rows[3], x);
            pix = _mm_shuffle_epi8(source, mask_lo);
            sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk_lo));
            pix = _mm_shuffle_epi8(source, mask_hi);
            sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk_hi));
            x += 4;
        }

        let coeffs_by_2 = reminder1.chunks_exact(2);
        let reminder2 = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            // [16] k1 k0 k1 k0 k1 k0 k1 k0
            let mmk = simd_utils::mm_load_and_clone_i16x2(k);

            // [8] x x x x x x x x a1 b1 g1 r1 a0 b0 g0 r0
            let mut pix = simd_utils::loadl_epi64(src_rows[0], x);
            // [16] a1 a0 b1 b0 g1 g0 r1 r0
            pix = _mm_shuffle_epi8(pix, mask);
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));

            pix = simd_utils::loadl_epi64(src_rows[1], x);
            pix = _mm_shuffle_epi8(pix, mask);
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

            pix = simd_utils::loadl_epi64(src_rows[2], x);
            pix = _mm_shuffle_epi8(pix, mask);
            sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));

            pix = simd_utils::loadl_epi64(src_rows[3], x);
            pix = _mm_shuffle_epi8(pix, mask);
            sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

            x += 2;
        }

        if let Some(&k) = reminder2.first() {
            // [16] xx k0 xx k0 xx k0 xx k0
            let mmk = _mm_set1_epi32(k as i32);
            // [16] xx a0 xx b0 xx g0 xx r0
            let mut pix = simd_utils::mm_cvtepu8_epi32(src_rows[0], x);
            sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));

            pix = simd_utils::mm_cvtepu8_epi32(src_rows[1], x);
            sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

            pix = simd_utils::mm_cvtepu8_epi32(src_rows[2], x);
            sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));

            pix = simd_utils::mm_cvtepu8_epi32(src_rows[3], x);
            sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));
        }

        sss0 = _mm_srai_epi32::<PRECISION>(sss0);
        sss1 = _mm_srai_epi32::<PRECISION>(sss1);
        sss2 = _mm_srai_epi32::<PRECISION>(sss2);
        sss3 = _mm_srai_epi32::<PRECISION>(sss3);

        sss0 = _mm_packs_epi32(sss0, sss0);
        sss1 = _mm_packs_epi32(sss1, sss1);
        sss2 = _mm_packs_epi32(sss2, sss2);
        sss3 = _mm_packs_epi32(sss3, sss3);
        *dst_rows[0].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm_packus_epi16(sss0, sss0)));
        *dst_rows[1].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm_packus_epi16(sss1, sss1)));
        *dst_rows[2].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm_packus_epi16(sss2, sss2)));
        *dst_rows[3].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm_packus_epi16(sss3, sss3)));
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coefficients_chunks.len() == dst_row.len()
/// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[target_feature(enable = "sse4.1")]
unsafe fn horiz_convolution_one_row<const PRECISION: i32>(
    src_row: &[U8x4],
    dst_row: &mut [U8x4],
    normalizer: &Normalizer16,
) {
    let initial = _mm_set1_epi32(1 << (PRECISION - 1));
    let sh1 = _mm_set_epi8(-1, 11, -1, 3, -1, 10, -1, 2, -1, 9, -1, 1, -1, 8, -1, 0);
    let sh2 = _mm_set_epi8(5, 4, 1, 0, 5, 4, 1, 0, 5, 4, 1, 0, 5, 4, 1, 0);
    let sh3 = _mm_set_epi8(-1, 15, -1, 7, -1, 14, -1, 6, -1, 13, -1, 5, -1, 12, -1, 4);
    let sh4 = _mm_set_epi8(7, 6, 3, 2, 7, 6, 3, 2, 7, 6, 3, 2, 7, 6, 3, 2);
    let sh5 = _mm_set_epi8(13, 12, 9, 8, 13, 12, 9, 8, 13, 12, 9, 8, 13, 12, 9, 8);
    let sh6 = _mm_set_epi8(
        15, 14, 11, 10, 15, 14, 11, 10, 15, 14, 11, 10, 15, 14, 11, 10,
    );
    let sh7 = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);

    for (dst_x, chunk) in normalizer.chunks().iter().enumerate() {
        let mut x = chunk.start as usize;
        let mut sss = initial;

        let coeffs_by_8 = chunk.values().chunks_exact(8);
        let reminder8 = coeffs_by_8.remainder();

        for k in coeffs_by_8 {
            let ksource = simd_utils::loadu_si128(k, 0);

            let mut source = simd_utils::loadu_si128(src_row, x);

            let mut pix = _mm_shuffle_epi8(source, sh1);
            let mut mmk = _mm_shuffle_epi8(ksource, sh2);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            pix = _mm_shuffle_epi8(source, sh3);
            mmk = _mm_shuffle_epi8(ksource, sh4);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            source = simd_utils::loadu_si128(src_row, x + 4);

            pix = _mm_shuffle_epi8(source, sh1);
            mmk = _mm_shuffle_epi8(ksource, sh5);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            pix = _mm_shuffle_epi8(source, sh3);
            mmk = _mm_shuffle_epi8(ksource, sh6);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            x += 8;
        }

        let coeffs_by_4 = reminder8.chunks_exact(4);
        let reminder4 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let source = simd_utils::loadu_si128(src_row, x);
            let ksource = simd_utils::loadl_epi64(k, 0);

            let mut pix = _mm_shuffle_epi8(source, sh1);
            let mut mmk = _mm_shuffle_epi8(ksource, sh2);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            pix = _mm_shuffle_epi8(source, sh3);
            mmk = _mm_shuffle_epi8(ksource, sh4);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            x += 4;
        }

        let coeffs_by_2 = reminder4.chunks_exact(2);
        let reminder2 = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let mmk = simd_utils::mm_load_and_clone_i16x2(k);
            let source = simd_utils::loadl_epi64(src_row, x);
            let pix = _mm_shuffle_epi8(source, sh7);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            x += 2
        }

        if let Some(&k) = reminder2.first() {
            let pix = simd_utils::mm_cvtepu8_epi32(src_row, x);
            let mmk = _mm_set1_epi32(k as i32);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));
        }

        sss = _mm_srai_epi32::<PRECISION>(sss);
        sss = _mm_packs_epi32(sss, sss);
        *dst_row.get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));
    }
}
