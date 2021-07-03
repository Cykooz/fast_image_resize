use std::num::NonZeroU32;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::imageops;
use resize::px::RGBA;
use resize::Pixel::{RGB8, RGBA8};
use rgb::{FromSlice, RGB};

use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};
use fast_image_resize::{ImageData, MulDiv};

mod utils;

pub fn bench_downscale_rgb(c: &mut Criterion) {
    let src_image = utils::get_big_rgb_image();
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let mut group = c.benchmark_group("Downscale RGB (4928x3279 => 852x567)");

    for alg_num in 0..4 {
        // image crate
        // https://crates.io/crates/image
        group.bench_with_input(
            BenchmarkId::new("image", alg_num),
            &alg_num,
            |b, alg_num| {
                b.iter(|| {
                    let filter = match alg_num {
                        0 => imageops::Nearest,
                        1 => imageops::Triangle,
                        2 => imageops::CatmullRom,
                        3 => imageops::Lanczos3,
                        _ => return,
                    };
                    imageops::resize(&src_image, new_width.get(), new_height.get(), filter);
                })
            },
        );

        // resize crate
        // https://crates.io/crates/resize
        let resize_src_image = src_image.as_raw().as_rgb();
        let mut dst = vec![RGB::new(0, 0, 0); (new_width.get() * new_height.get()) as usize];

        group.bench_with_input(
            BenchmarkId::new("resize", alg_num),
            &alg_num,
            |b, alg_num| {
                let filter = match alg_num {
                    0 => resize::Type::Point,
                    1 => resize::Type::Triangle,
                    2 => resize::Type::Catrom,
                    3 => resize::Type::Lanczos3,
                    _ => return,
                };
                let mut resizer = resize::new(
                    src_image.width() as usize,
                    src_image.height() as usize,
                    new_width.get() as usize,
                    new_height.get() as usize,
                    RGB8,
                    filter,
                )
                .unwrap();
                b.iter(|| {
                    resizer.resize(resize_src_image, &mut dst).unwrap();
                })
            },
        );

        // fast_image_resize crate;
        let src_rgba_image = utils::get_big_rgba_image();
        let src_image_data = ImageData::new(
            NonZeroU32::new(src_image.width()).unwrap(),
            NonZeroU32::new(src_image.height()).unwrap(),
            src_rgba_image.as_raw(),
            PixelType::U8x4,
        )
        .unwrap();
        let src_view = src_image_data.src_view();
        let mut dst_image = ImageData::new_owned(new_width, new_height, PixelType::U8x4);
        let mut dst_view = dst_image.dst_view();
        let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);
        unsafe {
            fast_resizer.set_cpu_extensions(CpuExtensions::None);
        }
        group.bench_with_input(
            BenchmarkId::new("fast_wo_simd", alg_num),
            &alg_num,
            |b, alg_num| {
                let resize_alg = match alg_num {
                    0 => ResizeAlg::Nearest,
                    1 => ResizeAlg::Convolution(FilterType::Bilinear),
                    2 => ResizeAlg::Convolution(FilterType::CatmulRom),
                    3 => ResizeAlg::Convolution(FilterType::Lanczos3),
                    _ => return,
                };
                fast_resizer.algorithm = resize_alg;
                b.iter(|| {
                    fast_resizer.resize(&src_view, &mut dst_view);
                })
            },
        );

        let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);
        unsafe {
            fast_resizer.set_cpu_extensions(CpuExtensions::Sse4_1);
        }
        group.bench_with_input(
            BenchmarkId::new("fast_sse4", alg_num),
            &alg_num,
            |b, alg_num| {
                let resize_alg = match alg_num {
                    0 => ResizeAlg::Nearest,
                    1 => ResizeAlg::Convolution(FilterType::Bilinear),
                    2 => ResizeAlg::Convolution(FilterType::CatmulRom),
                    3 => ResizeAlg::Convolution(FilterType::Lanczos3),
                    _ => return,
                };
                fast_resizer.algorithm = resize_alg;
                b.iter(|| {
                    fast_resizer.resize(&src_view, &mut dst_view);
                })
            },
        );

        let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);
        unsafe {
            fast_resizer.set_cpu_extensions(CpuExtensions::Avx2);
        }
        group.bench_with_input(
            BenchmarkId::new("fast_avx2", alg_num),
            &alg_num,
            |b, alg_num| {
                let resize_alg = match alg_num {
                    0 => ResizeAlg::Nearest,
                    1 => ResizeAlg::Convolution(FilterType::Bilinear),
                    2 => ResizeAlg::Convolution(FilterType::CatmulRom),
                    3 => ResizeAlg::Convolution(FilterType::Lanczos3),
                    _ => return,
                };
                fast_resizer.algorithm = resize_alg;
                b.iter(|| {
                    fast_resizer.resize(&src_view, &mut dst_view);
                })
            },
        );
    }

    group.finish();
}

