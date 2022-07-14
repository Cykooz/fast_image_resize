use crate::pixels::F32;
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};
use crate::CpuExtensions;

use super::{Coefficients, Convolution};

mod native;

impl Convolution for F32 {
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
