use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8;

pub(crate) fn horiz_convolution(
    src_image: TypedImageView<U8>,
    mut dst_image: TypedImageViewMut<U8>,
    offset: u32,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard::new(values);
    let precision = normalizer_guard.precision();
    let coefficients_chunks = normalizer_guard.normalized_i16_chunks(window_size, &bounds);

    let dst_rows = dst_image.iter_rows_mut();
    for (y_dst, dst_row) in dst_rows.enumerate() {
        let y_src = y_dst as u32 + offset;

        for (&coeffs_chunk, dst_pixel) in coefficients_chunks.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start;
            let ks = coeffs_chunk.values;

            let mut ss0 = 1 << (precision - 1);
            let src_pixels = src_image.iter_horiz(first_x_src, y_src);
            for (&k, &src_pixel) in ks.iter().zip(src_pixels) {
                ss0 += src_pixel as i32 * (k as i32);
            }
            *dst_pixel = unsafe { optimisations::clip8(ss0, precision) };
        }
    }
}

pub(crate) fn vert_convolution(
    src_image: TypedImageView<U8>,
    mut dst_image: TypedImageViewMut<U8>,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard::new(values);
    let precision = normalizer_guard.precision();
    let coefficients_chunks = normalizer_guard.normalized_i16_chunks(window_size, &bounds);

    let dst_rows = dst_image.iter_rows_mut();
    for (&coeffs_chunk, dst_row) in coefficients_chunks.iter().zip(dst_rows) {
        let first_y_src = coeffs_chunk.start;
        let ks = coeffs_chunk.values;

        for (x_src, dst_pixel) in dst_row.iter_mut().enumerate() {
            let mut ss0 = 1 << (precision - 1);
            for (dy, &k) in ks.iter().enumerate() {
                let src_pixel = src_image.get_pixel(x_src as u32, first_y_src + dy as u32);
                ss0 += src_pixel as i32 * (k as i32);
            }
            *dst_pixel = unsafe { optimisations::clip8(ss0, precision) };
        }
    }
}
