use std::ops::DerefMut;

use bytemuck::cast_slice_mut;
use image::DynamicImage;

use crate::image_view::try_pixel_type;
use crate::images::{TypedImage, TypedImageMut};
use crate::pixels::InnerPixel;
use crate::{ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelType};

impl IntoImageView for DynamicImage {
    fn pixel_type(&self) -> Option<PixelType> {
        match self {
            DynamicImage::ImageLuma8(_) => Some(PixelType::U8),
            DynamicImage::ImageLumaA8(_) => Some(PixelType::U8x2),
            DynamicImage::ImageRgb8(_) => Some(PixelType::U8x3),
            DynamicImage::ImageRgba8(_) => Some(PixelType::U8x4),
            DynamicImage::ImageLuma16(_) => Some(PixelType::U16),
            DynamicImage::ImageLumaA16(_) => Some(PixelType::U16x2),
            DynamicImage::ImageRgb16(_) => Some(PixelType::U16x3),
            DynamicImage::ImageRgba16(_) => Some(PixelType::U16x4),
            _ => None,
        }
    }

    fn width(&self) -> u32 {
        self.width()
    }

    fn height(&self) -> u32 {
        self.height()
    }

    fn image_view<P: InnerPixel>(&self) -> Option<impl ImageView<Pixel = P>> {
        if let Ok(pixel_type) = try_pixel_type(self) {
            if P::pixel_type() == pixel_type {
                return TypedImage::<P>::from_buffer(self.width(), self.height(), self.as_bytes())
                    .ok();
            }
        }
        None
    }
}

impl IntoImageViewMut for DynamicImage {
    fn image_view_mut<P: InnerPixel>(&mut self) -> Option<impl ImageViewMut<Pixel = P>> {
        if let Ok(pixel_type) = try_pixel_type(self) {
            if P::pixel_type() == pixel_type {
                return TypedImageMut::<P>::from_buffer(
                    self.width(),
                    self.height(),
                    image_as_bytes_mut(self),
                )
                .ok();
            }
        }
        None
    }
}

fn image_as_bytes_mut(image: &mut DynamicImage) -> &mut [u8] {
    match image {
        DynamicImage::ImageLuma8(img) => (*img).deref_mut(),
        DynamicImage::ImageLumaA8(img) => (*img).deref_mut(),
        DynamicImage::ImageRgb8(img) => (*img).deref_mut(),
        DynamicImage::ImageRgba8(img) => (*img).deref_mut(),
        DynamicImage::ImageLuma16(img) => cast_slice_mut((*img).deref_mut()),
        DynamicImage::ImageLumaA16(img) => cast_slice_mut((*img).deref_mut()),
        DynamicImage::ImageRgb16(img) => cast_slice_mut((*img).deref_mut()),
        DynamicImage::ImageRgba16(img) => cast_slice_mut((*img).deref_mut()),
        _ => &mut [],
    }
}
