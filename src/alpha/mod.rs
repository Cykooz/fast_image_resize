use thiserror::Error;

use crate::{CpuExtensions, PixelType};
use crate::{DstImageView, SrcImageView};

mod div;
mod mul;

#[derive(Error, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum MulDivImagesError {
    #[error("Size of source image does not match to destination image")]
    SizeIsDifferent,
    #[error("Pixel type of source image does not match to destination image")]
    PixelTypeIsDifferent,
    #[error("Pixel type of image is not supported")]
    UnsupportedPixelType,
}

#[derive(Error, Debug, Clone, Copy)]
#[non_exhaustive]
pub enum MulDivImageError {
    #[error("Pixel type of image is not supported")]
    UnsupportedPixelType,
}

/// Methods of this structure used to multiplies or divides RGB-channels
/// by alpha-channel.
///
/// By default instance of `MulDiv` created with best CPU-extensions provided by your CPU.
/// You can change this by use method [MulDiv::set_cpu_extensions].
///
/// # Examples
///
/// ```
/// use std::num::NonZeroU32;
/// use fast_image_resize::{ImageData, MulDiv, PixelType};
///
/// let width = NonZeroU32::new(10).unwrap();
/// let height = NonZeroU32::new(7).unwrap();
/// let src_image = ImageData::new(width, height, PixelType::U8x4);
/// let mut dst_image = ImageData::new(width, height, PixelType::U8x4);
///
/// let mul_div = MulDiv::default();
/// mul_div.multiply_alpha(&src_image.src_view(), &mut dst_image.dst_view()).unwrap();
/// ```
#[derive(Default, Debug, Clone)]
pub struct MulDiv {
    cpu_extensions: CpuExtensions,
}

impl MulDiv {
    #[inline(always)]
    pub fn cpu_extensions(&self) -> CpuExtensions {
        self.cpu_extensions
    }

    /// # Safety
    /// This is unsafe because this method allows you to set a CPU-extensions
    /// that is not actually supported by your CPU.
    pub unsafe fn set_cpu_extensions(&mut self, extensions: CpuExtensions) {
        self.cpu_extensions = extensions;
    }

    /// Multiplies RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    pub fn multiply_alpha(
        &self,
        src_image: &SrcImageView,
        dst_image: &mut DstImageView,
    ) -> Result<(), MulDivImagesError> {
        self.assert_images(src_image, dst_image)?;
        match self.cpu_extensions {
            CpuExtensions::Avx2 => mul::multiply_alpha_avx2(src_image, dst_image),
            // CpuExtensions::Sse2 => mul::multiply_alpha_sse2(src_image, dst_image),
            _ => mul::multiply_alpha_native(src_image, dst_image),
        }
        Ok(())
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    pub fn multiply_alpha_inplace(&self, image: &mut DstImageView) -> Result<(), MulDivImageError> {
        self.assert_image(image)?;
        match self.cpu_extensions {
            CpuExtensions::Avx2 => mul::multiply_alpha_inplace_avx2(image),
            // CpuExtensions::Sse2 => mul::multiply_alpha_sse2(src_image, dst_image),
            _ => mul::multiply_alpha_inplace_native(image),
        }
        Ok(())
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    pub fn divide_alpha(
        &self,
        src_image: &SrcImageView,
        dst_image: &mut DstImageView,
    ) -> Result<(), MulDivImagesError> {
        self.assert_images(src_image, dst_image)?;
        match self.cpu_extensions {
            CpuExtensions::Avx2 => div::divide_alpha_avx2(src_image, dst_image),
            CpuExtensions::Sse4_1 | CpuExtensions::Sse2 => {
                div::divide_alpha_sse2(src_image, dst_image)
            }
            _ => div::divide_alpha_native(src_image, dst_image),
        }
        Ok(())
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    pub fn divide_alpha_inplace(&self, image: &mut DstImageView) -> Result<(), MulDivImageError> {
        self.assert_image(image)?;
        match self.cpu_extensions {
            CpuExtensions::Avx2 => div::divide_alpha_inplace_avx2(image),
            CpuExtensions::Sse4_1 | CpuExtensions::Sse2 => div::divide_alpha_inplace_sse2(image),
            _ => div::divide_alpha_inplace_native(image),
        }
        Ok(())
    }

    #[inline]
    fn assert_images(
        &self,
        src_image: &SrcImageView,
        dst_image: &DstImageView,
    ) -> Result<(), MulDivImagesError> {
        if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
            return Err(MulDivImagesError::SizeIsDifferent);
        }
        if src_image.pixel_type() != PixelType::U8x4 {
            return Err(MulDivImagesError::UnsupportedPixelType);
        }
        if src_image.pixel_type() != dst_image.pixel_type() {
            return Err(MulDivImagesError::PixelTypeIsDifferent);
        }
        Ok(())
    }

    #[inline]
    fn assert_image(&self, image: &DstImageView) -> Result<(), MulDivImageError> {
        if image.pixel_type() != PixelType::U8x4 {
            return Err(MulDivImageError::UnsupportedPixelType);
        }
        Ok(())
    }
}
