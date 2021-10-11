use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

pub(crate) fn horiz_convolution(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
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
            let mut ss1 = ss0;
            let mut ss2 = ss0;
            let mut ss3 = ss0;
            let src_pixels = src_image.iter_horiz(first_x_src, y_src);
            for (&k, &src_pixel) in ks.iter().zip(src_pixels) {
                let components: [u8; 4] = src_pixel.to_le_bytes();
                ss0 += components[0] as i32 * (k as i32);
                ss1 += components[1] as i32 * (k as i32);
                ss2 += components[2] as i32 * (k as i32);
                ss3 += components[3] as i32 * (k as i32);
            }
            let t: [u8; 4] = unsafe {
                [
                    optimisations::clip8(ss0, precision),
                    optimisations::clip8(ss1, precision),
                    optimisations::clip8(ss2, precision),
                    optimisations::clip8(ss3, precision),
                ]
            };
            *dst_pixel = u32::from_le_bytes(t);
        }
    }
}

pub(crate) fn vert_convolution(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
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

        for (x_src, out_pixel) in dst_row.iter_mut().enumerate() {
            let mut ss0 = 1 << (precision - 1);
            let mut ss1 = ss0;
            let mut ss2 = ss0;
            let mut ss3 = ss0;
            for (dy, &k) in ks.iter().enumerate() {
                let pixel = src_image.get_pixel(x_src as u32, first_y_src + dy as u32);
                let components: [u8; 4] = pixel.to_le_bytes();
                ss0 += components[0] as i32 * (k as i32);
                ss1 += components[1] as i32 * (k as i32);
                ss2 += components[2] as i32 * (k as i32);
                ss3 += components[3] as i32 * (k as i32);
            }
            let t: [u8; 4] = unsafe {
                [
                    optimisations::clip8(ss0, precision),
                    optimisations::clip8(ss1, precision),
                    optimisations::clip8(ss2, precision),
                    optimisations::clip8(ss3, precision),
                ]
            };
            *out_pixel = u32::from_le_bytes(t);
        }
    }
}
