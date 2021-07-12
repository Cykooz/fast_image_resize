use std::arch::x86_64::*;
use std::intrinsics::transmute;

use crate::convolution::{Bound, Coefficients, CoefficientsChunk, Convolution};
use crate::image_view::{DstImageView, FourRows, FourRowsMut, SrcImageView};
use crate::{optimisations, simd_utils};

pub struct Sse4;

// This code is based on C-implementation from Pillow-SIMD package for Python
// https://github.com/uploadcare/pillow-simd
impl Sse4 {
    /// For safety, it is necessary to ensure the following conditions:
    /// - length of all rows in src_rows must be equal
    /// - length of all rows in dst_rows must be equal
    /// - bounds.len() == dst_rows.0.len()
    /// - coeffs.len() == dst_rows.0.len() * window_size
    /// - max(bound.size for bound in bounds) <= window_size
    /// - max(bound.start + bound.size for bound in bounds) <= src_row.0.len()
    /// - precision <= MAX_COEFS_PRECISION
    #[target_feature(enable = "sse4.1")]
    unsafe fn horiz_convolution_8u4x(
        &self,
        src_rows: FourRows,
        dst_rows: FourRowsMut,
        coefficients_chunks: &[CoefficientsChunk],
        precision: u8,
    ) {
        let (s_row0, s_row1, s_row2, s_row3) = src_rows;
        let (d_row0, d_row1, d_row2, d_row3) = dst_rows;
        let initial = _mm_set1_epi32(1 << (precision - 1));
        let mask_lo = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);
        let mask_hi = _mm_set_epi8(-1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8);
        let mask = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);

        for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
            let x_start = coeffs_chunk.start as usize;
            let mut x: usize = 0;

            let mut sss0 = initial;
            let mut sss1 = initial;
            let mut sss2 = initial;
            let mut sss3 = initial;

            let coeffs = coeffs_chunk.values;
            let coeffs_by_4 = coeffs.chunks_exact(4);
            let reminder1 = coeffs_by_4.remainder();

            for k in coeffs_by_4 {
                let mmk_lo = simd_utils::ptr_i16_to_set1_epi32(k, 0);
                let mmk_hi = simd_utils::ptr_i16_to_set1_epi32(k, 2);

                // [8] a3 b3 g3 r3 a2 b2 g2 r2 a1 b1 g1 r1 a0 b0 g0 r0
                let mut source = simd_utils::loadu_si128(s_row0, x + x_start);
                // [16] a1 a0 b1 b0 g1 g0 r1 r0
                let mut pix = _mm_shuffle_epi8(source, mask_lo);
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk_lo));
                // [16] a3 a2 b3 b2 g3 g2 r3 r2
                pix = _mm_shuffle_epi8(source, mask_hi);
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk_hi));

                source = simd_utils::loadu_si128(s_row1, x + x_start);
                pix = _mm_shuffle_epi8(source, mask_lo);
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk_lo));
                pix = _mm_shuffle_epi8(source, mask_hi);
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk_hi));

                source = simd_utils::loadu_si128(s_row2, x + x_start);
                pix = _mm_shuffle_epi8(source, mask_lo);
                sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk_lo));
                pix = _mm_shuffle_epi8(source, mask_hi);
                sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk_hi));

                source = simd_utils::loadu_si128(s_row3, x + x_start);
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
                let mmk = simd_utils::ptr_i16_to_set1_epi32(k, 0);

                // [8] x x x x x x x x a1 b1 g1 r1 a0 b0 g0 r0
                let mut pix = simd_utils::loadl_epi64(s_row0, x + x_start);
                // [16] a1 a0 b1 b0 g1 g0 r1 r0
                pix = _mm_shuffle_epi8(pix, mask);
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));

                pix = simd_utils::loadl_epi64(s_row1, x + x_start);
                pix = _mm_shuffle_epi8(pix, mask);
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                pix = simd_utils::loadl_epi64(s_row2, x + x_start);
                pix = _mm_shuffle_epi8(pix, mask);
                sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));

                pix = simd_utils::loadl_epi64(s_row3, x + x_start);
                pix = _mm_shuffle_epi8(pix, mask);
                sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

                x += 2;
            }

            for &k in reminder2 {
                // [16] xx k0 xx k0 xx k0 xx k0
                let mmk = _mm_set1_epi32(k as i32);
                // [16] xx a0 xx b0 xx g0 xx r0
                let mut pix = simd_utils::mm_cvtepu8_epi32(s_row0, x);
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));

                pix = simd_utils::mm_cvtepu8_epi32(s_row1, x);
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                pix = simd_utils::mm_cvtepu8_epi32(s_row2, x);
                sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));

                pix = simd_utils::mm_cvtepu8_epi32(s_row3, x);
                sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

                x += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss0 = _mm_srai_epi32(sss0, $imm8);
                    sss1 = _mm_srai_epi32(sss1, $imm8);
                    sss2 = _mm_srai_epi32(sss2, $imm8);
                    sss3 = _mm_srai_epi32(sss3, $imm8);
                }};
            }
            constify_imm8!(precision, call);

            sss0 = _mm_packs_epi32(sss0, sss0);
            sss1 = _mm_packs_epi32(sss1, sss1);
            sss2 = _mm_packs_epi32(sss2, sss2);
            sss3 = _mm_packs_epi32(sss3, sss3);
            *d_row0.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss0, sss0)));
            *d_row1.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss1, sss1)));
            *d_row2.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss2, sss2)));
            *d_row3.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss3, sss3)));
        }
    }

    /// For safety, it is necessary to ensure the following conditions:
    /// - bounds.len() == dst_row.len()
    /// - coeffs.len() == dst_rows.0.len() * window_size
    /// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
    /// - precision <= MAX_COEFS_PRECISION
    #[target_feature(enable = "sse4.1")]
    unsafe fn horiz_convolution_8u(
        &self,
        src_row: &[u32],
        dst_row: &mut [u32],
        coefficients_chunks: &[CoefficientsChunk],
        precision: u8,
    ) {
        let initial = _mm_set1_epi32(1 << (precision - 1));
        let sh1 = _mm_set_epi8(-1, 11, -1, 3, -1, 10, -1, 2, -1, 9, -1, 1, -1, 8, -1, 0);
        let sh2 = _mm_set_epi8(5, 4, 1, 0, 5, 4, 1, 0, 5, 4, 1, 0, 5, 4, 1, 0);
        let sh3 = _mm_set_epi8(-1, 15, -1, 7, -1, 14, -1, 6, -1, 13, -1, 5, -1, 12, -1, 4);
        let sh4 = _mm_set_epi8(7, 6, 3, 2, 7, 6, 3, 2, 7, 6, 3, 2, 7, 6, 3, 2);
        let sh5 = _mm_set_epi8(13, 12, 9, 8, 13, 12, 9, 8, 13, 12, 9, 8, 13, 12, 9, 8);
        let sh6 = _mm_set_epi8(
            15, 14, 11, 10, 15, 14, 11, 10, 15, 14, 11, 10, 15, 14, 11, 10,
        );
        let sh7 = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);

        for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
            // for (dst_x, (&bound, k)) in bounds.iter().zip(coeffs_chunks).enumerate() {
            let x_start = coeffs_chunk.start as usize;
            let mut x: usize = 0;
            let mut coeffs = coeffs_chunk.values;

            let mut sss = initial;

            let coeffs_by_8 = coeffs.chunks_exact(8);
            let reminder1 = coeffs_by_8.remainder();

            for k in coeffs_by_8 {
                let ksource = simd_utils::loadu_si128(k, 0);

                let mut source = simd_utils::loadu_si128(src_row, x + x_start);

                let mut pix = _mm_shuffle_epi8(source, sh1);
                let mut mmk = _mm_shuffle_epi8(ksource, sh2);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                pix = _mm_shuffle_epi8(source, sh3);
                mmk = _mm_shuffle_epi8(ksource, sh4);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                source = simd_utils::loadu_si128(src_row, x + 4 + x_start);

                pix = _mm_shuffle_epi8(source, sh1);
                mmk = _mm_shuffle_epi8(ksource, sh5);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                pix = _mm_shuffle_epi8(source, sh3);
                mmk = _mm_shuffle_epi8(ksource, sh6);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                x += 8;
            }

            let coeffs_by_4 = reminder1.chunks_exact(4);
            coeffs = coeffs_by_4.remainder();

            for k in coeffs_by_4 {
                let source = simd_utils::loadu_si128(src_row, x + x_start);
                let ksource = simd_utils::loadl_epi64(k, 0);

                let mut pix = _mm_shuffle_epi8(source, sh1);
                let mut mmk = _mm_shuffle_epi8(ksource, sh2);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                pix = _mm_shuffle_epi8(source, sh3);
                mmk = _mm_shuffle_epi8(ksource, sh4);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                x += 4;
            }

            let coeffs_by_2 = coeffs.chunks_exact(2);
            let reminder1 = coeffs_by_2.remainder();

            for k in coeffs_by_2 {
                let mmk = simd_utils::ptr_i16_to_set1_epi32(k, 0);
                let source = simd_utils::loadl_epi64(src_row, x + x_start);
                let pix = _mm_shuffle_epi8(source, sh7);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                x += 2
            }

            for &k in reminder1 {
                let pix = simd_utils::mm_cvtepu8_epi32(src_row, x + x_start);
                let mmk = _mm_set1_epi32(k as i32);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                x += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss = _mm_srai_epi32(sss, $imm8);
                }};
            }
            constify_imm8!(precision, call);

            sss = _mm_packs_epi32(sss, sss);
            *dst_row.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));
        }
    }

    #[target_feature(enable = "sse4.1")]
    pub unsafe fn vert_convolution_8u(
        &self,
        src_img: &SrcImageView,
        dst_row: &mut [u32],
        coeffs: &[i16],
        bound: Bound,
        precision: u8,
    ) {
        let mut xx: usize = 0;
        let src_width = src_img.width().get() as usize;
        let y_start = bound.start;
        let y_size = bound.size;

        let initial = _mm_set1_epi32(1 << (precision - 1));

        while xx < src_width.saturating_sub(7) {
            let mut sss0 = initial;
            let mut sss1 = initial;
            let mut sss2 = initial;
            let mut sss3 = initial;
            let mut sss4 = initial;
            let mut sss5 = initial;
            let mut sss6 = initial;
            let mut sss7 = initial;

            let mut y: u32 = 0;

            for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
                // Load two coefficients at once
                let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

                let mut source1 = simd_utils::loadu_si128(s_row1, xx); // top line
                let mut source2 = simd_utils::loadu_si128(s_row2, xx); // bottom line

                let mut source = _mm_unpacklo_epi8(source1, source2);
                let mut pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                source = _mm_unpackhi_epi8(source1, source2);
                pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

                source1 = simd_utils::loadu_si128(s_row1, xx + 4); // top line
                source2 = simd_utils::loadu_si128(s_row2, xx + 4); // bottom line

                source = _mm_unpacklo_epi8(source1, source2);
                pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss4 = _mm_add_epi32(sss4, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss5 = _mm_add_epi32(sss5, _mm_madd_epi16(pix, mmk));

                source = _mm_unpackhi_epi8(source1, source2);
                pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss6 = _mm_add_epi32(sss6, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss7 = _mm_add_epi32(sss7, _mm_madd_epi16(pix, mmk));

                y += 2;
            }

            for s_row in src_img.iter_rows(y_start + y, y_start + y_size) {
                let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

                let mut source1 = simd_utils::loadu_si128(s_row, xx); // top line

                let mut source = _mm_unpacklo_epi8(source1, _mm_setzero_si128());
                let mut pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                source = _mm_unpackhi_epi8(source1, _mm_setzero_si128());
                pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss2 = _mm_add_epi32(sss2, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss3 = _mm_add_epi32(sss3, _mm_madd_epi16(pix, mmk));

                source1 = simd_utils::loadu_si128(s_row, xx + 4); // top line

                source = _mm_unpacklo_epi8(source1, _mm_setzero_si128());
                pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss4 = _mm_add_epi32(sss4, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss5 = _mm_add_epi32(sss5, _mm_madd_epi16(pix, mmk));

                source = _mm_unpackhi_epi8(source1, _mm_setzero_si128());
                pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss6 = _mm_add_epi32(sss6, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss7 = _mm_add_epi32(sss7, _mm_madd_epi16(pix, mmk));

                y += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss0 = _mm_srai_epi32(sss0, $imm8);
                    sss1 = _mm_srai_epi32(sss1, $imm8);
                    sss2 = _mm_srai_epi32(sss2, $imm8);
                    sss3 = _mm_srai_epi32(sss3, $imm8);
                    sss4 = _mm_srai_epi32(sss4, $imm8);
                    sss5 = _mm_srai_epi32(sss5, $imm8);
                    sss6 = _mm_srai_epi32(sss6, $imm8);
                    sss7 = _mm_srai_epi32(sss7, $imm8);
                }};
            }
            constify_imm8!(precision, call);

            sss0 = _mm_packs_epi32(sss0, sss1);
            sss2 = _mm_packs_epi32(sss2, sss3);
            sss0 = _mm_packus_epi16(sss0, sss2);
            let dst_ptr = dst_row.get_unchecked_mut(xx..).as_mut_ptr() as *mut __m128i;
            _mm_storeu_si128(dst_ptr, sss0);
            sss4 = _mm_packs_epi32(sss4, sss5);
            sss6 = _mm_packs_epi32(sss6, sss7);
            sss4 = _mm_packus_epi16(sss4, sss6);
            let dst_ptr = dst_row.get_unchecked_mut(xx + 4..).as_mut_ptr() as *mut __m128i;
            _mm_storeu_si128(dst_ptr, sss4);

            xx += 8;
        }

        while xx < src_width.saturating_sub(1) {
            let mut sss0 = initial; // left row
            let mut sss1 = initial; // right row
            let mut y: u32 = 0;

            for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
                // Load two coefficients at once
                let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

                let source1 = simd_utils::loadl_epi64(s_row1, xx); // top line
                let source2 = simd_utils::loadl_epi64(s_row2, xx); // bottom line

                let source = _mm_unpacklo_epi8(source1, source2);
                let mut pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                y += 2;
            }

            for s_row1 in src_img.iter_rows(y_start + y, y_start + y_size) {
                let mmk = _mm_set1_epi32(*coeffs.get_unchecked(y as usize) as i32);

                let source1 = simd_utils::loadl_epi64(s_row1, xx); // top line

                let source = _mm_unpacklo_epi8(source1, _mm_setzero_si128());
                let mut pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                y += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss0 = _mm_srai_epi32(sss0, $imm8);
                    sss1 = _mm_srai_epi32(sss1, $imm8);
                }};
            }
            constify_imm8!(precision, call);

            sss0 = _mm_packs_epi32(sss0, sss1);
            sss0 = _mm_packus_epi16(sss0, sss0);
            let dst_ptr = dst_row.get_unchecked_mut(xx..).as_mut_ptr() as *mut __m128i;
            _mm_storel_epi64(dst_ptr, sss0);

            //
            xx += 2;
        }

        while xx < src_width {
            let mut sss = initial;
            let mut y: u32 = 0;

            for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
                // Load two coefficients at once
                let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

                let source1 = simd_utils::mm_cvtsi32_si128(s_row1, xx); // top line
                let source2 = simd_utils::mm_cvtsi32_si128(s_row2, xx); // bottom line

                let source = _mm_unpacklo_epi8(source1, source2);
                let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                y += 2;
            }

            for s_row in src_img.iter_rows(y_start + y, y_start + y_size) {
                let pix = simd_utils::mm_cvtepu8_epi32(s_row, xx);
                let mmk = _mm_set1_epi32(*coeffs.get_unchecked(y as usize) as i32);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                y += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss = _mm_srai_epi32(sss, $imm8);
                }};
            }
            constify_imm8!(precision, call);

            sss = _mm_packs_epi32(sss, sss);
            *dst_row.get_unchecked_mut(xx) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));

            xx += 1;
        }
    }
}

