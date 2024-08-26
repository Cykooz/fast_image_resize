use crate::convolution::optimisations::Normalizer16;
use crate::pixels::U8x3;
use crate::{ImageView, ImageViewMut};

#[inline(always)]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U8x3>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x3>,
    offset: u32,
    normalizer: &Normalizer16,
) {
    let precision = normalizer.precision();
    let coefficients = normalizer.chunks();
    let initial = 1i32 << (precision - 1);

    let src_rows = src_view.iter_rows(offset);
    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (coeffs_chunk, dst_pixel) in coefficients.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start as usize;
            let mut ss = [initial; 3];
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, src_pixel) in coeffs_chunk.values().iter().zip(src_pixels) {
                for (s, c) in ss.iter_mut().zip(src_pixel.0) {
                    *s += c as i32 * (k as i32);
                }
            }
            dst_pixel.0 = ss.map(|v| unsafe { normalizer.clip(v) });
        }
    }
}
