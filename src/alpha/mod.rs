pub use errors::*;

use crate::pixels::PixelExt;
use crate::CpuExtensions;
use crate::{ImageView, ImageViewMut};

mod common;
pub(crate) mod errors;
mod u16x2;
mod u16x4;
mod u8x2;
mod u8x4;

pub(crate) trait AlphaMulDiv
where
    Self: PixelExt,
{
    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    fn multiply_alpha(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    );

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    fn multiply_alpha_inplace(image: &mut ImageViewMut<Self>, cpu_extensions: CpuExtensions);

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    fn divide_alpha(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    );

    /// Divides RGB-channels of image by alpha-channel inplace.
    fn divide_alpha_inplace(image: &mut ImageViewMut<Self>, cpu_extensions: CpuExtensions);
}
