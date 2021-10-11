use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;
use crate::CpuExtensions;
use crate::{ImageView, ImageViewMut};
pub use errors::*;

#[cfg(target_arch = "x86_64")]
mod avx2;
mod errors;
mod native;
#[cfg(target_arch = "x86_64")]
mod sse2;

/// Methods of this structure used to multiply or divide RGB-channels
/// by alpha-channel.
///
/// By default, instance of `MulDiv` created with best CPU-extensions provided by your CPU.
/// You can change this by use method [MulDiv::set_cpu_extensions].
///
/// # Examples
///
/// ```
/// use std::num::NonZeroU32;
/// use fast_image_resize::{Image, MulDiv, PixelType};
///
/// let width = NonZeroU32::new(10).unwrap();
/// let height = NonZeroU32::new(7).unwrap();
/// let src_image = Image::new(width, height, PixelType::U8x4);
/// let mut dst_image = Image::new(width, height, PixelType::U8x4);
///
/// let mul_div = MulDiv::default();
/// mul_div.multiply_alpha(&src_image.view(), &mut dst_image.view_mut()).unwrap();
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
        src_image: &ImageView,
        dst_image: &mut ImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        let (src_image_u8x4, dst_image_u8x4) = assert_images(src_image, dst_image)?;
        match self.cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::multiply_alpha_avx2(src_image_u8x4, dst_image_u8x4),
            // WARNING: SSE2 implementation is drastically slower than native version
            // #[cfg(target_arch = "x86_64")]
            // CpuExtensions::Sse4_1 | CpuExtensions::Sse2 => {
            //     sse2::multiply_alpha_sse2(src_image, dst_image)
            // }
            _ => native::multiply_alpha_native(src_image_u8x4, dst_image_u8x4),
        }
        Ok(())
    }

    /// Multiplies RGB-channels of image by alpha-channel inplace.
    pub fn multiply_alpha_inplace(&self, image: &mut ImageViewMut) -> Result<(), MulDivImageError> {
        let image_u8x4 = assert_image(image)?;
        match self.cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::multiply_alpha_inplace_avx2(image_u8x4),
            // WARNING: SSE2 implementation is drastically slower than native version
            // #[cfg(target_arch = "x86_64")]
            // CpuExtensions::Sse4_1 | CpuExtensions::Sse2 => {
            //     sse2::multiply_alpha_sse2(src_image, dst_image)
            // }
            _ => native::multiply_alpha_inplace_native(image_u8x4),
        }
        Ok(())
    }

    /// Divides RGB-channels of source image by alpha-channel and store
    /// result into destination image.
    pub fn divide_alpha(
        &self,
        src_image: &ImageView,
        dst_image: &mut ImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        let (src_image_u8x4, dst_image_u8x4) = assert_images(src_image, dst_image)?;
        match self.cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::divide_alpha_avx2(src_image_u8x4, dst_image_u8x4),
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 | CpuExtensions::Sse2 => {
                sse2::divide_alpha_sse2(src_image_u8x4, dst_image_u8x4)
            }
            _ => native::divide_alpha_native(src_image_u8x4, dst_image_u8x4),
        }
        Ok(())
    }

    /// Divides RGB-channels of image by alpha-channel inplace.
    pub fn divide_alpha_inplace(&self, image: &mut ImageViewMut) -> Result<(), MulDivImageError> {
        let image_u8x4 = assert_image(image)?;
        match self.cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::divide_alpha_inplace_avx2(image_u8x4),
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 | CpuExtensions::Sse2 => {
                sse2::divide_alpha_inplace_sse2(image_u8x4)
            }
            _ => native::divide_alpha_inplace_native(image_u8x4),
        }
        Ok(())
    }
}

#[inline]
fn assert_images<'s, 'd, 'da>(
    src_image: &'s ImageView<'s>,
    dst_image: &'d mut ImageViewMut<'da>,
) -> Result<
    (
        TypedImageView<'s, 's, U8x4>,
        TypedImageViewMut<'d, 'da, U8x4>,
    ),
    MulDivImagesError,
> {
    let src_image_u8x4 = src_image
        .u32_image()
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    let dst_image_u8x4 = dst_image
        .u32_image()
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    if src_image_u8x4.width() != dst_image_u8x4.width()
        || src_image_u8x4.height() != dst_image_u8x4.height()
    {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    Ok((src_image_u8x4, dst_image_u8x4))
}

#[inline]
fn assert_image<'a, 'b>(
    image: &'a mut ImageViewMut<'b>,
) -> Result<TypedImageViewMut<'a, 'b, U8x4>, MulDivImageError> {
    image
        .u32_image()
        .ok_or(MulDivImageError::UnsupportedPixelType)
}
