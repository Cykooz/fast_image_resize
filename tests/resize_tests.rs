use std::fs::File;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, GenericImageView};

use fast_image_resize::ImageData;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer, SrcImageView};

fn get_source_image() -> ImageData<Vec<u8>> {
    let img = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = img.width();
    let height = img.height();
    let rgb = img.to_rgba8();
    let buf = rgb.as_raw().clone();
    ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        buf,
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_small_source_image() -> ImageData<Vec<u8>> {
    let img = ImageReader::open("./data/nasa-852x567.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = img.width();
    let height = img.height();
    let rgb = img.to_rgba8();
    let buf = rgb.as_raw().clone();
    ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        buf,
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_new_height(src_image: &SrcImageView, new_width: u32) -> u32 {
    let scale = new_width as f32 / src_image.width().get() as f32;
    (src_image.height().get() as f32 * scale).round() as u32
}

const NEW_WIDTH: u32 = 255;
const NEW_BIG_WIDTH: u32 = 5016;

fn save_result(image: &SrcImageView, name: &str) {
    std::fs::create_dir_all("./data/result").unwrap();
    let mut file = File::create(format!("./data/result/{}.png", name)).unwrap();
    let encoder = PngEncoder::new(&mut file);
    encoder
        .encode(
            &image.get_buffer(),
            image.width().get(),
            image.height().get(),
            ColorType::Rgba8,
        )
        .unwrap();
}

#[test]
fn resample_wo_simd_lanczos3_test() {
    let image = get_source_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    let new_height = get_new_height(&image.src_view(), NEW_WIDTH);
    let mut result = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    resizer.resize(&image.src_view(), &mut result.dst_view());
    save_result(&result.src_view(), "lanczos3_wo_simd");
}

#[test]
fn resample_sse4_lanczos3_test() {
    let image = get_source_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Sse4_1);
    }
    let new_height = get_new_height(&image.src_view(), NEW_WIDTH);
    let mut result = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    resizer.resize(&image.src_view(), &mut result.dst_view());
    save_result(&result.src_view(), "lanczos3_sse4");
}

fn resize_lanczos3(src_pixels: &[u8], width: NonZeroU32, height: NonZeroU32) -> Vec<u8> {
    let src_image = ImageData::new(width, height, src_pixels, PixelType::U8x4).unwrap();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_image.pixel_type());
    resizer.resize(&src_image.src_view(), &mut dst_image.dst_view());
    dst_image.get_buffer().to_owned()
}

#[test]
fn resample_avx2_lanczos3_test() {
    let image = get_source_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    let new_height = get_new_height(&image.src_view(), NEW_WIDTH);
    let mut result = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    resizer.resize(&image.src_view(), &mut result.dst_view());
    save_result(&result.src_view(), "lanczos3_avx2");
}

#[test]
fn resample_avx2_lanczos3_upscale_test() {
    let image = get_small_source_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    let new_height = get_new_height(&image.src_view(), NEW_BIG_WIDTH);
    let mut result = ImageData::new_owned(
        NonZeroU32::new(NEW_BIG_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    resizer.resize(&image.src_view(), &mut result.dst_view());
    save_result(&result.src_view(), "lanczos3_avx2_upscale");
}

#[test]
fn resample_nearest_test() {
    let image = get_source_image();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    let new_height = get_new_height(&image.src_view(), NEW_WIDTH);
    let mut result = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    resizer.resize(&image.src_view(), &mut result.dst_view());
    save_result(&result.src_view(), "nearest_wo_simd");
}

#[test]
fn resample_super_sampling_test() {
    let image = get_source_image();
    let mut resizer = Resizer::new(ResizeAlg::SuperSampling(FilterType::Lanczos3, 2));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    let new_height = get_new_height(&image.src_view(), NEW_WIDTH);
    let mut result = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        image.pixel_type(),
    );
    resizer.resize(&image.src_view(), &mut result.dst_view());
    save_result(&result.src_view(), "super_sampling_avx2");
}
