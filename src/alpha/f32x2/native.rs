use num_traits::Zero;

use crate::pixels::F32x2;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

pub(crate) fn multiply_alpha(
    src_view: &impl ImageView<Pixel = F32x2>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components: [f32; 2] = src_pixel.0;
        let alpha = components[1];
        dst_pixel.0 = [components[0] * alpha, alpha];
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row_inplace(row: &mut [F32x2]) {
    for pixel in row {
        pixel.0[0] *= pixel.0[1];
    }
}

// Divide

#[inline]
pub(crate) fn divide_alpha(
    src_view: &impl ImageView<Pixel = F32x2>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x2>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline]
pub(crate) fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x2>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn divide_alpha_row(src_row: &[F32x2], dst_row: &mut [F32x2]) {
    foreach_with_pre_reading(
        src_row.iter().zip(dst_row),
        |(&src_pixel, dst_pixel)| (src_pixel, dst_pixel),
        |(src_pixel, dst_pixel)| {
            let alpha = src_pixel.0[1];
            if alpha.is_zero() {
                dst_pixel.0 = [0.; 2];
            } else {
                dst_pixel.0 = [src_pixel.0[0] / alpha, alpha];
            }
        },
    );
}

#[inline(always)]
pub(crate) fn divide_alpha_row_inplace(row: &mut [F32x2]) {
    for pixel in row {
        let components: [f32; 2] = pixel.0;
        let alpha = components[1];
        if alpha.is_zero() {
            pixel.0[0] = 0.;
        } else {
            pixel.0[0] = components[0] / alpha;
        }
    }
}
