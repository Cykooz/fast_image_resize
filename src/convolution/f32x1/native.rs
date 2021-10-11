use crate::convolution::Coefficients;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::F32;

pub(crate) fn horiz_convolution(
    src_image: TypedImageView<F32>,
    mut dst_image: TypedImageViewMut<F32>,
    offset: u32,
    coeffs: Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let mut y_src = offset;

    for out_row in dst_image.iter_rows_mut() {
        for (out_pixel, coeffs_chunk) in out_row.iter_mut().zip(&coefficients_chunks) {
            let first_x_src = coeffs_chunk.start;
            let mut ss = 0.;
            let pixels = src_image.iter_horiz(first_x_src, y_src);
            for (&k, &pixel) in coeffs_chunk.values.iter().zip(pixels) {
                ss += pixel as f64 * k;
            }
            *out_pixel = ss.round() as f32;
        }
        y_src += 1;
    }
}

pub(crate) fn vert_convolution(
    src_image: TypedImageView<F32>,
    mut dst_image: TypedImageViewMut<F32>,
    coeffs: Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();

    for (out_row, coeffs_chunk) in dst_image.iter_rows_mut().zip(coefficients_chunks) {
        let first_y_src = coeffs_chunk.start;
        for (x_src, out_pixel) in out_row.iter_mut().enumerate() {
            let mut ss = 0.;
            let mut y_src = first_y_src;
            for &k in coeffs_chunk.values.iter() {
                let pixel = src_image.get_pixel(x_src as u32, y_src);
                ss += pixel as f64 * k;
                y_src += 1;
            }
            *out_pixel = ss.round() as f32;
        }
    }
}
