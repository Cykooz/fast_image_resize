use std::num::NonZeroU32;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image::imageops;
use resize::px::RGBA;
use resize::Pixel::RGBA8;
use rgb::FromSlice;

use fast_image_resize::ImageData;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};
mod utils;

pub fn bench_downscale(c: &mut Criterion) {
    let src_image = utils::get_big_rgb_image();
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let mut group = c.benchmark_group("Downscale (4928x3279 => 852x567)");

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

        // fast_image_resize crate
        let src_image_data = ImageData::new(
            NonZeroU32::new(src_image.width()).unwrap(),
            NonZeroU32::new(src_image.height()).unwrap(),
            src_image.as_raw(),
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

criterion_group!(benches, bench_downscale,);
criterion_main!(benches);
