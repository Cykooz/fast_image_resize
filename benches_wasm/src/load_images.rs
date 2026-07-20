use std::io::Cursor;
use std::ops::Deref;

use fast_image_resize::images::Image;
use fast_image_resize::pixels::*;
use fast_image_resize::{CpuExtensions, PixelTrait, PixelType};
use image::{ImageBuffer, ImageFormat, ImageReader};
use wasm_bindgen::JsValue;

use super::utils::load_file;

pub trait PixelTestingExt: PixelTrait {
    type ImagePixel: image::Pixel;
    type Container: Deref<Target = [<Self::ImagePixel as image::Pixel>::Subpixel]>;

    fn pixel_type_str() -> &'static str {
        match Self::pixel_type() {
            PixelType::U8 => "u8",
            PixelType::U8x2 => "u8x2",
            PixelType::U8x3 => "u8x3",
            PixelType::U8x4 => "u8x4",
            PixelType::U16 => "u16",
            PixelType::U16x2 => "u16x2",
            PixelType::U16x3 => "u16x3",
            PixelType::U16x4 => "u16x4",
            PixelType::I32 => "i32",
            PixelType::F32 => "f32",
            PixelType::F32x2 => "f32x2",
            PixelType::F32x3 => "f32x3",
            PixelType::F32x4 => "f32x4",
            _ => unreachable!(),
        }
    }

    fn cpu_extensions() -> Vec<CpuExtensions> {
        vec![CpuExtensions::None, CpuExtensions::Simd128]
    }

    fn img_paths() -> (&'static str, &'static str, &'static str) {
        match Self::pixel_type() {
            PixelType::U8
            | PixelType::U8x3
            | PixelType::U16
            | PixelType::U16x3
            | PixelType::I32
            | PixelType::F32
            | PixelType::F32x3 => (
                "../data/nasa-4928x3279.png",
                "../data/nasa-4019x4019.png",
                "../data/nasa-852x567.png",
            ),
            PixelType::U8x2
            | PixelType::U8x4
            | PixelType::U16x2
            | PixelType::U16x4
            | PixelType::F32x2
            | PixelType::F32x4 => (
                "../data/nasa-4928x3279-rgba.png",
                "../data/nasa-4019x4019-rgba.png",
                "../data/nasa-852x567-rgba.png",
            ),
            _ => unreachable!(),
        }
    }

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container>;

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8>;
}

pub async fn load_big_image<T: PixelTestingExt>()
-> Result<ImageBuffer<T::ImagePixel, T::Container>, JsValue> {
    let path = T::img_paths().0;
    let data = load_file(path).await?;
    let mut reader = ImageReader::new(Cursor::new(data));
    reader.set_format(ImageFormat::Png);
    Ok(T::load_image_buffer(reader))
}

pub async fn load_big_square_image<T: PixelTestingExt>()
-> Result<ImageBuffer<T::ImagePixel, T::Container>, JsValue> {
    let path = T::img_paths().1;
    let data = load_file(path).await?;
    let mut reader = ImageReader::new(Cursor::new(data));
    reader.set_format(ImageFormat::Png);
    Ok(T::load_image_buffer(reader))
}

pub async fn load_small_image<T: PixelTestingExt>()
-> Result<ImageBuffer<T::ImagePixel, T::Container>, JsValue> {
    let path = T::img_paths().2;
    let data = load_file(path).await?;
    let mut reader = ImageReader::new(Cursor::new(data));
    reader.set_format(ImageFormat::Png);
    Ok(T::load_image_buffer(reader))
}

pub async fn load_big_src_image<T: PixelTestingExt>() -> Result<Image<'static>, JsValue> {
    let img = load_big_image::<T>().await?;
    Ok(Image::from_vec_u8(
        img.width(),
        img.height(),
        T::img_into_bytes(img),
        T::pixel_type(),
    )
    .unwrap())
}

pub async fn load_big_square_src_image<T: PixelTestingExt>() -> Result<Image<'static>, JsValue> {
    let img = load_big_square_image::<T>().await?;
    Ok(Image::from_vec_u8(
        img.width(),
        img.height(),
        T::img_into_bytes(img),
        T::pixel_type(),
    )
    .unwrap())
}

pub async fn load_small_src_image<T: PixelTestingExt>() -> Result<Image<'static>, JsValue> {
    let img = load_small_image::<T>().await?;
    Ok(Image::from_vec_u8(
        img.width(),
        img.height(),
        T::img_into_bytes(img),
        T::pixel_type(),
    )
    .unwrap())
}

impl PixelTestingExt for U8 {
    type ImagePixel = image::Luma<u8>;
    type Container = Vec<u8>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_luma8()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.into_raw()
    }
}

impl PixelTestingExt for U8x2 {
    type ImagePixel = image::LumaA<u8>;
    type Container = Vec<u8>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_luma_alpha8()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.into_raw()
    }
}

impl PixelTestingExt for U8x3 {
    type ImagePixel = image::Rgb<u8>;
    type Container = Vec<u8>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgb8()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.into_raw()
    }
}

impl PixelTestingExt for U16 {
    type ImagePixel = image::Luma<u16>;
    type Container = Vec<u16>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_luma16()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
    }
}

impl PixelTestingExt for U16x2 {
    type ImagePixel = image::LumaA<u16>;
    type Container = Vec<u16>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_luma_alpha16()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
    }
}

impl PixelTestingExt for U16x3 {
    type ImagePixel = image::Rgb<u16>;
    type Container = Vec<u16>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgb16()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
    }
}

impl PixelTestingExt for U16x4 {
    type ImagePixel = image::Rgba<u16>;
    type Container = Vec<u16>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgba16()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
    }
}

impl PixelTestingExt for I32 {
    type ImagePixel = image::Luma<i32>;
    type Container = Vec<i32>;

    fn cpu_extensions() -> Vec<CpuExtensions> {
        vec![CpuExtensions::None]
    }

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        let image_u16 = img_reader.decode().unwrap().to_luma32f();
        ImageBuffer::from_fn(image_u16.width(), image_u16.height(), |x, y| {
            let pixel = image_u16.get_pixel(x, y);
            image::Luma::from([(pixel.0[0] * i32::MAX as f32).round() as i32])
        })
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw()
            .iter()
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

impl PixelTestingExt for F32 {
    type ImagePixel = image::Luma<f32>;
    type Container = Vec<f32>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_luma32f()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw()
            .iter()
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

impl PixelTestingExt for F32x2 {
    type ImagePixel = image::LumaA<f32>;
    type Container = Vec<f32>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_luma_alpha32f()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw()
            .iter()
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

impl PixelTestingExt for F32x3 {
    type ImagePixel = image::Rgb<f32>;
    type Container = Vec<f32>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgb32f()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
    }
}

impl PixelTestingExt for F32x4 {
    type ImagePixel = image::Rgba<f32>;
    type Container = Vec<f32>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgba32f()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
    }
}

impl PixelTestingExt for U8x4 {
    type ImagePixel = image::Rgba<u8>;
    type Container = Vec<u8>;

    fn load_image_buffer(
        img_reader: ImageReader<Cursor<Vec<u8>>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgba8()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.into_raw()
    }
}

pub const fn cpu_ext_into_str(cpu_extensions: CpuExtensions) -> &'static str {
    match cpu_extensions {
        CpuExtensions::None => "rust",
        CpuExtensions::Simd128 => "simd128",
    }
}
