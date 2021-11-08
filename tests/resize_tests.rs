use std::fs::File;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, GenericImageView};

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

fn get_source_image_u8x1() -> Image<'static> {
    let img = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.to_luma8().into_raw(),
        PixelType::U8,
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
fn resize_wo_simd_lanczos3_test() {
    let image = get_source_image_u8x4();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
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
    save_result(&result, "u8x4-lanczos3-native");
}

#[test]
fn resize_sse4_lanczos3_test() {
    let image = get_source_image_u8x4();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Sse4_1);
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
    save_result(&result, "u8x4-lanczos3-sse4");
}

#[test]
fn resize_avx2_lanczos3_test() {
    let image = get_source_image_u8x4();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
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
    save_result(&result, "u8x4-lanczos3-avx2");
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
fn resize_nearest_test() {
    let image = get_source_image_u8x4();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
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
    save_result(&result, "u8x4-nearest-native");
}

#[test]
fn resize_super_sampling_test() {
    let image = get_source_image_u8x4();
    let mut resizer = Resizer::new(ResizeAlg::SuperSampling(FilterType::Lanczos3, 2));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
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
    save_result(&result, "u8x4-super_sampling-avx2");
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

#[test]
fn resize_nearest_u8x1() {
    let image = get_source_image_u8x1();
    assert!(matches!(image.pixel_type(), PixelType::U8));

    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
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
    save_result(&result, "u8x1-nearest-native");
}

#[test]
fn resize_lanczos3_u8x1_native() {
    let image = get_source_image_u8x1();
    assert!(matches!(image.pixel_type(), PixelType::U8));

    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
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
    save_result(&result, "u8x1-lanczos3-native");
}

#[test]
fn resize_lanczos3_u8x1_avx2() {
    let image = get_source_image_u8x1();
    assert!(matches!(image.pixel_type(), PixelType::U8));

    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
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
    save_result(&result, "u8x1-lanczos3-avx2");
}
