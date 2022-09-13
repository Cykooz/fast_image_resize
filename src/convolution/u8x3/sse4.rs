use crate::convolution::Coefficients;
use crate::pixels::U8x3;
use crate::{ImageView, ImageViewMut};

use super::native;

#[target_feature(enable = "sse4.1")]
pub(crate) unsafe fn horiz_convolution(
    src_image: &ImageView<U8x3>,
    dst_image: &mut ImageViewMut<U8x3>,
    offset: u32,
    coeffs: Coefficients,
) {
    native::horiz_convolution(src_image, dst_image, offset, coeffs);
}
