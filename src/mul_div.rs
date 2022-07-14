use crate::alpha::AlphaMulDiv;
use crate::pixels::{U16x2, U16x4, U8x2, U8x4};
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};
use crate::{
    CpuExtensions, ImageView, ImageViewMut, MulDivImageError, MulDivImagesError, PixelType,
};

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
        src_image: &ImageView,
        dst_image: &mut ImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        match src_image.pixel_type() {
            PixelType::U8x2 => {
                let (typed_src_image, typed_dst_image) = assert_images_u8x2(src_image, dst_image)?;
                multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U8x4 => {
                let (typed_src_image, typed_dst_image) = assert_images_u8x4(src_image, dst_image)?;
                multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x2 => {
                let (typed_src_image, typed_dst_image) = assert_images_u16x2(src_image, dst_image)?;
                multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x4 => {
                let (typed_src_image, typed_dst_image) = assert_images_u16x4(src_image, dst_image)?;
                multiply_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }

            _ => Err(MulDivImagesError::UnsupportedPixelType),
        }
    }

    /// Multiplies color-channels (RGB or Luma) of image by alpha-channel inplace.
    pub fn multiply_alpha_inplace(&self, image: &mut ImageViewMut) -> Result<(), MulDivImageError> {
        match image.pixel_type() {
            PixelType::U8x2 => {
                let typed_image = assert_image_u8x2(image)?;
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U8x4 => {
                let typed_image = assert_image_u8x4(image)?;
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x2 => {
                let typed_image = assert_image_u16x2(image)?;
                multiply_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x4 => {
                let typed_image = assert_image_u16x4(image)?;
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
        src_image: &ImageView,
        dst_image: &mut ImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        match src_image.pixel_type() {
            PixelType::U8x2 => {
                let (typed_src_image, typed_dst_image) = assert_images_u8x2(src_image, dst_image)?;
                divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U8x4 => {
                let (typed_src_image, typed_dst_image) = assert_images_u8x4(src_image, dst_image)?;
                divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x2 => {
                let (typed_src_image, typed_dst_image) = assert_images_u16x2(src_image, dst_image)?;
                divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x4 => {
                let (typed_src_image, typed_dst_image) = assert_images_u16x4(src_image, dst_image)?;
                divide_alpha(typed_src_image, typed_dst_image, self.cpu_extensions);
                Ok(())
            }
            _ => Err(MulDivImagesError::UnsupportedPixelType),
        }
    }

    /// Divides color-channels (RGB or Luma) of image by alpha-channel inplace.
    pub fn divide_alpha_inplace(&self, image: &mut ImageViewMut) -> Result<(), MulDivImageError> {
        match image.pixel_type() {
            PixelType::U8x2 => {
                let typed_image = assert_image_u8x2(image)?;
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U8x4 => {
                let typed_image = assert_image_u8x4(image)?;
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x2 => {
                let typed_image = assert_image_u16x2(image)?;
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            PixelType::U16x4 => {
                let typed_image = assert_image_u16x4(image)?;
                divide_alpha_inplace(typed_image, self.cpu_extensions);
                Ok(())
            }
            _ => Err(MulDivImageError::UnsupportedPixelType),
        }
    }
}

#[inline]
fn assert_images_u8x2<'s, 'd, 'da>(
    src_image: &'s ImageView<'s>,
    dst_image: &'d mut ImageViewMut<'da>,
) -> Result<
    (
        TypedImageView<'s, 's, U8x2>,
        TypedImageViewMut<'d, 'da, U8x2>,
    ),
    MulDivImagesError,
> {
    let src_image_u8x2 = TypedImageView::from_image_view(src_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    let dst_image_u8x2 = TypedImageViewMut::from_image_view(dst_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    if src_image_u8x2.width() != dst_image_u8x2.width()
        || src_image_u8x2.height() != dst_image_u8x2.height()
    {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    Ok((src_image_u8x2, dst_image_u8x2))
}

#[inline]
fn assert_image_u8x2<'a, 'b>(
    image: &'a mut ImageViewMut<'b>,
) -> Result<TypedImageViewMut<'a, 'b, U8x2>, MulDivImageError> {
    TypedImageViewMut::from_image_view(image).ok_or(MulDivImageError::UnsupportedPixelType)
}

#[inline]
fn assert_images_u8x4<'s, 'd, 'da>(
    src_image: &'s ImageView<'s>,
    dst_image: &'d mut ImageViewMut<'da>,
) -> Result<
    (
        TypedImageView<'s, 's, U8x4>,
        TypedImageViewMut<'d, 'da, U8x4>,
    ),
    MulDivImagesError,
> {
    let src_image_u8x4 = TypedImageView::from_image_view(src_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    let dst_image_u8x4 = TypedImageViewMut::from_image_view(dst_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    if src_image_u8x4.width() != dst_image_u8x4.width()
        || src_image_u8x4.height() != dst_image_u8x4.height()
    {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    Ok((src_image_u8x4, dst_image_u8x4))
}

#[inline]
fn assert_image_u8x4<'a, 'b>(
    image: &'a mut ImageViewMut<'b>,
) -> Result<TypedImageViewMut<'a, 'b, U8x4>, MulDivImageError> {
    TypedImageViewMut::from_image_view(image).ok_or(MulDivImageError::UnsupportedPixelType)
}

#[inline]
fn assert_images_u16x2<'s, 'd, 'da>(
    src_image: &'s ImageView<'s>,
    dst_image: &'d mut ImageViewMut<'da>,
) -> Result<
    (
        TypedImageView<'s, 's, U16x2>,
        TypedImageViewMut<'d, 'da, U16x2>,
    ),
    MulDivImagesError,
> {
    let src_image_u16x2 = TypedImageView::from_image_view(src_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    let dst_image_u16x2 = TypedImageViewMut::from_image_view(dst_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    if src_image_u16x2.width() != dst_image_u16x2.width()
        || src_image_u16x2.height() != dst_image_u16x2.height()
    {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    Ok((src_image_u16x2, dst_image_u16x2))
}

#[inline]
fn assert_image_u16x2<'a, 'b>(
    image: &'a mut ImageViewMut<'b>,
) -> Result<TypedImageViewMut<'a, 'b, U16x2>, MulDivImageError> {
    TypedImageViewMut::from_image_view(image).ok_or(MulDivImageError::UnsupportedPixelType)
}

#[inline]
fn assert_images_u16x4<'s, 'd, 'da>(
    src_image: &'s ImageView<'s>,
    dst_image: &'d mut ImageViewMut<'da>,
) -> Result<
    (
        TypedImageView<'s, 's, U16x4>,
        TypedImageViewMut<'d, 'da, U16x4>,
    ),
    MulDivImagesError,
> {
    let src_image_u16x4 = TypedImageView::from_image_view(src_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    let dst_image_u16x4 = TypedImageViewMut::from_image_view(dst_image)
        .ok_or(MulDivImagesError::UnsupportedPixelType)?;
    if src_image_u16x4.width() != dst_image_u16x4.width()
        || src_image_u16x4.height() != dst_image_u16x4.height()
    {
        return Err(MulDivImagesError::SizeIsDifferent);
    }
    Ok((src_image_u16x4, dst_image_u16x4))
}

#[inline]
fn assert_image_u16x4<'a, 'b>(
    image: &'a mut ImageViewMut<'b>,
) -> Result<TypedImageViewMut<'a, 'b, U16x4>, MulDivImageError> {
    TypedImageViewMut::from_image_view(image).ok_or(MulDivImageError::UnsupportedPixelType)
}

fn multiply_alpha<P>(
    src_image: TypedImageView<P>,
    dst_image: TypedImageViewMut<P>,
    cpu_extensions: CpuExtensions,
) where
    P: AlphaMulDiv,
{
    P::multiply_alpha(src_image, dst_image, cpu_extensions)
}

fn multiply_alpha_inplace<P>(image: TypedImageViewMut<P>, cpu_extensions: CpuExtensions)
where
    P: AlphaMulDiv,
{
    P::multiply_alpha_inplace(image, cpu_extensions)
}

fn divide_alpha<P>(
    src_image: TypedImageView<P>,
    dst_image: TypedImageViewMut<P>,
    cpu_extensions: CpuExtensions,
) where
    P: AlphaMulDiv,
{
    P::divide_alpha(src_image, dst_image, cpu_extensions)
}

fn divide_alpha_inplace<P>(image: TypedImageViewMut<P>, cpu_extensions: CpuExtensions)
where
    P: AlphaMulDiv,
{
    P::divide_alpha_inplace(image, cpu_extensions)
}
