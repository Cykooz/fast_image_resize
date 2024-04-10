use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{ImageView, ImageViewMut};
use crate::pixels::U8x4;

#[inline(always)]
pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = U8x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x4>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let precision = normalizer.precision();
    let coefficients_chunks = normalizer.normalized_chunks();
    let initial = 1 << (precision - 1);

    let src_rows = src_view.iter_rows(offset);
    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (&coeffs_chunk, dst_pixel) in coefficients_chunks.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start as usize;
            let mut ss = [initial; 4];
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, &src_pixel) in coeffs_chunk.values.iter().zip(src_pixels) {
                for (i, s) in ss.iter_mut().enumerate() {
                    *s += src_pixel.0[i] as i32 * (k as i32);
                }
            }
            dst_pixel.0 = ss.map(|v| unsafe { normalizer.clip(v) });
        }
    }
}
