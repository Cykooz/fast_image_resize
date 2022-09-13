use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U16x2;
use crate::{ImageView, ImageViewMut};

#[inline(always)]
pub(crate) fn horiz_convolution(
    src_image: &ImageView<U16x2>,
    dst_image: &mut ImageViewMut<U16x2>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let precision = normalizer.precision();
    let coefficients_chunks = normalizer.normalized_chunks();
    let initial: i64 = 1 << (precision - 1);

    let src_rows = src_image.iter_rows(offset);
    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (&coeffs_chunk, dst_pixel) in coefficients_chunks.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start as usize;
            let mut ss = [initial; 2];
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, src_pixel) in coeffs_chunk.values.iter().zip(src_pixels) {
                for (i, s) in ss.iter_mut().enumerate() {
                    *s += src_pixel.0[i] as i64 * (k as i64);
                }
            }
            for (i, s) in ss.iter().copied().enumerate() {
                dst_pixel.0[i] = normalizer.clip(s);
            }
        }
    }
}
