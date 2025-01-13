use crate::cpu_extensions::CpuExtensions;
use crate::image_view::{try_pixel_type, ImageViewMut, IntoImageView, IntoImageViewMut};
use crate::pixels::{F32x2, F32x4, U16x2, U16x4, U8x2, U8x4};
use crate::{ImageError, ImageView, MulDivImagesError, PixelTrait, PixelType};

/// Methods of this structure used to multiply or divide color-channels (RGB or Luma)
/// by alpha-channel. Supported pixel types: U8x2, U8x4, U16x2, U16x4, F32x2 and F32x4.
///
/// By default, the instance of `MulDiv` created with the best CPU-extension provided by your CPU.
/// You can change this by using method [MulDiv::set_cpu_extensions].
///
/// # Examples
///
/// ```
/// use fast_image_resize::pixels::PixelType;
/// use fast_image_resize::images::Image;
/// use fast_image_resize::MulDiv;
///
/// let width: u32 = 10;
/// let height: u32 = 7;
/// let src_image = Image::new(width, height, PixelType::U8x4);
/// let mut dst_image = Image::new(width, height, PixelType::U8x4);
///
/// let mul_div = MulDiv::new();
/// mul_div.multiply_alpha(&src_image, &mut dst_image).unwrap();
/// ```
#[derive(Default, Debug, Clone)]
pub struct MulDiv {
    cpu_extensions: CpuExtensions,
}

impl MulDiv {
    pub fn new() -> Self {
        Default::default()
    }

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
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        let src_pixel_type = try_pixel_type(src_image)?;
        let dst_pixel_type = try_pixel_type(dst_image)?;
        if src_pixel_type != dst_pixel_type {
            return Err(MulDivImagesError::PixelTypesAreDifferent);
        }

        #[cfg(not(feature = "only_u8x4"))]
        match src_pixel_type {
            PixelType::U8x2 => self.multiply::<U8x2>(src_image, dst_image),
            PixelType::U8x4 => self.multiply::<U8x4>(src_image, dst_image),
            PixelType::U16x2 => self.multiply::<U16x2>(src_image, dst_image),
            PixelType::U16x4 => self.multiply::<U16x4>(src_image, dst_image),
            PixelType::F32x2 => self.multiply::<F32x2>(src_image, dst_image),
            PixelType::F32x4 => self.multiply::<F32x4>(src_image, dst_image),
            _ => Err(MulDivImagesError::ImageError(
                ImageError::UnsupportedPixelType,
            )),
        }

