use crate::pixels::F32;
use crate::CpuExtensions;
use crate::{ImageView, ImageViewMut};

use super::{Coefficients, Convolution};

mod native;

impl Convolution for F32 {
    fn horiz_convolution(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        native::horiz_convolution(src_image, dst_image, offset, coeffs);
    }

    fn vert_convolution(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        native::vert_convolution(src_image, dst_image, offset, coeffs);
    }
}
