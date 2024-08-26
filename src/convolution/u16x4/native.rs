use crate::convolution::optimisations::Normalizer32;
use crate::pixels::U16x4;
use crate::{ImageView, ImageViewMut};

#[inline(always)]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U16x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U16x4>,
    offset: u32,
    normalizer: &Normalizer32,
) {
    let precision = normalizer.precision();
    let coefficients_chunks = normalizer.chunks();
    let initial: i64 = 1 << (precision - 1);

    let src_rows = src_view.iter_rows(offset);
    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (coeffs_chunk, dst_pixel) in coefficients_chunks.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start as usize;
            let mut ss = [initial; 4];
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, src_pixel) in coeffs_chunk.values().iter().zip(src_pixels) {
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
