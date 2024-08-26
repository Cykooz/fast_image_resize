use crate::convolution::Coefficients;
use crate::pixels::F32;
use crate::{ImageView, ImageViewMut};

pub(crate) fn horiz_convolution(
    src_view: &impl ImageView<Pixel = F32>,
    dst_view: &mut impl ImageViewMut<Pixel = F32>,
    offset: u32,
    coeffs: &Coefficients,
) {
    let coefficients_chunks = coeffs.get_chunks();
    let src_rows = src_view.iter_rows(offset);
    let dst_rows = dst_view.iter_rows_mut(0);
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (dst_pixel, coeffs_chunk) in dst_row.iter_mut().zip(&coefficients_chunks) {
            let first_x_src = coeffs_chunk.start as usize;
            let end_x_src = first_x_src + coeffs_chunk.values.len();
            let mut ss = 0.;
            let mut src_pixels = unsafe { src_row.get_unchecked(first_x_src..end_x_src) };
            let mut coefs = coeffs_chunk.values;

            (coefs, src_pixels) = convolution_by_chunks::<8>(coefs, src_pixels, &mut ss);

            for (&k, &pixel) in coefs.iter().zip(src_pixels) {
                ss += pixel.0 as f64 * k;
            }
            dst_pixel.0 = ss as f32;
        }
    }
}

#[inline(always)]
fn convolution_by_chunks<'a, 'b, const CHUNK_SIZE: usize>(
    coefs: &'a [f64],
    src_pixels: &'b [F32],
    ss: &mut f64,
) -> (&'a [f64], &'b [F32]) {
    let coef_chunks = coefs.chunks_exact(CHUNK_SIZE);
    let coefs = coef_chunks.remainder();
    let pixel_chunks = src_pixels.chunks_exact(CHUNK_SIZE);
    let src_pixels = pixel_chunks.remainder();
    for (ks, pixels) in coef_chunks.zip(pixel_chunks) {
        for (&k, &pixel) in ks.iter().zip(pixels) {
            *ss += pixel.0 as f64 * k;
        }
    }
    (coefs, src_pixels)
}