pub fn bench_downscale_rgba(c: &mut Criterion) {
    let src_image = utils::get_big_rgba_image();
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let mut group = c.benchmark_group("Downscale RGBA (4928x3279 => 852x567)");

    for alg_num in 0..4 {
        // image crate
        // https://crates.io/crates/image
        group.bench_with_input(
            BenchmarkId::new("image", alg_num),
            &alg_num,
            |b, alg_num| {
                b.iter(|| {
                    let filter = match alg_num {
                        0 => imageops::Nearest,
                        1 => imageops::Triangle,
                        2 => imageops::CatmullRom,
                        3 => imageops::Lanczos3,
                        _ => return,
                    };
                    imageops::resize(&src_image, new_width.get(), new_height.get(), filter);
                })
            },
        );

        // resize crate
        // https://crates.io/crates/resize
        let resize_src_image = src_image.as_raw().as_rgba();
        let mut dst = vec![RGBA::new(0, 0, 0, 0); (new_width.get() * new_height.get()) as usize];

        group.bench_with_input(
            BenchmarkId::new("resize", alg_num),
            &alg_num,
            |b, alg_num| {
                let filter = match alg_num {
                    0 => resize::Type::Point,
                    1 => resize::Type::Triangle,
                    2 => resize::Type::Catrom,
                    3 => resize::Type::Lanczos3,
                    _ => return,
                };
                let mut resizer = resize::new(
                    src_image.width() as usize,
                    src_image.height() as usize,
                    new_width.get() as usize,
                    new_height.get() as usize,
                    RGBA8,
                    filter,
                )
                .unwrap();
                b.iter(|| {
                    resizer.resize(resize_src_image, &mut dst).unwrap();
                })
            },
        );

        // fast_image_resize crate;
        let src_image_data = ImageData::new(
            NonZeroU32::new(src_image.width()).unwrap(),
            NonZeroU32::new(src_image.height()).unwrap(),
            src_image.as_raw(),
            PixelType::U8x4,
        )
        .unwrap();
        let src_view = src_image_data.src_view();
        let mut premultiplied_src_image = ImageData::new_owned(
            NonZeroU32::new(src_image.width()).unwrap(),
            NonZeroU32::new(src_image.height()).unwrap(),
            PixelType::U8x4,
        );
        let mut dst_image = ImageData::new_owned(new_width, new_height, PixelType::U8x4);
        let mut dst_view = dst_image.dst_view();
        let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);
        let mut mul_div = MulDiv::default();
        unsafe {
            fast_resizer.set_cpu_extensions(CpuExtensions::None);
            mul_div.set_cpu_extensions(CpuExtensions::None);
        }
        group.bench_with_input(
            BenchmarkId::new("fast_wo_simd", alg_num),
            &alg_num,
            |b, alg_num| {
                let resize_alg = match alg_num {
                    0 => ResizeAlg::Nearest,
                    1 => ResizeAlg::Convolution(FilterType::Bilinear),
                    2 => ResizeAlg::Convolution(FilterType::CatmulRom),
                    3 => ResizeAlg::Convolution(FilterType::Lanczos3),
                    _ => return,
                };
                fast_resizer.algorithm = resize_alg;
                b.iter(|| {
                    mul_div
                        .multiply_alpha(&src_view, &mut premultiplied_src_image.dst_view())
                        .unwrap();
                    fast_resizer.resize(&premultiplied_src_image.src_view(), &mut dst_view);
                    mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
                })
            },
        );

        let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);
        unsafe {
            fast_resizer.set_cpu_extensions(CpuExtensions::Sse4_1);
            mul_div.set_cpu_extensions(CpuExtensions::Sse4_1);
        }
        group.bench_with_input(
            BenchmarkId::new("fast_sse4", alg_num),
            &alg_num,
            |b, alg_num| {
                let resize_alg = match alg_num {
                    0 => ResizeAlg::Nearest,
                    1 => ResizeAlg::Convolution(FilterType::Bilinear),
                    2 => ResizeAlg::Convolution(FilterType::CatmulRom),
                    3 => ResizeAlg::Convolution(FilterType::Lanczos3),
                    _ => return,
                };
                fast_resizer.algorithm = resize_alg;
                b.iter(|| {
                    mul_div
                        .multiply_alpha(&src_view, &mut premultiplied_src_image.dst_view())
                        .unwrap();
                    fast_resizer.resize(&premultiplied_src_image.src_view(), &mut dst_view);
                    mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
                })
            },
        );

        let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);
        unsafe {
            fast_resizer.set_cpu_extensions(CpuExtensions::Avx2);
            mul_div.set_cpu_extensions(CpuExtensions::Avx2);
        }
        group.bench_with_input(
            BenchmarkId::new("fast_avx2", alg_num),
            &alg_num,
            |b, alg_num| {
                let resize_alg = match alg_num {
                    0 => ResizeAlg::Nearest,
                    1 => ResizeAlg::Convolution(FilterType::Bilinear),
                    2 => ResizeAlg::Convolution(FilterType::CatmulRom),
                    3 => ResizeAlg::Convolution(FilterType::Lanczos3),
                    _ => return,
                };
                fast_resizer.algorithm = resize_alg;
                b.iter(|| {
                    mul_div
                        .multiply_alpha(&src_view, &mut premultiplied_src_image.dst_view())
                        .unwrap();
                    fast_resizer.resize(&premultiplied_src_image.src_view(), &mut dst_view);
                    mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_downscale_rgb, bench_downscale_rgba);
criterion_main!(benches);
