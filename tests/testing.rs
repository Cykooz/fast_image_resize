use std::fs::File;
use std::io::BufReader;
use std::num::NonZeroU32;
use std::ops::Deref;

use fast_image_resize::images::Image;
use fast_image_resize::pixels::*;
use fast_image_resize::{change_type_of_pixel_components, CpuExtensions, PixelTrait, PixelType};
use image::{ColorType, ExtendedColorType, ImageBuffer, ImageReader};

pub fn non_zero_u32(v: u32) -> NonZeroU32 {
    NonZeroU32::new(v).unwrap()
}

pub fn image_checksum<P: PixelTrait, const N: usize>(image: &Image) -> [u64; N] {
    let buffer = image.buffer();
    let mut res = [0u64; N];
    let component_size = P::size() / P::count_of_components();
    match component_size {
        1 => {
            for pixel in buffer.chunks_exact(N) {
                res.iter_mut().zip(pixel).for_each(|(d, &s)| *d += s as u64);
            }
        }
        2 => {
            let buffer_u16 = unsafe { buffer.align_to::<u16>().1 };
            for pixel in buffer_u16.chunks_exact(N) {
                res.iter_mut().zip(pixel).for_each(|(d, &s)| *d += s as u64);
            }
        }
        4 => {
            let buffer_u32 = unsafe { buffer.align_to::<u32>().1 };
            for pixel in buffer_u32.chunks_exact(N) {
                res.iter_mut()
                    .zip(pixel)
                    .for_each(|(d, &s)| *d = d.overflowing_add(s as u64).0);
            }
        }
        _ => (),
    };
    res
}

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
        let mut cpu_extensions_vec = vec![CpuExtensions::None];
        #[cfg(target_arch = "x86_64")]
        {
            cpu_extensions_vec.push(CpuExtensions::Sse4_1);
            cpu_extensions_vec.push(CpuExtensions::Avx2);
        }
        #[cfg(target_arch = "aarch64")]
        {
            cpu_extensions_vec.push(CpuExtensions::Neon);
        }
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        cpu_extensions_vec
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
                "./data/nasa-4928x3279.png",
                "./data/nasa-4019x4019.png",
                "./data/nasa-852x567.png",
            ),
            PixelType::U8x2
            | PixelType::U8x4
            | PixelType::U16x2
            | PixelType::U16x4
            | PixelType::F32x2
            | PixelType::F32x4 => (
                "./data/nasa-4928x3279-rgba.png",
                "./data/nasa-4019x4019-rgba.png",
                "./data/nasa-852x567-rgba.png",
            ),
            _ => unreachable!(),
        }
    }

    fn load_image_buffer(
        img_reader: ImageReader<BufReader<File>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container>;

    fn load_big_image() -> ImageBuffer<Self::ImagePixel, Self::Container> {
        Self::load_image_buffer(ImageReader::open(Self::img_paths().0).unwrap())
    }

    fn load_big_square_image() -> ImageBuffer<Self::ImagePixel, Self::Container> {
        Self::load_image_buffer(ImageReader::open(Self::img_paths().1).unwrap())
    }

    fn load_small_image() -> ImageBuffer<Self::ImagePixel, Self::Container> {
        Self::load_image_buffer(ImageReader::open(Self::img_paths().2).unwrap())
    }

    fn load_big_src_image() -> Image<'static> {
        let img = Self::load_big_image();
        Image::from_vec_u8(
            img.width(),
            img.height(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn load_big_square_src_image() -> Image<'static> {
        let img = Self::load_big_square_image();
        Image::from_vec_u8(
            img.width(),
            img.height(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn load_small_src_image() -> Image<'static> {
        let img = Self::load_small_image();
        Image::from_vec_u8(
            img.width(),
            img.height(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8>;
}

#[cfg(not(feature = "only_u8x4"))]
pub mod not_u8x4 {
    use super::*;

    impl PixelTestingExt for U8 {
        type ImagePixel = image::Luma<u8>;
        type Container = Vec<u8>;

        fn load_image_buffer(
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
        ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
            img_reader.decode().unwrap().to_luma16()
        }

        fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
            // img.as_raw()
            //     .iter()
            //     .enumerate()
            //     .flat_map(|(i, &c)| ((i & 0xffff) as u16).to_le_bytes())
            //     .collect()
            img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
        }
    }

    impl PixelTestingExt for U16x2 {
        type ImagePixel = image::LumaA<u16>;
        type Container = Vec<u16>;

        fn load_image_buffer(
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
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
            img_reader: ImageReader<BufReader<File>>,
        ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
            img_reader.decode().unwrap().to_rgba32f()
        }

        fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
            img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect()
        }
    }
}

impl PixelTestingExt for U8x4 {
    type ImagePixel = image::Rgba<u8>;
    type Container = Vec<u8>;

    fn load_image_buffer(
        img_reader: ImageReader<BufReader<File>>,
    ) -> ImageBuffer<Self::ImagePixel, Self::Container> {
        img_reader.decode().unwrap().to_rgba8()
    }

    fn img_into_bytes(img: ImageBuffer<Self::ImagePixel, Self::Container>) -> Vec<u8> {
        img.into_raw()
    }
}

pub fn save_result(image: &Image, name: &str) {
    if std::env::var("SAVE_RESULT")
        .unwrap_or_else(|_| "".to_owned())
        .is_empty()
    {
        return;
    }
    std::fs::create_dir_all("./data/result").unwrap();
    let path = format!("./data/result/{name}.png");

    let color_type: ExtendedColorType = match image.pixel_type() {
        PixelType::U8 => ColorType::L8.into(),
        PixelType::U8x2 => ColorType::La8.into(),
        PixelType::U8x3 => ColorType::Rgb8.into(),
        PixelType::U8x4 => ColorType::Rgba8.into(),
        PixelType::U16 => ColorType::L16.into(),
        PixelType::U16x2 => ColorType::La16.into(),
        PixelType::U16x3 => ColorType::Rgb16.into(),
        PixelType::U16x4 => ColorType::Rgba16.into(),
        PixelType::I32 | PixelType::F32 => {
            let mut image_u16 = Image::new(image.width(), image.height(), PixelType::U16);
            change_type_of_pixel_components(image, &mut image_u16).unwrap();
            save_result(&image_u16, name);
            return;
        }
        PixelType::F32x2 => {
            let mut image_u16 = Image::new(image.width(), image.height(), PixelType::U16x2);
            change_type_of_pixel_components(image, &mut image_u16).unwrap();
            save_result(&image_u16, name);
            return;
        }
        PixelType::F32x3 => {
            let mut image_u16 = Image::new(image.width(), image.height(), PixelType::U16x3);
            change_type_of_pixel_components(image, &mut image_u16).unwrap();
            save_result(&image_u16, name);
            return;
        }
        PixelType::F32x4 => {
            let mut image_u16 = Image::new(image.width(), image.height(), PixelType::U16x4);
            change_type_of_pixel_components(image, &mut image_u16).unwrap();
            save_result(&image_u16, name);
            return;
        }
        _ => panic!("Unsupported type of pixels"),
    };
    image::save_buffer(
        path,
        image.buffer(),
        image.width(),
        image.height(),
        color_type,
    )
    .unwrap();
}

pub const fn cpu_ext_into_str(cpu_extensions: CpuExtensions) -> &'static str {
    match cpu_extensions {
        CpuExtensions::None => "rust",
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => "sse4.1",
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => "avx2",
        #[cfg(target_arch = "aarch64")]
        CpuExtensions::Neon => "neon",
        #[cfg(target_arch = "wasm32")]
        CpuExtensions::Simd128 => "simd128",
    }
}
