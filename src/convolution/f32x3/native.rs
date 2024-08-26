use crate::convolution::Coefficients;
use crate::pixels::F32x3;
use crate::{ImageView, ImageViewMut};

pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = F32x3>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x3>,
    offset: u32,
    coeffs: &Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let src_rows = src_view.iter_rows(offset);
    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (dst_pixel, coeffs_chunk) in dst_row.iter_mut().zip(&coefficients_chunks) {
            let first_x_src = coeffs_chunk.start as usize;
            let mut ss = [0.; 3];
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, &src_pixel) in coeffs_chunk.values.iter().zip(src_pixels) {
                for (s, c) in ss.iter_mut().zip(src_pixel.0) {
                    *s += c as f64 * k;
                }
            }
            dst_pixel.0 = ss.map(|v| v as f32);
        }
    }
}
