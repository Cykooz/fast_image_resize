use crate::alpha::common::{div_and_clip, mul_div_255, RECIP_ALPHA};
use crate::pixels::U8x4;
use crate::{ImageView, ImageViewMut};

pub(crate) fn multiply_alpha(
    src_view: &impl ImageView<Pixel = U8x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x4>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);
    let rows = src_rows.zip(dst_rows);

    for (src_row, dst_row) in rows {
        for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row.iter_mut()) {
            *dst_pixel = multiply_alpha_pixel(*src_pixel);
        }
    }
}

pub(crate) fn multiply_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = U8x4>) {
    let rows = image_view.iter_rows_mut(0);
    for row in rows {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        *dst_pixel = multiply_alpha_pixel(*src_pixel);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row_inplace(row: &mut [U8x4]) {
    for pixel in row.iter_mut() {
        *pixel = multiply_alpha_pixel(*pixel);
    }
}

#[inline(always)]
fn multiply_alpha_pixel(mut pixel: U8x4) -> U8x4 {
    let alpha = pixel.0[3];
    pixel.0 = [
        mul_div_255(pixel.0[0], alpha),
        mul_div_255(pixel.0[1], alpha),
        mul_div_255(pixel.0[2], alpha),
        alpha,
    ];
    pixel
}

// Divide

#[inline]
pub(crate) fn divide_alpha(
    src_view: &impl ImageView<Pixel = U8x4>,
    dst_view: &mut impl ImageViewMut<Pixel = U8x4>,
) {
    let src_rows = src_view.iter_rows(0);
    let dst_rows = dst_view.iter_rows_mut(0);
    let rows = src_rows.zip(dst_rows);
    for (src_row, dst_row) in rows {
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline]
pub(crate) fn divide_alpha_inplace(image_view: &mut impl ImageViewMut<Pixel = U8x4>) {
    let rows = image_view.iter_rows_mut(0);
    for row in rows {
        row.iter_mut().for_each(|pixel| {
            *pixel = divide_alpha_pixel(*pixel);
        });
    }
}

#[inline(always)]
pub(crate) fn divide_alpha_row(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        *dst_pixel = divide_alpha_pixel(*src_pixel);
    }
}

#[inline(always)]
fn divide_alpha_pixel(mut pixel: U8x4) -> U8x4 {
    let alpha = pixel.0[3];
    let recip_alpha = RECIP_ALPHA[alpha as usize];
    pixel.0 = [
        div_and_clip(pixel.0[0], recip_alpha),
        div_and_clip(pixel.0[1], recip_alpha),
        div_and_clip(pixel.0[2], recip_alpha),
        alpha,
    ];
    pixel
}
