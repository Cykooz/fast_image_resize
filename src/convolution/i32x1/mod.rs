use crate::pixels::I32;
use crate::{CpuExtensions, ImageView, ImageViewMut};

use super::{Coefficients, Convolution};

mod native;

impl Convolution for I32 {
    fn horiz_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        native::horiz_convolution(src_view, dst_view, offset, coeffs);
    }

    fn vert_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        native::vert_convolution(src_view, dst_view, offset, coeffs);
    }
}
