use crate::convolution::Coefficients;
use crate::pixels::F32;
use crate::{ImageView, ImageViewMut};

pub(crate) fn horiz_convolution(
    src_image: &ImageView<F32>,
    dst_image: &mut ImageViewMut<F32>,
    offset: u32,
    coeffs: Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let src_rows = src_image.iter_rows(offset);
    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (dst_pixel, coeffs_chunk) in dst_row.iter_mut().zip(&coefficients_chunks) {
            let first_x_src = coeffs_chunk.start as usize;
            let mut ss = 0.;
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, &pixel) in coeffs_chunk.values.iter().zip(src_pixels) {
                ss += pixel.0 as f64 * k;
            }
            dst_pixel.0 = ss.round() as f32;
        }
    }
}

pub(crate) fn vert_convolution(
    src_image: &ImageView<F32>,
    dst_image: &mut ImageViewMut<F32>,
    offset: u32,
    coeffs: Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let dst_rows = dst_image.iter_rows_mut();
    let start_src_x = offset as usize;
    for (&coeffs_chunk, dst_row) in coefficients_chunks.iter().zip(dst_rows) {
        let first_y_src = coeffs_chunk.start;
        let mut src_x = start_src_x;
        for dst_pixel in dst_row.iter_mut() {
            let mut ss = 0.;
            let src_rows = src_image.iter_rows(first_y_src);
            for (src_row, &k) in src_rows.zip(coeffs_chunk.values) {
                let src_pixel = unsafe { src_row.get_unchecked(src_x) };
                ss += src_pixel.0 as f64 * k;
            }
            dst_pixel.0 = ss.round() as f32;
            src_x += 1;
        }
    }
}
