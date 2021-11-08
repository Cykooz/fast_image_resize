use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::Image;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};

mod utils;

const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

const NEW_BIG_WIDTH: u32 = 4928;
const NEW_BIG_HEIGHT: u32 = 3279;

fn get_big_source_image() -> Image<'static> {
    let img = utils::get_big_rgba_image();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.into_raw(),
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_big_i32_image() -> Image<'static> {
    let img = utils::get_big_luma16_image();
    let img_data: Vec<u32> = img
        .as_raw()
        .iter()
        .map(|&p| p as u32 * (i16::MAX as u32 + 1))
        .collect();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u32(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img_data,
        PixelType::I32,
    )
    .unwrap()
}

fn get_big_u8_image() -> Image<'static> {
    let img = utils::get_big_luma8_image();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.into_raw(),
        PixelType::U8,
    )
    .unwrap()
}

fn get_small_source_image() -> Image<'static> {
    let img = utils::get_small_rgba_image();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.into_raw(),
        PixelType::U8x4,
    )
    .unwrap()
}

fn native_nearest_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("nearest wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn native_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("lanczos3 wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn sse4_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Sse4_1);
    }
    bench.task("lanczos3 sse4", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn avx2_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("lanczos3 avx2", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn avx2_supersampling_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::SuperSampling(FilterType::Lanczos3, 2));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("supersampling lanczos3 avx2", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn avx2_lanczos3_upscale_bench(bench: &mut Bench) {
    let image = get_small_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_BIG_WIDTH).unwrap(),
        NonZeroU32::new(NEW_BIG_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("lanczos3 upscale avx2", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn native_lanczos3_i32_bench(bench: &mut Bench) {
    let image = get_big_i32_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("i32 lanczos3 wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn avx2_lanczos3_u8_bench(bench: &mut Bench) {
    let image = get_big_u8_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("u8 lanczos3 avx2", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn native_lanczos3_u8_bench(bench: &mut Bench) {
    let image = get_big_u8_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("u8 lanczos3 wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn native_nearest_u8_bench(bench: &mut Bench) {
    let image = get_big_u8_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("u8 nearest wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

pub fn main() {
    use glassbench::*;
    let name = env!("CARGO_CRATE_NAME");
    let cmd = Command::read();
    if cmd.include_bench(name) {
        let mut bench = create_bench(name, "Resize", &cmd);
        native_nearest_bench(&mut bench);
        native_lanczos3_bench(&mut bench);
        native_lanczos3_i32_bench(&mut bench);
        native_lanczos3_u8_bench(&mut bench);
        native_nearest_u8_bench(&mut bench);
        #[cfg(target_arch = "x86_64")]
        {
            sse4_lanczos3_bench(&mut bench);
            avx2_supersampling_lanczos3_bench(&mut bench);
            avx2_lanczos3_upscale_bench(&mut bench);
            avx2_lanczos3_bench(&mut bench);
            avx2_lanczos3_u8_bench(&mut bench);
        }
        if let Err(e) = after_bench(&mut bench, &cmd) {
            eprintln!("{:?}", e);
        }
    } else {
        println!("skipping bench {:?}", &name);
    }
}
