use crate::convolution::Coefficients;
use crate::pixels::U8x3;
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};

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
