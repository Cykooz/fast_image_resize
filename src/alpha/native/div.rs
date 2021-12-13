use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

pub(crate) fn divide_alpha_native(
    src_image: TypedImageView<U8x4>,
    mut dst_image: TypedImageViewMut<U8x4>,
) {
    let src_rows = src_image.iter_rows(0);
    let dst_rows = dst_image.iter_rows_mut();

    for (src_row, dst_row) in src_rows.zip(dst_rows) {
        divide_alpha_row_native(src_row, dst_row);
    }
}

pub(crate) fn divide_alpha_inplace_native(mut image: TypedImageViewMut<U8x4>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = unsafe { std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len()) };
        divide_alpha_row_native(src_row, dst_row);
    }
}

#[inline(always)]
pub(crate) fn div_and_clip(v: u8, recip_alpha: f32) -> u8 {
    let res = v as f32 * recip_alpha;
    res.min(255.) as u8
}

#[inline(always)]
pub(crate) fn divide_alpha_row_native(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    src_row
        .iter()
        .zip(dst_row)
        .for_each(|(src_pixel, dst_pixel)| {
            let components: [u8; 4] = src_pixel.0.to_le_bytes();
            let alpha = components[3];
            let recip_alpha = if alpha == 0 { 0. } else { 255. / alpha as f32 };
            dst_pixel.0 = u32::from_le_bytes([
                div_and_clip(components[0], recip_alpha),
                div_and_clip(components[1], recip_alpha),
                div_and_clip(components[2], recip_alpha),
                alpha,
            ]);
        });
}
