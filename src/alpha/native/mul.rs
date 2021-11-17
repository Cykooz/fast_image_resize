use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

pub(crate) fn multiply_alpha_native(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        multiply_alpha_row_native(src_row, dst_row);
    }
}

pub(crate) fn multiply_alpha_inplace_native(mut image: TypedImageViewMut<U8x4>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = unsafe { std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len()) };
        multiply_alpha_row_native(src_row, dst_row);
    }
}

#[inline(always)]
pub(crate) fn multiply_alpha_row_native(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components: [u8; 4] = src_pixel.0.to_le_bytes();
        let alpha = components[3];
        dst_pixel.0 = u32::from_le_bytes([
            mul_div_255(components[0], alpha),
            mul_div_255(components[1], alpha),
            mul_div_255(components[2], alpha),
            alpha,
        ]);
    }
}

#[inline(always)]
pub(crate) fn mul_div_255(a: u8, b: u8) -> u8 {
    let tmp = a as u32 * b as u32 + 128;
    (((tmp >> 8) + tmp) >> 8) as u8
}
