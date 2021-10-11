use super::{Coefficients, Convolution};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8;
use crate::CpuExtensions;

mod native;

impl Convolution for U8 {
    fn horiz_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        native::horiz_convolution(src_image, dst_image, offset, coeffs);
    }

    fn vert_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        native::vert_convolution(src_image, dst_image, coeffs);
    }
}
