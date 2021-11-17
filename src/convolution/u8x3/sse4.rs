use crate::convolution::Coefficients;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x3;

use super::native;

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn horiz_convolution(
    src_image: TypedImageView<U8x3>,
    dst_image: TypedImageViewMut<U8x3>,
    offset: u32,
    coeffs: Coefficients,
) {
    native::horiz_convolution(src_image, dst_image, offset, coeffs);
}

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn vert_convolution(
    src_image: TypedImageView<U8x3>,
    dst_image: TypedImageViewMut<U8x3>,
    coeffs: Coefficients,
) {
    native::vert_convolution(src_image, dst_image, coeffs);
}
