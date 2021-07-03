use std::num::NonZeroU32;

use criterion::{criterion_group, criterion_main, Criterion};

use fast_image_resize::ImageData;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};

mod utils;

const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

const NEW_BIG_WIDTH: u32 = 4928;
const NEW_BIG_HEIGHT: u32 = 3279;

fn get_big_source_image() -> ImageData<Vec<u8>> {
    let img = utils::get_big_rgb_image();
    let width = img.width();
    let height = img.height();
    let buf = img.as_raw().clone();
    ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        buf,
        PixelType::U8x4,
    )
    .unwrap()
}

fn get_small_source_image() -> ImageData<Vec<u8>> {
    let img = utils::get_small_rgb_image();
    let width = img.width();
    let height = img.height();
    let buf = img.as_raw().clone();
    ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        buf,
        PixelType::U8x4,
    )
    .unwrap()
}

fn downscale_nearest_wo_simd_bench(c: &mut Criterion) {
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
    c.bench_function("Downscale nearest wo SIMD", |b| {
        b.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn downscale_lanczos3_wo_simd_bench(c: &mut Criterion) {
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
    c.bench_function("Downscale lanczos3 wo SIMD", |b| {
        b.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn sse4_lanczos3_bench(c: &mut Criterion) {
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
    c.bench_function("sse4 lanczos3", |b| {
        b.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn avx2_lanczos3_bench(c: &mut Criterion) {
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
    c.bench_function("avx2 lanczos3", |b| {
        b.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn avx2_supersampling_lanczos3_bench(c: &mut Criterion) {
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
    c.bench_function("avx2 supersampling lanczos3", |b| {
        b.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

fn avx2_lanczos3_upscale_bench(c: &mut Criterion) {
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
    c.bench_function("avx2 lanczos3 upscale", |b| {
        b.iter(|| {
            resizer.resize(&src_image, &mut dst_image);
        })
    });
}

criterion_group!(
    benches,
    // downscale_nearest_wo_simd_bench,
    // downscale_lanczos3_wo_simd_bench,
    // resize_lanczos3_bench,
    // sse4_lanczos3_bench,
    avx2_lanczos3_bench,
    // avx2_supersampling_lanczos3_bench,
    // avx2_lanczos3_upscale_bench,
);
criterion_main!(benches);
