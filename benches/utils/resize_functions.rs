use std::ops::Deref;

use image::{imageops, ImageBuffer};

use crate::utils::bencher::{bench, BenchGroup};
use fast_image_resize::{CpuExtensions, FilterType, Image, MulDiv, ResizeAlg, Resizer};
use testing::{cpu_ext_into_str, nonzero, PixelTestingExt};

const ALG_NAMES: [&str; 4] = ["Nearest", "Bilinear", "CatmullRom", "Lanczos3"];
const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

/// Resize image with help of "image" crate (https://crates.io/crates/image)
pub fn image_resize<P, C>(bench_group: &mut BenchGroup, src_image: &ImageBuffer<P, C>)
where
    P: image::Pixel + 'static,
    C: Deref<Target = [P::Subpixel]>,
{
    for alg_name in ALG_NAMES {
        let (filter, sample_size) = match alg_name {
            "Nearest" => (imageops::Nearest, 80),
            "Bilinear" => (imageops::Triangle, 50),
            "CatmullRom" => (imageops::CatmullRom, 30),
            "Lanczos3" => (imageops::Lanczos3, 20),
            _ => continue,
        };
        bench(bench_group, sample_size, "image", alg_name, |bencher| {
            bencher.iter(|| {
                imageops::resize(src_image, NEW_WIDTH, NEW_HEIGHT, filter);
            })
        });
    }
}

/// Resize image with help of "resize" crate (https://crates.io/crates/resize)
pub fn resize_resize<Format, Out>(
    bench_group: &mut BenchGroup,
    pixel_format: Format,
    src_image: &[Format::InputPixel],
    src_width: u32,
    src_height: u32,
) where
    Out: Clone,
    Format: resize::PixelFormat<OutputPixel = Out> + Copy,
{
    for alg_name in ALG_NAMES {
        if alg_name == "Nearest" {
            // "resize" doesn't support "nearest" algorithm
            continue;
        }
        let mut dst =
            vec![pixel_format.into_pixel(Format::new()); (NEW_WIDTH * NEW_HEIGHT) as usize];
        let sample_size = if alg_name == "Lanczos3" { 60 } else { 100 };

        bench(bench_group, sample_size, "resize", alg_name, |bencher| {
            let filter = match alg_name {
                "Bilinear" => resize::Type::Triangle,
                "CatmullRom" => resize::Type::Catrom,
                "Lanczos3" => resize::Type::Lanczos3,
                _ => return,
            };
            let mut resizer = resize::new(
                src_width as usize,
                src_height as usize,
                NEW_WIDTH as usize,
                NEW_HEIGHT as usize,
                pixel_format,
                filter,
            )
            .unwrap();
            bencher.iter(|| {
                resizer.resize(src_image, &mut dst).unwrap();
            })
        });
    }
}

/// Resize image with help of "fast_imager_resize" crate
pub fn fir_resize<P: PixelTestingExt>(bench_group: &mut BenchGroup) {
    let src_image_data = P::load_big_src_image();
    let src_view = src_image_data.view();
    let mut dst_image = Image::new(
        nonzero(NEW_WIDTH),
        nonzero(NEW_HEIGHT),
        src_view.pixel_type(),
    );
    let mut dst_view = dst_image.view_mut();

    let mut cpu_extensions = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions.push(CpuExtensions::Sse4_1);
        cpu_extensions.push(CpuExtensions::Avx2);
    }
    #[cfg(target_arch = "aarch64")]
    {
        cpu_extensions.push(CpuExtensions::Neon);
    }
    #[cfg(target_arch = "wasm32")]
    {
        cpu_extensions.push(CpuExtensions::Simd128);
    }
    for cpu_ext in cpu_extensions {
        for alg_name in ALG_NAMES {
            let resize_alg = match alg_name {
                "Nearest" => {
                    if cpu_ext != CpuExtensions::None {
                        // Nearest algorithm implemented only for native Rust.
                        continue;
                    }
                    ResizeAlg::Nearest
                }
                "Bilinear" => ResizeAlg::Convolution(FilterType::Bilinear),
                "CatmullRom" => ResizeAlg::Convolution(FilterType::CatmullRom),
                "Lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
                _ => continue,
            };
            let mut fast_resizer = Resizer::new(resize_alg);
            unsafe {
                fast_resizer.set_cpu_extensions(cpu_ext);
            }
            let sample_size = 100;

            bench(
                bench_group,
                sample_size,
                format!("fir {}", cpu_ext_into_str(cpu_ext)),
                alg_name,
                |bencher| {
                    fast_resizer.reset_internal_buffers();
                    bencher.iter(|| {
                        fast_resizer.resize(&src_view, &mut dst_view).unwrap();
                    })
                },
            );
        }
    }
}

/// Resize image with alpha channel with help of "fast_imager_resize" crate
pub fn fir_resize_with_alpha<P: PixelTestingExt>(bench_group: &mut BenchGroup) {
    let src_image = P::load_big_src_image();
    let src_view = src_image.view();
    let mut premultiplied_src_image =
        Image::new(src_image.width(), src_image.height(), src_view.pixel_type());
    let mut dst_image = Image::new(
        nonzero(NEW_WIDTH),
        nonzero(NEW_HEIGHT),
        src_view.pixel_type(),
    );
    let mut dst_view = dst_image.view_mut();
    let mut mul_div = MulDiv::default();

    let mut cpu_ext_and_name = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_ext_and_name.push(CpuExtensions::Sse4_1);
        cpu_ext_and_name.push(CpuExtensions::Avx2);
    }
    #[cfg(target_arch = "aarch64")]
    {
        cpu_ext_and_name.push(CpuExtensions::Neon);
    }
    #[cfg(target_arch = "wasm32")]
    {
        cpu_ext_and_name.push(CpuExtensions::Simd128);
    }
    for cpu_ext in cpu_ext_and_name {
        for alg_name in ALG_NAMES {
            let resize_alg = match alg_name {
                "Nearest" => {
                    if cpu_ext != CpuExtensions::None {
                        // Nearest algorithm implemented only for native Rust.
                        continue;
                    }
                    ResizeAlg::Nearest
                }
                "Bilinear" => ResizeAlg::Convolution(FilterType::Bilinear),
                "CatmullRom" => ResizeAlg::Convolution(FilterType::CatmullRom),
                "Lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
                _ => return,
            };
            let mut fast_resizer = Resizer::new(resize_alg);
            unsafe {
                fast_resizer.set_cpu_extensions(cpu_ext);
                mul_div.set_cpu_extensions(cpu_ext);
            }
            let sample_size = 100;

            bench(
                bench_group,
                sample_size,
                format!("fir {}", cpu_ext_into_str(cpu_ext)),
                alg_name,
                |bencher| {
                    fast_resizer.reset_internal_buffers();
                    match resize_alg {
                        ResizeAlg::Nearest => {
                            bencher.iter(|| {
                                fast_resizer.resize(&src_view, &mut dst_view).unwrap();
                            });
                        }
                        _ => {
                            bencher.iter(|| {
                                let mut premultiplied_view = premultiplied_src_image.view_mut();
                                mul_div
                                    .multiply_alpha(&src_view, &mut premultiplied_view)
                                    .unwrap();
                                fast_resizer
                                    .resize(&premultiplied_view.into(), &mut dst_view)
                                    .unwrap();
                                mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
                            });
                        }
                    }
                },
            );
        }
    }
}
