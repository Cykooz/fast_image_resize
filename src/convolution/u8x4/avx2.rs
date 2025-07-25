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
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_four_rows<const PRECISION: i32>(
    src_rows: [&[U8x4]; 4],
    dst_rows: [&mut [U8x4]; 4],
    normalizer: &Normalizer16,
) {
    let zero = _mm256_setzero_si256();
    let initial = _mm256_set1_epi32(1 << (PRECISION - 1));

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

    let coefficients_chunks = normalizer.chunks();

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x = coeffs_chunk.start as usize;

        let mut sss0 = initial;
        let mut sss1 = initial;
        let coeffs = coeffs_chunk.values();

        let coeffs_by_4 = coeffs.chunks_exact(4);
        let reminder1 = coeffs_by_4.remainder();

        for k in coeffs_by_4 {
            let mmk0 = simd_utils::mm256_load_and_clone_i16x2(k);
            let mmk1 = simd_utils::mm256_load_and_clone_i16x2(&k[2..]);

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadu_si128(src_rows[0], x)),
                simd_utils::loadu_si128(src_rows[1], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk0));
            let pix = _mm256_shuffle_epi8(source, sh2);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk1));

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadu_si128(src_rows[2], x)),
                simd_utils::loadu_si128(src_rows[3], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk0));
            let pix = _mm256_shuffle_epi8(source, sh2);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk1));

            x += 4;
        }

        let coeffs_by_2 = reminder1.chunks_exact(2);
        let reminder2 = coeffs_by_2.remainder();

        for k in coeffs_by_2 {
            let mmk = simd_utils::mm256_load_and_clone_i16x2(k);

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi64(src_rows[0], x)),
                simd_utils::loadl_epi64(src_rows[1], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

            let source = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::loadl_epi64(src_rows[2], x)),
                simd_utils::loadl_epi64(src_rows[3], x),
            );
            let pix = _mm256_shuffle_epi8(source, sh1);
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));

            x += 2;
        }

        if let Some(&k) = reminder2.first() {
            // [16] xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0 xx k0
            let mmk = _mm256_set1_epi32(k as i32);

            // [16] xx a0 xx b0 xx g0 xx r0 xx a0 xx b0 xx g0 xx r0
            let pix = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::mm_cvtepu8_epi32(src_rows[0], x)),
                simd_utils::mm_cvtepu8_epi32(src_rows[1], x),
            );
            sss0 = _mm256_add_epi32(sss0, _mm256_madd_epi16(pix, mmk));

            let pix = _mm256_inserti128_si256::<1>(
                _mm256_castsi128_si256(simd_utils::mm_cvtepu8_epi32(src_rows[2], x)),
                simd_utils::mm_cvtepu8_epi32(src_rows[3], x),
            );
            sss1 = _mm256_add_epi32(sss1, _mm256_madd_epi16(pix, mmk));
        }

        sss0 = _mm256_srai_epi32::<PRECISION>(sss0);
        sss1 = _mm256_srai_epi32::<PRECISION>(sss1);

        sss0 = _mm256_packs_epi32(sss0, zero);
        sss1 = _mm256_packs_epi32(sss1, zero);
        sss0 = _mm256_packus_epi16(sss0, zero);
        sss1 = _mm256_packus_epi16(sss1, zero);
        *dst_rows[0].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm256_extracti128_si256::<0>(sss0)));
        *dst_rows[1].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm256_extracti128_si256::<1>(sss0)));
        *dst_rows[2].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm256_extracti128_si256::<0>(sss1)));
        *dst_rows[3].get_unchecked_mut(dst_x) =
            transmute::<i32, U8x4>(_mm_cvtsi128_si32(_mm256_extracti128_si256::<1>(sss1)));
    }
}

/// For safety, it is necessary to ensure the following conditions:
/// - bounds.len() == dst_row.len()
/// - coeffs.len() == dst_rows.0.len() * window_size
/// - max(bound.start + bound.size for bound in bounds) <= src_row.len()
/// - precision <= MAX_COEFS_PRECISION
#[inline]
#[target_feature(enable = "avx2")]
unsafe fn horiz_convolution_one_row<const PRECISION: i32>(
    src_row: &[U8x4],
    dst_row: &mut [U8x4],
    normalizer: &Normalizer16,
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

    let coefficients_chunks = normalizer.chunks();

    for (dst_x, coeffs_chunk) in coefficients_chunks.iter().enumerate() {
        let mut x = coeffs_chunk.start as usize;
        let mut coeffs = coeffs_chunk.values();

        let mut sss = if coeffs.len() < 8 {
            _mm_set1_epi32(1 << (PRECISION - 1))
        } else {
            // Lower part will be added to higher, use only half of the error
            let mut sss256 = _mm256_set1_epi32(1 << (PRECISION - 2));

            let coeffs_by_8 = coeffs.chunks_exact(8);
            let reminder1 = coeffs_by_8.remainder();

            for k in coeffs_by_8 {
                let tmp = simd_utils::loadu_si128(k, 0);
                let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                let source = simd_utils::loadu_si256(src_row, x);

                let pix = _mm256_shuffle_epi8(source, sh1);
                let mmk = _mm256_shuffle_epi8(ksource, sh2);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                let pix = _mm256_shuffle_epi8(source, sh3);
                let mmk = _mm256_shuffle_epi8(ksource, sh4);
                sss256 = _mm256_add_epi32(sss256, _mm256_madd_epi16(pix, mmk));

                x += 8;
            }

            let coeffs_by_4 = reminder1.chunks_exact(4);
            coeffs = coeffs_by_4.remainder();

            for k in coeffs_by_4 {
                let tmp = simd_utils::loadl_epi64(k, 0);
                let ksource = _mm256_insertf128_si256::<1>(_mm256_castsi128_si256(tmp), tmp);

                let tmp = simd_utils::loadu_si128(src_row, x);
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
            let mmk = simd_utils::mm_load_and_clone_i16x2(k);
            let source = simd_utils::loadl_epi64(src_row, x);
            let pix = _mm_shuffle_epi8(source, sh7);
            sss = _mm_add_epi32(sss, _mm_madd_epi16(pix, mmk));

            x += 2
        }

        if let Some(&k) = reminder1.first() {
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
