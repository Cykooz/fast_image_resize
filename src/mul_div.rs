use crate::alpha::AlphaMulDiv;
use crate::{
    CpuExtensions, DynamicImageView, DynamicImageViewMut, MulDivImageError, MulDivImagesError,
};
use crate::{ImageView, ImageViewMut};

/// Methods of this structure used to multiply or divide color-channels (RGB or Luma)
/// by alpha-channel. Supported pixel types: U8x2, U8x4, U16x2 and U16x4.
///
/// By default, instance of `MulDiv` created with best CPU-extensions provided by your CPU.
/// You can change this by use method [MulDiv::set_cpu_extensions].
///
/// # Examples
///
/// ```
/// use std::num::NonZeroU32;
/// use fast_image_resize::pixels::PixelType;
/// use fast_image_resize::{Image, MulDiv};
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

    /// Multiplies color-channels (RGB or Luma) of source image by alpha-channel and store
    /// result into destination image.
    pub fn multiply_alpha(
        &self,
        src_image: &DynamicImageView,
        dst_image: &mut DynamicImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        match (src_image, dst_image) {
            (
                DynamicImageView::U8x2(typed_src_image),
                DynamicImageViewMut::U8x2(typed_dst_image),
            ) => multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            (
                DynamicImageView::U8x4(typed_src_image),
                DynamicImageViewMut::U8x4(typed_dst_image),
            ) => multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            (
                DynamicImageView::U16x2(typed_src_image),
                DynamicImageViewMut::U16x2(typed_dst_image),
            ) => multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            (
                DynamicImageView::U16x4(typed_src_image),
                DynamicImageViewMut::U16x4(typed_dst_image),
            ) => multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            _ => Err(MulDivImagesError::UnsupportedPixelType),
        }
    }

    /// Multiplies color-channels (RGB or Luma) of image by alpha-channel inplace.
    pub fn multiply_alpha_inplace(
        &self,
        image: &mut DynamicImageViewMut,
    ) -> Result<(), MulDivImageError> {
        match image {
            DynamicImageViewMut::U8x2(typed_image) => {
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            DynamicImageViewMut::U8x4(typed_image) => {
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            DynamicImageViewMut::U16x2(typed_image) => {
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            DynamicImageViewMut::U16x4(typed_image) => {
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            _ => Err(MulDivImageError::UnsupportedPixelType),
        }
    }

    /// Divides color-channels (RGB or Luma) of source image by alpha-channel and store
    /// result into destination image.
    pub fn divide_alpha(
        &self,
        src_image: &DynamicImageView,
        dst_image: &mut DynamicImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        match (src_image, dst_image) {
            (
                DynamicImageView::U8x2(typed_src_image),
                DynamicImageViewMut::U8x2(typed_dst_image),
            ) => divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            (
                DynamicImageView::U8x4(typed_src_image),
                DynamicImageViewMut::U8x4(typed_dst_image),
            ) => divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            (
                DynamicImageView::U16x2(typed_src_image),
                DynamicImageViewMut::U16x2(typed_dst_image),
            ) => divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            (
                DynamicImageView::U16x4(typed_src_image),
                DynamicImageViewMut::U16x4(typed_dst_image),
            ) => divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions),
            _ => Err(MulDivImagesError::UnsupportedPixelType),
        }
    }

    /// Divides color-channels (RGB or Luma) of image by alpha-channel inplace.
    pub fn divide_alpha_inplace(
        &self,
        image: &mut DynamicImageViewMut,
    ) -> Result<(), MulDivImageError> {
        match image {
            DynamicImageViewMut::U8x2(typed_image) => {
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            DynamicImageViewMut::U8x4(typed_image) => {
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            DynamicImageViewMut::U16x2(typed_image) => {
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            DynamicImageViewMut::U16x4(typed_image) => {
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            _ => Err(MulDivImageError::UnsupportedPixelType),
        }
    }
}

fn multiply_alpha<P>(
    src_image: &ImageView<P>,
    dst_image: &mut ImageViewMut<P>,
    cpu_extensions: CpuExtensions,
) -> Result<(), MulDivImagesError>
where
    P: AlphaMulDiv,
{
    if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    P::multiply_alpha(src_image, dst_image, cpu_extensions);
    Ok(())
}

fn multiply_alpha_inplace<P>(image: &mut ImageViewMut<P>, cpu_extensions: CpuExtensions)
where
    P: AlphaMulDiv,
{
    P::multiply_alpha_inplace(image, cpu_extensions)
}

fn divide_alpha<P>(
    src_image: &ImageView<P>,
    dst_image: &mut ImageViewMut<P>,
    cpu_extensions: CpuExtensions,
) -> Result<(), MulDivImagesError>
where
    P: AlphaMulDiv,
{
    if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    P::divide_alpha(src_image, dst_image, cpu_extensions);
    Ok(())
}

fn divide_alpha_inplace<P>(image: &mut ImageViewMut<P>, cpu_extensions: CpuExtensions)
where
    P: AlphaMulDiv,
{
    P::divide_alpha_inplace(image, cpu_extensions)
}
