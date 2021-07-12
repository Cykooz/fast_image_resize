use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::ImageData;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};

mod utils;

const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

const NEW_BIG_WIDTH: u32 = 4928;
const NEW_BIG_HEIGHT: u32 = 3279;

fn get_big_source_image() -> ImageData<Vec<u32>> {
    let img = utils::get_big_rgba_image();
    let width = img.width();
    let height = img.height();
    let buf = img
        .as_raw()
        .chunks_exact(4)
        .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
        .collect();
    ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        buf,
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_small_source_image() -> ImageData<Vec<u32>> {
    let img = utils::get_small_rgba_image();
    let width = img.width();
    let height = img.height();
    let buf = img
        .as_raw()
        .chunks_exact(4)
        .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
        .collect();
    ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        buf,
        PixelType::U8x4,
    )
    .unwrap()
}

fn nearest_wo_simd_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.src_view();
    let mut dst_image = res_image.dst_view();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("nearest wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn lanczos3_wo_simd_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.src_view();
    let mut dst_image = res_image.dst_view();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("lanczos3 wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn sse4_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.src_view();
    let mut dst_image = res_image.dst_view();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Sse4_1);
    }
    bench.task("sse4 lanczos3", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn avx2_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.src_view();
    let mut dst_image = res_image.dst_view();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("avx2 lanczos3", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn avx2_supersampling_lanczos3_bench(bench: &mut Bench) {
    let image = get_big_source_image();
    let mut res_image = ImageData::new_owned(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.src_view();
    let mut dst_image = res_image.dst_view();
    let mut resizer = Resizer::new(ResizeAlg::SuperSampling(FilterType::Lanczos3, 2));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("avx2 supersampling lanczos3", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn avx2_lanczos3_upscale_bench(bench: &mut Bench) {
    let image = get_small_source_image();
    let mut res_image = ImageData::new_owned(
        NonZeroU32::new(NEW_BIG_WIDTH).unwrap(),
        NonZeroU32::new(NEW_BIG_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.src_view();
    let mut dst_image = res_image.dst_view();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::Avx2);
    }
    bench.task("avx2 lanczos3 upscale", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

glassbench!(
    "Resize",
    nearest_wo_simd_bench,
    lanczos3_wo_simd_bench,
    sse4_lanczos3_bench,
    avx2_lanczos3_bench,
    avx2_supersampling_lanczos3_bench,
    avx2_lanczos3_upscale_bench,
);
