use crate::cpu_extensions::CpuExtensions;
use crate::pixels::F32;
use crate::{ImageView, ImageViewMut};

use super::{Coefficients, Convolution};

mod native;

impl Convolution for F32 {
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
