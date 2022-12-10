use crate::alpha::common::{div_and_clip16, mul_div_65535, RECIP_ALPHA16};
use crate::pixels::U16x4;
use crate::{ImageView, ImageViewMut};

pub(crate) fn multiply_alpha(src_image: &ImageView<U16x4>, dst_image: &mut ImageViewMut<U16x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row(src_row, dst_row);
    }
}

pub(crate) fn multiply_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        multiply_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components: [u16; 4] = src_pixel.0;
        let alpha = components[3];
        dst_pixel.0 = [
            mul_div_65535(components[0], alpha),
            mul_div_65535(components[1], alpha),
            mul_div_65535(components[2], alpha),
            alpha,
        ];
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row_inplace(row: &mut [U16x4]) {
    for pixel in row {
        let components: [u16; 4] = pixel.0;
        let alpha = components[3];
        pixel.0 = [
            mul_div_65535(components[0], alpha),
            mul_div_65535(components[1], alpha),
            mul_div_65535(components[2], alpha),
            alpha,
        ];
    }
}

// Divide

#[inline]
pub(crate) fn divide_alpha(src_image: &ImageView<U16x4>, dst_image: &mut ImageViewMut<U16x4>) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row(src_row, dst_row);
    }
}

#[inline]
pub(crate) fn divide_alpha_inplace(image: &mut ImageViewMut<U16x4>) {
    for row in image.iter_rows_mut() {
        divide_alpha_row_inplace(row);
    }
}

#[inline(always)]
pub(crate) fn divide_alpha_row(src_row: &[U16x4], dst_row: &mut [U16x4]) {
    src_row
        .iter()
        .zip(dst_row)
        .for_each(|(src_pixel, dst_pixel)| {
            let components: [u16; 4] = src_pixel.0;
            let alpha = components[3];
            let recip_alpha = RECIP_ALPHA16[alpha as usize];
            dst_pixel.0 = [
                div_and_clip16(components[0], recip_alpha),
                div_and_clip16(components[1], recip_alpha),
                div_and_clip16(components[2], recip_alpha),
                alpha,
            ];
        });
}

#[inline(always)]
pub(crate) fn divide_alpha_row_inplace(row: &mut [U16x4]) {
    row.iter_mut().for_each(|pixel| {
        let components: [u16; 4] = pixel.0;
        let alpha = components[3];
        let recip_alpha = RECIP_ALPHA16[alpha as usize];
        pixel.0 = [
            div_and_clip16(components[0], recip_alpha),
            div_and_clip16(components[1], recip_alpha),
            div_and_clip16(components[2], recip_alpha),
            alpha,
        ];
    });
}
