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

fn get_big_u8x2_source_image() -> Image<'static> {
    let img = utils::get_big_luma_alpha8_image();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.into_raw(),
        PixelType::U8x2,
    )
    .unwrap()
}

fn get_big_u8x3_source_image() -> Image<'static> {
    let img = utils::get_big_rgb_image();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.into_raw(),
        PixelType::U8x3,
    )
    .unwrap()
}

fn get_big_u16x3_source_image() -> Image<'static> {
    let img = utils::get_big_rgb16_image();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        img.as_raw().iter().flat_map(|&c| c.to_le_bytes()).collect(),
        PixelType::U16x3,
    )
    .unwrap()
}

fn get_big_i32_image() -> Image<'static> {
    let img = utils::get_big_luma16_image();
    let img_data: Vec<u8> = img
        .as_raw()
        .iter()
        .flat_map(|&p| (p as u32 * (i16::MAX as u32 + 1)).to_le_bytes())
        .collect();
    let width = img.width();
    let height = img.height();
    Image::from_vec_u8(
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

fn native_nearest_u8x4_bench(bench: &mut Bench) {
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

fn u8x4_lanczos3_bench(bench: &mut Bench, cpu_extensions: CpuExtensions, name: &str) {
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
        resizer.set_cpu_extensions(cpu_extensions);
    }
    bench.task(name, |task| {
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

fn u8_lanczos3_bench(bench: &mut Bench, cpu_extensions: CpuExtensions, name: &str) {
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
        resizer.set_cpu_extensions(cpu_extensions);
    }
    bench.task(name, |task| {
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

fn u8x3_lanczos3_bench(bench: &mut Bench, cpu_extensions: CpuExtensions, name: &str) {
    let image = get_big_u8x3_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    bench.task(name, |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn u16x3_lanczos3_bench(bench: &mut Bench, cpu_extensions: CpuExtensions, name: &str) {
    let image = get_big_u16x3_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    bench.task(name, |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn u8x2_lanczos3_bench(bench: &mut Bench, cpu_extensions: CpuExtensions, name: &str) {
    let image = get_big_u8x2_source_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    bench.task(name, |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

pub fn main() {
    // Pin process to #0 CPU core
    let mut cpu_set = nix::sched::CpuSet::new();
    cpu_set.set(0).unwrap();
    nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();

    use glassbench::*;
    let name = env!("CARGO_CRATE_NAME");
    let cmd = Command::read();
    if cmd.include_bench(name) {
        let mut bench = create_bench(name, "Resize", &cmd);
        native_nearest_u8x4_bench(&mut bench);
        native_nearest_u8_bench(&mut bench);

        u8_lanczos3_bench(&mut bench, CpuExtensions::None, "u8 lanczos3 wo SIMD");
        u8x3_lanczos3_bench(&mut bench, CpuExtensions::None, "u8x3 lanczos3 wo SIMD");
        u8x4_lanczos3_bench(&mut bench, CpuExtensions::None, "u8x4 lanczos3 wo SIMD");
        u16x3_lanczos3_bench(&mut bench, CpuExtensions::None, "u16x3 lanczos3 wo SIMD");
        native_lanczos3_i32_bench(&mut bench);
        #[cfg(target_arch = "x86_64")]
        {
            u8_lanczos3_bench(&mut bench, CpuExtensions::Sse4_1, "u8 lanczos3 sse4.1");
            u8_lanczos3_bench(&mut bench, CpuExtensions::Avx2, "u8 lanczos3 avx2");

            u8x2_lanczos3_bench(&mut bench, CpuExtensions::Avx2, "u8x2 lanczos3 avx2");

            u8x3_lanczos3_bench(&mut bench, CpuExtensions::Sse4_1, "u8x3 lanczos3 sse4.1");
            u8x3_lanczos3_bench(&mut bench, CpuExtensions::Avx2, "u8x3 lanczos3 avx2");
            u16x3_lanczos3_bench(&mut bench, CpuExtensions::Sse4_1, "u16x3 lanczos3 sse4.1");
            u16x3_lanczos3_bench(&mut bench, CpuExtensions::Avx2, "u16x3 lanczos3 avx2");

            u8x4_lanczos3_bench(&mut bench, CpuExtensions::Sse4_1, "u8x4 lanczos3 sse4.1");
            u8x4_lanczos3_bench(&mut bench, CpuExtensions::Avx2, "u8x4 lanczos3 avx2");

            avx2_supersampling_lanczos3_bench(&mut bench);
            avx2_lanczos3_upscale_bench(&mut bench);
        }
        if let Err(e) = after_bench(&mut bench, &cmd) {
            eprintln!("{:?}", e);
        }
    } else {
        println!("skipping bench {:?}", &name);
    }
}
