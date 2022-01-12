use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;

#[inline]
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

#[inline]
pub(crate) fn divide_alpha_inplace_native(mut image: TypedImageViewMut<U8x4>) {
    for dst_row in image.iter_rows_mut() {
        let src_row = unsafe { std::slice::from_raw_parts(dst_row.as_ptr(), dst_row.len()) };
        divide_alpha_row_native(src_row, dst_row);
    }
}

#[inline(always)]
pub(crate) fn divide_alpha_row_native(src_row: &[U8x4], dst_row: &mut [U8x4]) {
    src_row
        .iter()
        .zip(dst_row)
        .for_each(|(src_pixel, dst_pixel)| {
            let components: [u8; 4] = src_pixel.0.to_le_bytes();
            let alpha = components[3];
            let recip_alpha = RECIP_ALPHA[alpha as usize];
            dst_pixel.0 = u32::from_le_bytes([
                div_and_clip(components[0], recip_alpha),
                div_and_clip(components[1], recip_alpha),
                div_and_clip(components[2], recip_alpha),
                alpha,
            ]);
        });
}

const fn recip_alpha_array(precision: u32) -> [u32; 256] {
    let mut res = [0; 256];
    let scale = 1 << (precision + 1);
    let mut i: usize = 1;
    while i < 256 {
        res[i] = (((255 * scale / i as u32) + 1) >> 1) as u32;
        i += 1;
    }
    res
}

const PRECISION: u32 = 8;

#[inline(always)]
fn div_and_clip(v: u8, recip_alpha: u32) -> u8 {
    ((v as u32 * recip_alpha) >> PRECISION).min(255) as u8
}

const RECIP_ALPHA: [u32; 256] = recip_alpha_array(PRECISION);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recip_alpha_array() {
        for alpha in 0..=255u8 {
            let expected = if alpha == 0 {
                0
            } else {
                let scale = (1 << PRECISION) as f64;
                (255.0 * scale / alpha as f64).round() as u32
            };

            let recip_alpha = RECIP_ALPHA[alpha as usize];
            assert_eq!(expected, recip_alpha, "alpha {}", alpha);
        }
    }

    #[test]
    fn test_div_and_clip() {
        let mut err_sum: i32 = 0;
        for alpha in 0..=255u8 {
            for color in 0..=255u8 {
                let multiplied_color = (color as f64 * alpha as f64 / 255.).round().min(255.) as u8;

                let expected_color = if alpha == 0 {
                    0
                } else {
                    let recip_alpha = 255. / alpha as f64;
                    let res = multiplied_color as f64 * recip_alpha;
                    res.min(255.) as u8
                };

                let recip_alpha = RECIP_ALPHA[alpha as usize];
                let result_color = div_and_clip(multiplied_color, recip_alpha);
                let delta = result_color as i32 - expected_color as i32;
                err_sum += delta.abs();
            }
        }
        assert_eq!(err_sum, 3468);
    }
}
