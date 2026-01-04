use core::ops::DerefMut;

use crate::image_view::try_pixel_type;
use crate::images::{TypedImage, TypedImageRef};
use crate::{ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelTrait, PixelType};
use bytemuck::cast_slice_mut;
use image::{
    DynamicImage, GrayAlphaImage, GrayImage, ImageBuffer, Luma, LumaA, Rgb, Rgb32FImage, RgbImage,
    Rgba, Rgba32FImage, RgbaImage,
};

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
            DynamicImage::ImageRgb32F(_) => Some(PixelType::F32x3),
            DynamicImage::ImageRgba32F(_) => Some(PixelType::F32x4),
            _ => None,
        }
    }

    fn width(&self) -> u32 {
        self.width()
    }

    fn height(&self) -> u32 {
        self.height()
    }

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
        if let Ok(pixel_type) = try_pixel_type(self) {
            if P::pixel_type() == pixel_type {
                return TypedImageRef::<P>::from_buffer(
                    self.width(),
                    self.height(),
                    self.as_bytes(),
                )
                .ok();
            }
        }
        None
    }
}

impl IntoImageViewMut for DynamicImage {
    fn image_view_mut<P: PixelTrait>(&mut self) -> Option<impl ImageViewMut<Pixel = P>> {
        if let Ok(pixel_type) = try_pixel_type(self) {
            if P::pixel_type() == pixel_type {
                return TypedImage::<P>::from_buffer(
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
        DynamicImage::ImageRgb32F(img) => cast_slice_mut((*img).deref_mut()),
        DynamicImage::ImageRgba32F(img) => cast_slice_mut((*img).deref_mut()),
        _ => &mut [],
    }
}

// Implementations for supported versions of ImageBuffer

macro_rules! impl_for_img_buffer {
    ($img_type:tt, $pixel_type:expr) => {
        impl IntoImageView for $img_type {
            fn pixel_type(&self) -> Option<PixelType> {
                Some($pixel_type)
            }

            fn width(&self) -> u32 {
                self.width()
            }

            fn height(&self) -> u32 {
                self.height()
            }

            fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
                if P::pixel_type() == $pixel_type {
                    let bytes = bytemuck::cast_slice(self.as_raw().as_ref());
                    return TypedImageRef::<P>::from_buffer(self.width(), self.height(), bytes)
                        .ok();
                }
                None
            }
        }

        impl IntoImageViewMut for $img_type {
            fn image_view_mut<P: PixelTrait>(&mut self) -> Option<impl ImageViewMut<Pixel = P>> {
                if P::pixel_type() == $pixel_type {
                    return TypedImage::<P>::from_buffer(
                        self.width(),
                        self.height(),
                        cast_slice_mut((*self).deref_mut()),
                    )
                    .ok();
                }
                None
            }
        }
    };
}

impl_for_img_buffer!(GrayImage, PixelType::U8);
impl_for_img_buffer!(GrayAlphaImage, PixelType::U8x2);
impl_for_img_buffer!(RgbImage, PixelType::U8x3);
impl_for_img_buffer!(RgbaImage, PixelType::U8x4);

type Gray16Image = ImageBuffer<Luma<u16>, Vec<u16>>;
impl_for_img_buffer!(Gray16Image, PixelType::U16);

type GrayAlpha16Image = ImageBuffer<LumaA<u16>, Vec<u16>>;
impl_for_img_buffer!(GrayAlpha16Image, PixelType::U16x2);

type Rgb16Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
impl_for_img_buffer!(Rgb16Image, PixelType::U16x3);

type Rgba16Image = ImageBuffer<Rgba<u16>, Vec<u16>>;
impl_for_img_buffer!(Rgba16Image, PixelType::U16x4);

impl_for_img_buffer!(Rgb32FImage, PixelType::F32x3);
impl_for_img_buffer!(Rgba32FImage, PixelType::F32x4);
