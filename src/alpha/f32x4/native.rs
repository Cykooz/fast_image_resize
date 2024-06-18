use num_traits::Zero;

use crate::pixels::F32x4;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

pub(crate) fn multiply_alpha(
    src_view: &impl ImageView<Pixel = F32x4>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x4>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row(src_row: &[F32x4], dst_row: &mut [F32x4]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components = src_pixel.0;
        let alpha = components[3];
        dst_pixel.0 = [
            components[0] * alpha,
            components[1] * alpha,
            components[2] * alpha,
            alpha,
        ];
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row_inplace(row: &mut [F32x4]) {
    for pixel in row {
        let alpha = pixel.0[3];
        pixel.0[0] *= alpha;
        pixel.0[1] *= alpha;
        pixel.0[2] *= alpha;
    }
}

// Divide

#[inline]
pub(crate) fn divide_alpha(
    src_view: &impl ImageView<Pixel = F32x4>,
    dst_view: &mut impl ImageViewMut<Pixel = F32x4>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline]
pub(crate) fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = F32x4>) {
    for row in image_view.iter_rows_mut(0) {
        divide_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn divide_alpha_row(src_row: &[F32x4], dst_row: &mut [F32x4]) {
    foreach_with_pre_reading(
        src_row.iter().zip(dst_row),
        |(&src_pixel, dst_pixel)| (src_pixel, dst_pixel),
        |(src_pixel, dst_pixel)| {
            let components = src_pixel.0;
            let alpha = components[3];
            if alpha.is_zero() {
                dst_pixel.0 = [0.; 4];
            } else {
                let recip_alpha = 1. / alpha;
                dst_pixel.0 = [
                    components[0] * recip_alpha,
                    components[1] * recip_alpha,
                    components[2] * recip_alpha,
                    alpha,
                ];
            }
        },
    );
}

#[inline(always)]
pub(crate) fn divide_alpha_row_inplace(row: &mut [F32x4]) {
    for pixel in row {
        let components = pixel.0;
        let alpha = components[3];
        if alpha.is_zero() {
            pixel.0 = [0.; 4];
        } else {
            let recip_alpha = 1. / alpha;
            pixel.0 = [
                components[0] * recip_alpha,
                components[1] * recip_alpha,
                components[2] * recip_alpha,
                alpha,
            ];
        }
    }
}
