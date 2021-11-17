use std::fs::File;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, DynamicImage, GenericImageView};

use fast_image_resize::pixels::*;
use fast_image_resize::{
    CpuExtensions, DifferentTypesOfPixelsError, FilterType, Image, ImageView, PixelType, ResizeAlg,
    Resizer,
};

fn get_source_image_u8x4() -> Image<'static> {
    let img = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.to_rgba8().into_raw(),
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_small_source_image() -> Image<'static> {
    let img = ImageReader::open("./data/nasa-852x567.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.to_rgba8().into_raw(),
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_new_height(src_image: &ImageView, new_width: u32) -> u32 {
    let scale = new_width as f32 / src_image.width().get() as f32;
    (src_image.height().get() as f32 * scale).round() as u32
}

const NEW_WIDTH: u32 = 255;
const NEW_BIG_WIDTH: u32 = 5016;

fn save_result(image: &Image, name: &str) {
    std::fs::create_dir_all("./data/result").unwrap();
    let mut file = File::create(format!("./data/result/{}.png", name)).unwrap();
    let color_type = match image.pixel_type() {
        PixelType::U8x3 => ColorType::Rgb8,
        PixelType::U8x4 => ColorType::Rgba8,
        PixelType::U8 => ColorType::L8,
        _ => panic!("Unsupported type of pixels"),
    };
    PngEncoder::new(&mut file)
        .encode(
            image.buffer(),
            image.width().get(),
            image.height().get(),
            color_type,
        )
        .unwrap();
}

#[test]
fn resize_avx2_lanczos3_upscale_test() {
    let image = get_small_source_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    let new_height = get_new_height(&image.view(), NEW_BIG_WIDTH);
    let mut result = Image::new(
        NonZeroU32::new(NEW_BIG_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    assert!(resizer
        .resize(&image.view(), &mut result.view_mut())
        .is_ok());
    save_result(&result, "u8x4-lanczos3_upscale-avx2");
}

#[test]
fn try_resize_to_other_pixel_type() {
    let src_image = get_source_image_u8x4();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    let mut dst_image = Image::new(
        NonZeroU32::new(1024).unwrap(),
        NonZeroU32::new(256).unwrap(),
        PixelType::U8,
    );
    assert!(matches!(
        resizer.resize(&src_image.view(), &mut dst_image.view_mut()),
        Err(DifferentTypesOfPixelsError)
    ));
}

trait PixelExt: Pixel {
    fn pixel_type_str() -> &'static str {
        match Self::pixel_type() {
            PixelType::U8 => "u8",
            PixelType::U8x3 => "u8x3",
            PixelType::U8x4 => "u8x4",
            PixelType::I32 => "i32",
            PixelType::F32 => "f32",
        }
    }

    fn load_src_image() -> Image<'static> {
        let img = ImageReader::open("./data/nasa-4928x3279.png")
            .unwrap()
            .decode()
            .unwrap();
        Image::from_vec_u8(
            NonZeroU32::new(img.width()).unwrap(),
            NonZeroU32::new(img.height()).unwrap(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn img_into_bytes(img: DynamicImage) -> Vec<u8>;
}

impl PixelExt for U8 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_luma8().into_raw()
    }
}

impl PixelExt for U8x3 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_rgb8().into_raw()
    }
}

impl PixelExt for U8x4 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_rgba8().into_raw()
    }
}

impl PixelExt for I32 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_luma16()
            .as_raw()
            .iter()
            .map(|&p| p as u32 * (i16::MAX as u32 + 1))
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

impl PixelExt for F32 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_luma16()
            .as_raw()
            .iter()
            .map(|&p| p as f32 * (i16::MAX as f32 + 1.0))
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

fn resize_test<P: PixelExt>(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions) {
    let image = P::load_src_image();
    assert_eq!(image.pixel_type(), P::pixel_type());

    let mut resizer = Resizer::new(resize_alg);
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    let new_height = get_new_height(&image.view(), NEW_WIDTH);
    let mut result = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    assert!(resizer
        .resize(&image.view(), &mut result.view_mut())
        .is_ok());

    let alg_name = match resize_alg {
        ResizeAlg::Nearest => "nearest",
        ResizeAlg::Convolution(filter) => match filter {
            FilterType::Box => "box",
            FilterType::Bilinear => "bilinear",
            FilterType::Hamming => "hamming",
            FilterType::Mitchell => "mitchell",
            FilterType::CatmullRom => "catmullrom",
            FilterType::Lanczos3 => "lanczos3",
            _ => "unknown",
        },
        ResizeAlg::SuperSampling(_, _) => "supersampling",
        _ => "unknown",
    };

    let ext_name = match cpu_extensions {
        CpuExtensions::None => "native",
        CpuExtensions::Sse2 => "sse2",
        CpuExtensions::Sse4_1 => "sse41",
        CpuExtensions::Avx2 => "avx2",
    };

    let name = format!("{}-{}-{}", P::pixel_type_str(), alg_name, ext_name);
    save_result(&result, &name);
}

#[test]
fn resize_u8() {
    type P = U8;
    resize_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    for cpu_extensions in [CpuExtensions::None, CpuExtensions::Avx2] {
        resize_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
    }
}

#[test]
fn resize_u8x3() {
    type P = U8x3;
    resize_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    for cpu_extensions in [CpuExtensions::None] {
        resize_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
    }
}

#[test]
fn resize_u8x4() {
    type P = U8x4;
    resize_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    for cpu_extensions in [
        CpuExtensions::None,
        CpuExtensions::Sse4_1,
        CpuExtensions::Avx2,
    ] {
        resize_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        resize_test::<P>(
            ResizeAlg::SuperSampling(FilterType::Lanczos3, 2),
            cpu_extensions,
        );
    }
}

// #[test]
fn _resize_i32() {
    type P = I32;
    resize_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    for cpu_extensions in [CpuExtensions::None] {
        resize_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
    }
}
