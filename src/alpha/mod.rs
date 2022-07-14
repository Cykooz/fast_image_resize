pub use errors::*;

use crate::pixels::Pixel;
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};
use crate::CpuExtensions;

mod common;
pub(crate) mod errors;
mod u16x2;
mod u16x4;
mod u8x2;
mod u8x4;

pub(crate) trait AlphaMulDiv
where
    Self: Pixel,
{
    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    fn multiply_alpha(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    );

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    fn multiply_alpha_inplace(image: TypedImageViewMut<Self>, cpu_extensions: CpuExtensions);

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    fn divide_alpha(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    );

    /// Divides RGB-channels of image by alpha-channel inplace.
    fn divide_alpha_inplace(image: TypedImageViewMut<Self>, cpu_extensions: CpuExtensions);
}
