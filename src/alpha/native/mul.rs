use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

pub(crate) fn multiply_alpha_native(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0, src_image.height().get());
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
pub(crate) fn multiply_alpha_row_native(src_row: &[u32], dst_row: &mut [u32]) {
    for (src_pixel, dst_pixel) in src_row.iter().zip(dst_row) {
        let components: [u8; 4] = src_pixel.to_le_bytes();
        let alpha = components[3];
        let res: [u8; 4] = [
            mul_div_255(components[0], alpha),
            mul_div_255(components[1], alpha),
            mul_div_255(components[2], alpha),
            alpha,
        ];
        *dst_pixel = u32::from_le_bytes(res);
    }
}

#[inline(always)]
pub(crate) fn mul_div_255(a: u8, b: u8) -> u8 {
    let tmp = a as u32 * b as u32 + 128;
    (((tmp >> 8) + tmp) >> 8) as u8
}