        #[cfg(feature = "only_u8x4")]
        match src_pixel_type {
            PixelType::U8x4 => self.multiply::<U8x4>(src_image, dst_image),
            _ => Err(MulDivImagesError::ImageError(
                ImageError::UnsupportedPixelType,
            )),
        }
    }

    #[inline]
    fn multiply<P: PixelTrait>(
        &self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        match (src_image.image_view(), dst_image.image_view_mut()) {
            (Some(src), Some(mut dst)) => self.multiply_alpha_typed::<P>(&src, &mut dst),
            _ => Err(MulDivImagesError::ImageError(
                ImageError::UnsupportedPixelType,
            )),
        }
    }

    pub fn multiply_alpha_typed<P: PixelTrait>(
        &self,
        src_view: &impl ImageView<Pixel = P>,
        dst_view: &mut impl ImageViewMut<Pixel = P>,
    ) -> Result<(), MulDivImagesError> {
        if src_view.width() != dst_view.width() || src_view.height() != dst_view.height() {
            return Err(MulDivImagesError::SizeIsDifferent);
        }
        if src_view.width() > 0 && src_view.height() > 0 {
            P::multiply_alpha(src_view, dst_view, self.cpu_extensions)?;
        }
        Ok(())
    }

    /// Multiplies color-channels (RGB or Luma) of image by alpha-channel inplace.
    pub fn multiply_alpha_inplace(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), ImageError> {
        let pixel_type = try_pixel_type(image)?;

        #[cfg(not(feature = "only_u8x4"))]
        match pixel_type {
            PixelType::U8x2 => self.multiply_inplace::<U8x2>(image),
            PixelType::U8x4 => self.multiply_inplace::<U8x4>(image),
            PixelType::U16x2 => self.multiply_inplace::<U16x2>(image),
            PixelType::U16x4 => self.multiply_inplace::<U16x4>(image),
            PixelType::F32x2 => self.multiply_inplace::<F32x2>(image),
            PixelType::F32x4 => self.multiply_inplace::<F32x4>(image),
            _ => Err(ImageError::UnsupportedPixelType),
        }

        #[cfg(feature = "only_u8x4")]
        match pixel_type {
            PixelType::U8x4 => self.multiply_inplace::<U8x4>(image),
            _ => Err(ImageError::UnsupportedPixelType),
        }
    }

    #[inline]
    fn multiply_inplace<P: PixelTrait>(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), ImageError> {
        match image.image_view_mut() {
            Some(mut view) => self.multiply_alpha_inplace_typed::<P>(&mut view),
            _ => Err(ImageError::UnsupportedPixelType),
        }
    }

    pub fn multiply_alpha_inplace_typed<P: PixelTrait>(
        &self,
        img_view: &mut impl ImageViewMut<Pixel = P>,
    ) -> Result<(), ImageError> {
        if img_view.width() > 0 && img_view.height() > 0 {
            P::multiply_alpha_inplace(img_view, self.cpu_extensions)
        } else {
            Ok(())
        }
    }

    /// Divides color-channels (RGB or Luma) of source image by alpha-channel and store
    /// result into destination image.
    pub fn divide_alpha(
        &self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        let src_pixel_type = try_pixel_type(src_image)?;
        let dst_pixel_type = try_pixel_type(dst_image)?;
        if src_pixel_type != dst_pixel_type {
            return Err(MulDivImagesError::PixelTypesAreDifferent);
        }

        #[cfg(not(feature = "only_u8x4"))]
        match src_pixel_type {
            PixelType::U8x2 => self.divide::<U8x2>(src_image, dst_image),
            PixelType::U8x4 => self.divide::<U8x4>(src_image, dst_image),
            PixelType::U16x2 => self.divide::<U16x2>(src_image, dst_image),
            PixelType::U16x4 => self.divide::<U16x4>(src_image, dst_image),
            PixelType::F32x2 => self.divide::<F32x2>(src_image, dst_image),
            PixelType::F32x4 => self.divide::<F32x4>(src_image, dst_image),
            _ => Err(MulDivImagesError::ImageError(
                ImageError::UnsupportedPixelType,
            )),
        }

        #[cfg(feature = "only_u8x4")]
        match src_pixel_type {
            PixelType::U8x4 => self.divide::<U8x4>(src_image, dst_image),
            _ => Err(MulDivImagesError::ImageError(
                ImageError::UnsupportedPixelType,
            )),
        }
    }

    #[inline]
    fn divide<P: PixelTrait>(
        &self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MulDivImagesError> {
        match (src_image.image_view(), dst_image.image_view_mut()) {
            (Some(src), Some(mut dst)) => self.divide_alpha_typed::<P>(&src, &mut dst),
            _ => Err(MulDivImagesError::ImageError(
                ImageError::UnsupportedPixelType,
            )),
        }
    }

    pub fn divide_alpha_typed<P: PixelTrait>(
        &self,
        src_view: &impl ImageView<Pixel = P>,
        dst_view: &mut impl ImageViewMut<Pixel = P>,
    ) -> Result<(), MulDivImagesError> {
        if src_view.width() != dst_view.width() || src_view.height() != dst_view.height() {
            return Err(MulDivImagesError::SizeIsDifferent);
        }
        if src_view.width() > 0 && src_view.height() > 0 {
            P::divide_alpha(src_view, dst_view, self.cpu_extensions)?;
        }
        Ok(())
    }

    /// Divides color-channels (RGB or Luma) of image by alpha-channel inplace.
    pub fn divide_alpha_inplace(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), ImageError> {
        let pixel_type = try_pixel_type(image)?;

        #[cfg(not(feature = "only_u8x4"))]
        match pixel_type {
            PixelType::U8x2 => self.divide_inplace::<U8x2>(image),
            PixelType::U8x4 => self.divide_inplace::<U8x4>(image),
            PixelType::U16x2 => self.divide_inplace::<U16x2>(image),
            PixelType::U16x4 => self.divide_inplace::<U16x4>(image),
            PixelType::F32x2 => self.divide_inplace::<F32x2>(image),
            PixelType::F32x4 => self.divide_inplace::<F32x4>(image),
            _ => Err(ImageError::UnsupportedPixelType),
        }

        #[cfg(feature = "only_u8x4")]
        match pixel_type {
            PixelType::U8x4 => self.divide_inplace::<U8x4>(image),
            _ => Err(ImageError::UnsupportedPixelType),
        }
    }

    #[inline]
    fn divide_inplace<P: PixelTrait>(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), ImageError> {
        match image.image_view_mut() {
            Some(mut view) => self.divide_alpha_inplace_typed::<P>(&mut view),
            _ => Err(ImageError::UnsupportedPixelType),
        }
    }

    pub fn divide_alpha_inplace_typed<P: PixelTrait>(
        &self,
        img_view: &mut impl ImageViewMut<Pixel = P>,
    ) -> Result<(), ImageError> {
        if img_view.width() > 0 && img_view.height() > 0 {
            P::divide_alpha_inplace(img_view, self.cpu_extensions)
        } else {
            Ok(())
        }
    }

    pub fn is_supported(&self, pixel_type: PixelType) -> bool {
        #[cfg(not(feature = "only_u8x4"))]
        {
            matches!(
                pixel_type,
                PixelType::U8x2
                    | PixelType::U8x4
                    | PixelType::U16x2
                    | PixelType::U16x4
                    | PixelType::F32x2
                    | PixelType::F32x4
            )
        }
        #[cfg(feature = "only_u8x4")]
        {
            matches!(pixel_type, PixelType::U8x4)
        }
    }
}