impl Convolution for Sse4 {
    #[inline]
    fn horiz_convolution(
        &self,
        src_image: &SrcImageView,
        dst_image: &mut DstImageView,
        offset: u32,
        coeffs: Coefficients,
    ) {
        let (values, window_size, bounds_per_pixel) =
            (coeffs.values, coeffs.window_size, coeffs.bounds);

        let normalizer_guard = optimisations::NormalizerGuard::new(values);
        let precision = normalizer_guard.precision();
        let coefficients_chunks =
            normalizer_guard.normalized_chunks(window_size, &bounds_per_pixel);
        let dst_height = dst_image.height().get();

        let src_iter = src_image.iter_4_rows(offset, dst_height + offset);
        let dst_iter = dst_image.iter_4_rows_mut();
        for (src_rows, dst_rows) in src_iter.zip(dst_iter) {
            unsafe {
                self.horiz_convolution_8u4x(src_rows, dst_rows, &coefficients_chunks, precision);
            }
        }

        let mut yy = dst_height - dst_height % 4;
        while yy < dst_height {
            unsafe {
                self.horiz_convolution_8u(
                    src_image.get_row(yy + offset).unwrap(),
                    dst_image.get_row_mut(yy).unwrap(),
                    &coefficients_chunks,
                    precision,
                );
            }
            yy += 1;
        }
    }

    #[inline]
    fn vert_convolution(
        &self,
        src_image: &SrcImageView,
        dst_image: &mut DstImageView,
        coeffs: Coefficients,
    ) {
        let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

        let normalizer_guard = optimisations::NormalizerGuard::new(values);
        let precision = normalizer_guard.precision();
        let coeffs_i16 = normalizer_guard.normalized();
        let coeffs_chunks = coeffs_i16.chunks(window_size);

        let dst_rows = dst_image.iter_rows_mut();
        for ((&bound, k), dst_row) in bounds.iter().zip(coeffs_chunks).zip(dst_rows) {
            unsafe {
                self.vert_convolution_8u(src_image, dst_row, k, bound, precision);
            }
        }
    }
}
