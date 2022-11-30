use crate::alpha::common::{div_and_clip, mul_div_255, RECIP_ALPHA};
use crate::pixels::U8x4;
use crate::{ImageView, ImageViewMut};

pub(crate) fn multiply_alpha(src_image: &ImageView<U8x4>, dst_image: &mut ImageViewMut<U8x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row.iter_mut()) {
            *dst_pixel = multiply_alpha_pixel(*src_pixel);
        }
    }
}

pub(crate) fn multiply_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
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
    let components: [u8; 4] = pixel.0.to_le_bytes();
    let alpha = components[3];
    pixel.0 = u32::from_le_bytes([
        mul_div_255(components[0], alpha),
        mul_div_255(components[1], alpha),
        mul_div_255(components[2], alpha),
        alpha,
    ]);
    pixel
}

// Divide

#[inline]
pub(crate) fn divide_alpha(src_image: &ImageView<U8x4>, dst_image: &mut ImageViewMut<U8x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline]
pub(crate) fn divide_alpha_inplace(image: &mut ImageViewMut<U8x4>) {
    for row in image.iter_rows_mut() {
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
    let components: [u8; 4] = pixel.0.to_le_bytes();
    let alpha = components[3];
    let recip_alpha = RECIP_ALPHA[alpha as usize];
    pixel.0 = u32::from_le_bytes([
        div_and_clip(components[0], recip_alpha),
        div_and_clip(components[1], recip_alpha),
        div_and_clip(components[2], recip_alpha),
        alpha,
    ]);
    pixel
}
