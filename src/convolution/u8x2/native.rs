use crate::convolution::{optimisations, Coefficients};
use crate::pixels::U8x2;
use crate::{ImageView, ImageViewMut};

pub(crate) fn horiz_convolution(
    src_image: &ImageView<U8x2>,
    dst_image: &mut ImageViewMut<U8x2>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer16::new(coeffs);
    let precision = normalizer.precision();
    let coefficients_chunks = normalizer.normalized_chunks();
    let initial = 1 << (precision - 1);

    let src_rows = src_image.iter_rows(offset);
    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (&coeffs_chunk, dst_pixel) in coefficients_chunks.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start as usize;
            let ks = coeffs_chunk.values;
            let mut ss = [initial; 2];
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, &src_pixel) in ks.iter().zip(src_pixels) {
                let components: [u8; 2] = src_pixel.0.to_le_bytes();
                for (i, s) in ss.iter_mut().enumerate() {
                    *s += components[i] as i32 * (k as i32);
                }
            }
            dst_pixel.0 = u16::from_le_bytes(ss.map(|v| unsafe { normalizer.clip(v) }));
        }
    }
}
