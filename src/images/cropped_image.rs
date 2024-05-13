use crate::images::{check_crop_box, TypedCroppedImage, TypedCroppedImageMut};
use crate::{
    CropBoxError, ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelTrait, PixelType,
};

/// It is a wrapper that provides [IntoImageView] for part of wrapped image.
pub struct CroppedImage<'a, V: IntoImageView> {
    image: &'a V,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

/// It is a wrapper that provides [IntoImageView] and [IntoImageViewMut] for part of wrapped image.
pub struct CroppedImageMut<'a, V: IntoImageView> {
    image: &'a mut V,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

impl<'a, V: IntoImageView> CroppedImage<'a, V> {
    pub fn new(
        image: &'a V,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, CropBoxError> {
        check_crop_box(image.width(), image.height(), left, top, width, height)?;
        Ok(Self {
            image,
            left,
            top,
            width,
            height,
        })
    }
}

impl<'a, V: IntoImageView> CroppedImageMut<'a, V> {
    pub fn new(
        image: &'a mut V,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, CropBoxError> {
        check_crop_box(image.width(), image.height(), left, top, width, height)?;
        Ok(Self {
            image,
            left,
            top,
            width,
            height,
        })
    }
}

impl<'a, V: IntoImageView> IntoImageView for CroppedImage<'a, V> {
    fn pixel_type(&self) -> Option<PixelType> {
        self.image.pixel_type()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
        self.image.image_view().map(|v| {
            TypedCroppedImage::new(v, self.left, self.top, self.width, self.height).unwrap()
        })
    }
}

impl<'a, V: IntoImageView> IntoImageView for CroppedImageMut<'a, V> {
    fn pixel_type(&self) -> Option<PixelType> {
        self.image.pixel_type()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
        self.image.image_view().map(|v| {
            TypedCroppedImage::new(v, self.left, self.top, self.width, self.height).unwrap()
        })
    }
}

impl<'a, V: IntoImageViewMut> IntoImageViewMut for CroppedImageMut<'a, V> {
    fn image_view_mut<P: PixelTrait>(&mut self) -> Option<impl ImageViewMut<Pixel = P>> {
        self.image.image_view_mut().map(|v| {
            TypedCroppedImageMut::new(v, self.left, self.top, self.width, self.height).unwrap()
        })
    }
}
