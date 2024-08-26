use crate::{pixels, CpuExtensions, ImageError, ImageView, ImageViewMut};

#[macro_use]
mod common;
pub(crate) mod errors;

mod u8x4;
cfg_if::cfg_if! {
    if #[cfg(not(feature = "only_u8x4"))] {
        mod u16x2;
        mod u16x4;
        mod u8x2;
        mod f32x2;
        mod f32x4;
    }
}

pub(crate) trait AlphaMulDiv: pixels::InnerPixel {
    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    #[allow(unused_variables)]
    fn multiply_alpha(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        Err(ImageError::UnsupportedPixelType)
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    #[allow(unused_variables)]
    fn multiply_alpha_inplace(
        image_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        Err(ImageError::UnsupportedPixelType)
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    #[allow(unused_variables)]
    fn divide_alpha(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        Err(ImageError::UnsupportedPixelType)
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    #[allow(unused_variables)]
    fn divide_alpha_inplace(
        image_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        Err(ImageError::UnsupportedPixelType)
    }
}

impl AlphaMulDiv for pixels::U8 {}
impl AlphaMulDiv for pixels::U8x3 {}
impl AlphaMulDiv for pixels::U16 {}
impl AlphaMulDiv for pixels::U16x3 {}
impl AlphaMulDiv for pixels::I32 {}
impl AlphaMulDiv for pixels::F32 {}
impl AlphaMulDiv for pixels::F32x3 {}
