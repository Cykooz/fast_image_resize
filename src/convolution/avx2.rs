use std::arch::x86_64::*;
use std::intrinsics::transmute;

use crate::convolution::{Bound, Coefficients, CoefficientsChunk, Convolution};
use crate::image_view::{DstImageView, FourRows, FourRowsMut, SrcImageView};
use crate::{optimisations, simd_utils};

pub struct Avx2U8x4;

// This code is based on C-implementation from Pillow-SIMD package for Python
// https://github.com/uploadcare/pillow-simd
impl Avx2U8x4 {
    /// For safety, it is necessary to ensure the following conditions:
    /// - length of all rows in src_rows must be equal
    /// - length of all rows in dst_rows must be equal
    /// - coefficients_chunks.len() == dst_rows.0.len()
    /// - max(chunk.start + chunk.values.len() for chunk in coefficients_chunks) <= src_row.0.len()
    /// - precision <= MAX_COEFS_PRECISION
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn horiz_convolution_8u4x(
        &self,
        src_rows: FourRows,
        dst_rows: FourRowsMut,
        coefficients_chunks: &[CoefficientsChunk],
        precision: u8,
    ) {
        let (s_row0, s_row1, s_row2, s_row3) = src_rows;
        let (d_row0, d_row1, d_row2, d_row3) = dst_rows;
        let zero = _mm256_setzero_si256();
        let initial = _mm256_set1_epi32(1 << (precision - 1));

        #[rustfmt::skip]
        let sh1 = _mm256_set_epi8(
            -1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0,
            -1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0,
        );
        #[rustfmt::skip]
        let sh2 = _mm256_set_epi8(
            -1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8,
            -1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8,
        );

        for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
            let x_start = coeffs_chunk.start as usize;
            let mut x: usize = 0;

            let mut sss0 = initial;
            let mut sss1 = initial;
            let coeffs = coeffs_chunk.values;

            let coeffs_by_4 = coeffs.chunks_exact(4);
            let reminder1 = coeffs_by_4.remainder();

            for k in coeffs_by_4 {
                let mmk0 = simd_utils::ptr_i16_to_256set1_epi32(k, 0);
                let mmk1 = simd_utils::ptr_i16_to_256set1_epi32(k, 2);

                let mut source = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadu_si128(s_row0, x + x_start)),
                    simd_utils::loadu_si128(s_row1, x + x_start),
                );
                let mut pix = _mm256_shuffle_epi8(source, sh1);
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk0));
                pix = _mm256_shuffle_epi8(source, sh2);
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk1));

                source = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadu_si128(s_row2, x + x_start)),
                    simd_utils::loadu_si128(s_row3, x + x_start),
                );
                pix = _mm256_shuffle_epi8(source, sh1);
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk0));
                pix = _mm256_shuffle_epi8(source, sh2);
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk1));

                x += 4;
            }

            let coeffs_by_2 = reminder1.chunks_exact(2);
            let reminder2 = coeffs_by_2.remainder();

            for k in coeffs_by_2 {
                let mmk = simd_utils::ptr_i16_to_256set1_epi32(k, 0);

                let mut pix = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadl_epi64(s_row0, x + x_start)),
                    simd_utils::loadl_epi64(s_row1, x + x_start),
                );
                pix = _mm256_shuffle_epi8(pix, sh1);
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

                pix = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::loadl_epi64(s_row2, x + x_start)),
                    simd_utils::loadl_epi64(s_row3, x + x_start),
                );
                pix = _mm256_shuffle_epi8(pix, sh1);
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

                x += 2;
            }

            for &k in reminder2 {
                // [16] xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0
                let mmk = _mm256_set1_epi32(k as i32);

                // [16] xx a0 xx b0 xx g0 xx r0 xx a0 xx b0 xx g0 xx r0
                let mut pix = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::mm_cvtepu8_epi32(s_row0, x + x_start)),
                    simd_utils::mm_cvtepu8_epi32(s_row1, x + x_start),
                );
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

                pix = _mm256_inserti128_si256::<1>(
                    _mm256_castsi128_si256(simd_utils::mm_cvtepu8_epi32(s_row2, x + x_start)),
                    simd_utils::mm_cvtepu8_epi32(s_row3, x + x_start),
                );
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));
                x += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss0 = _mm256_srai_epi32::<$imm8>(sss0);
                    sss1 = _mm256_srai_epi32::<$imm8>(sss1);
                }};
            }
            constify_imm8!(precision, call);

            sss0 = _mm256_packs_epi32(sss0, zero);
            sss1 = _mm256_packs_epi32(sss1, zero);
            sss0 = _mm256_packus_epi16(sss0, zero);
            sss1 = _mm256_packus_epi16(sss1, zero);
            *d_row0.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<0>(sss0)));
            *d_row1.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<1>(sss0)));
            *d_row2.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<0>(sss1)));
            *d_row3.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm256_extracti128_si256::<1>(sss1)));
        }
    }

    /// For safety, it is necessary to ensure the following conditions:
    /// - bounds.len() == dst_row.len()
    /// - coeffs.len() == dst_rows.0.len() * window_size
    /// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
    /// - precision <= MAX_COEFS_PRECISION
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn horiz_convolution_8u(
        &self,
        src_row: &[u32],
        dst_row: &mut [u32],
        coefficients_chunks: &[CoefficientsChunk],
        precision: u8,
    ) {
        #[rustfmt::skip]
            let sh1 = _mm256_set_epi8(
            -1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0,
            -1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0,
        );
        #[rustfmt::skip]
            let sh2 = _mm256_set_epi8(
            11, 10, 9, 8, 11, 10, 9, 8, 11, 10, 9, 8, 11, 10, 9, 8,
            3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0,
        );
        #[rustfmt::skip]
            let sh3 = _mm256_set_epi8(
            -1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8,
            -1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8,
        );
        #[rustfmt::skip]
            let sh4 = _mm256_set_epi8(
            15, 14, 13, 12, 15, 14, 13, 12, 15, 14, 13, 12, 15, 14, 13, 12,
            7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4,
        );
        #[rustfmt::skip]
            let sh5 = _mm256_set_epi8(
            -1, 15, -1, 11, -1, 14, -1, 10, -1, 13, -1, 9, -1, 12, -1, 8,
            -1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0,
        );
        #[rustfmt::skip]
            let sh6 = _mm256_set_epi8(
            7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4, 7, 6, 5, 4,
            3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0,
        );
        let sh7 = _mm_set_epi8(-1, 7, -1, 3, -1, 6, -1, 2, -1, 5, -1, 1, -1, 4, -1, 0);

        for (dst_x, &coeffs_chunk) in coefficients_chunks.iter().enumerate() {
            let x_start = coeffs_chunk.start as usize;
            let mut x: usize = 0;
            let mut coeffs = coeffs_chunk.values;

            let mut sss: __m128i = if coeffs.len() < 8 {
                _mm_set1_epi32(1 << (precision - 1))
            } else {
                // Lower part will be added to higher, use only half of the error
                let mut sss256 = _mm256_set1_epi32(1 << (precision - 2));

                let coeffs_by_8 = coeffs.chunks_exact(8);
                let reminder1 = coeffs_by_8.remainder();

                for k in coeffs_by_8 {
                    let tmp = simd_utils::loadu_si128(k, 0);
                    let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                    let source = simd_utils::loadu_si256(src_row, x + x_start);

                    let mut pix = _mm256_shuffle_epi8(source, sh1);
                    let mut mmk = _mm256_shuffle_epi8(ksource, sh2);
                    sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                    pix = _mm256_shuffle_epi8(source, sh3);
                    mmk = _mm256_shuffle_epi8(ksource, sh4);
                    sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                    x += 8;
                }

                let coeffs_by_4 = reminder1.chunks_exact(4);
                coeffs = coeffs_by_4.remainder();

                for k in coeffs_by_4 {
                    let tmp = simd_utils::loadl_epi64(k, 0);
                    let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                    let tmp = simd_utils::loadu_si128(src_row, x + x_start);
                    let source = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                    let pix = _mm256_shuffle_epi8(source, sh5);
                    let mmk = _mm256_shuffle_epi8(ksource, sh6);
                    sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                    x += 4;
                }

                _mm_add_epi32(
                    _mm256_extracti128_si256::<0>(sss256),
                    _mm256_extracti128_si256::<1>(sss256),
                )
            };

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
                    sss = _mm_srai_epi32::<$imm8>(sss);
                }};
            }
            constify_imm8!(precision, call);

            sss = _mm_packs_epi32(sss, sss);
            *dst_row.get_unchecked_mut(dst_x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));
        }
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    pub unsafe fn vert_convolution_8u(
        &self,
        src_img: &SrcImageView,
        dst_row: &mut [u32],
        coeffs: &[i16],
        bound: Bound,
        precision: u8,
    ) {
        let src_width = src_img.width().get() as usize;
        let y_start = bound.start;
        let y_size = bound.size;

        let initial = _mm_set1_epi32(1 << (precision - 1));
        let initial_256 = _mm256_set1_epi32(1 << (precision - 1));

        let mut x: usize = 0;
        while x < src_width.saturating_sub(7) {
            let mut sss0 = initial_256;
            let mut sss1 = initial_256;
            let mut sss2 = initial_256;
            let mut sss3 = initial_256;

            let mut y: u32 = 0;

            for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
                // Load two coefficients at once
                let mmk = simd_utils::ptr_i16_to_256set1_epi32(coeffs, y as usize);

                let source1 = simd_utils::loadu_si256(s_row1, x); // top line
                let source2 = simd_utils::loadu_si256(s_row2, x); // bottom line

                let mut source = _mm256_unpacklo_epi8(source1, source2);
                let mut pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));
                pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

                source = _mm256_unpackhi_epi8(source1, source2);
                pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
                sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(pix, mmk));
                pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
                sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(pix, mmk));

                y += 2;
            }

            for s_row in src_img.iter_rows(y_start + y, y_start + y_size) {
                let mmk = _mm256_set1_epi32(coeffs[y as usize] as i32);

                let source1 = simd_utils::loadu_si256(s_row, x); // top line
                let source2 = _mm256_setzero_si256(); // bottom line is empty

                let mut source = _mm256_unpacklo_epi8(source1, source2);
                let mut pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
                sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));
                pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
                sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

                source = _mm256_unpackhi_epi8(source1, _mm256_setzero_si256());
                pix = _mm256_unpacklo_epi8(source, _mm256_setzero_si256());
                sss2 = _mm256_add_epi32(sss2, _mm256_madd_epi16(pix, mmk));
                pix = _mm256_unpackhi_epi8(source, _mm256_setzero_si256());
                sss3 = _mm256_add_epi32(sss3, _mm256_madd_epi16(pix, mmk));

                y += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss0 = _mm256_srai_epi32::<$imm8>(sss0);
                    sss1 = _mm256_srai_epi32::<$imm8>(sss1);
                    sss2 = _mm256_srai_epi32::<$imm8>(sss2);
                    sss3 = _mm256_srai_epi32::<$imm8>(sss3);
                }};
            }
            constify_imm8!(precision, call);

            sss0 = _mm256_packs_epi32(sss0, sss1);
            sss2 = _mm256_packs_epi32(sss2, sss3);
            sss0 = _mm256_packus_epi16(sss0, sss2);
            let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m256i;
            _mm256_storeu_si256(dst_ptr, sss0);

            x += 8;
        }

        while x < src_width.saturating_sub(1) {
            let mut sss0 = initial; // left row
            let mut sss1 = initial; // right row
            let mut y: u32 = 0;

            for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
                // Load two coefficients at once
                let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

                let source1 = simd_utils::loadl_epi64(s_row1, x); // top line
                let source2 = simd_utils::loadl_epi64(s_row2, x); // bottom line

                let source = _mm_unpacklo_epi8(source1, source2);
                let mut pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                y += 2;
            }

            for s_row in src_img.iter_rows(y_start + y, y_start + y_size) {
                let mmk = _mm_set1_epi32(*coeffs.get_unchecked(y as usize) as i32);

                let source1 = simd_utils::loadl_epi64(s_row, x); // top line
                let source2 = _mm_setzero_si128(); // bottom line is empty

                let source = _mm_unpacklo_epi8(source1, source2);
                let mut pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss0 = _mm_add_epi32(sss0, _mm_madd_epi16(pix, mmk));
                pix = _mm_unpackhi_epi8(source, _mm_setzero_si128());
                sss1 = _mm_add_epi32(sss1, _mm_madd_epi16(pix, mmk));

                y += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss0 = _mm_srai_epi32::<$imm8>(sss0);
                    sss1 = _mm_srai_epi32::<$imm8>(sss1);
                }};
            }
            constify_imm8!(precision, call);

            sss0 = _mm_packs_epi32(sss0, sss1);
            sss0 = _mm_packus_epi16(sss0, sss0);
            let dst_ptr = dst_row.get_unchecked_mut(x..).as_mut_ptr() as *mut __m128i;
            _mm_storel_epi64(dst_ptr, sss0);

            x += 2;
        }

        while x < src_width {
            let mut sss = initial;
            let mut y: u32 = 0;
            for (s_row1, s_row2) in src_img.iter_2_rows(y_start, y_start + y_size) {
                // Load two coefficients at once
                let mmk = simd_utils::ptr_i16_to_set1_epi32(coeffs, y as usize);

                let source1 = simd_utils::mm_cvtsi32_si128(s_row1, x); // top line
                let source2 = simd_utils::mm_cvtsi32_si128(s_row2, x); // bottom line

                let source = _mm_unpacklo_epi8(source1, source2);
                let pix = _mm_unpacklo_epi8(source, _mm_setzero_si128());
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                y += 2;
            }

            for s_row in src_img.iter_rows(y_start + y, y_start + y_size) {
                let pix = simd_utils::mm_cvtepu8_epi32(s_row, x);
                let mmk = _mm_set1_epi32(*coeffs.get_unchecked(y as usize) as i32);
                sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

                y += 1;
            }

            macro_rules! call {
                ($imm8:expr) => {{
                    sss = _mm_srai_epi32::<$imm8>(sss);
                }};
            }
            constify_imm8!(precision, call);

            sss = _mm_packs_epi32(sss, sss);
            *dst_row.get_unchecked_mut(x) =
                transmute(_mm_cvtsi128_si32(_mm_packus_epi16(sss, sss)));

            x += 1;
        }
    }
}

impl Convolution for Avx2U8x4 {
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
