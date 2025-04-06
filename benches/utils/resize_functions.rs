use std::ops::Deref;

use criterion::black_box;
use fast_image_resize::images::Image;
use fast_image_resize::{CpuExtensions, FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::{imageops, ImageBuffer};

use super::bencher::{bench, BenchGroup};
use super::testing::{cpu_ext_into_str, PixelTestingExt};

const ALG_NAMES: [&str; 5] = ["Nearest", "Box", "Bilinear", "Bicubic", "Lanczos3"];
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
            "Bicubic" => (imageops::CatmullRom, 30),
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
        let mut dst =
            vec![pixel_format.into_pixel(Format::new()); (NEW_WIDTH * NEW_HEIGHT) as usize];
        let sample_size = if alg_name == "Lanczos3" { 60 } else { 100 };

        bench(bench_group, sample_size, "resize", alg_name, |bencher| {
            let filter = match alg_name {
                "Nearest" => resize::Type::Point,
                "Box" => resize::Type::Custom(resize::Filter::box_filter(0.5)),
                "Bilinear" => resize::Type::Triangle,
                "Bicubic" => resize::Type::Catrom,
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

/// Resize image with help of "libvips" crate (https://crates.io/crates/libvips)

pub fn libvips_resize<P: PixelTestingExt>(bench_group: &mut BenchGroup, has_alpha: bool) {
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
    vips::libvips_resize_inner::<P>(bench_group, has_alpha);
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
mod vips {
    use libvips::ops::{self, BandFormat, Kernel, ReduceOptions};
    use libvips::{VipsApp, VipsImage};

    use super::*;

    const SAMPLE_SIZE: usize = 100;

    pub(crate) fn libvips_resize_inner<P: PixelTestingExt>(
        bench_group: &mut BenchGroup,
        has_alpha: bool,
    ) {
        let app = VipsApp::new("Test Libvips", false).expect("Cannot initialize libvips");
        let num_threads: u32 = std::env::var("RAYON_NUM_THREADS")
            .map(|s| s.parse().unwrap_or(1))
            .unwrap_or(1);
        if num_threads > 0 {
            app.concurrency_set(num_threads as i32);
        }
        app.cache_set_max(0);
        app.cache_set_max_mem(0);

        let src_image_data = P::load_big_src_image();
        let src_width = src_image_data.width() as i32;
        let src_height = src_image_data.height() as i32;
        let band_format = match P::count_of_component_values() {
            0x100 => BandFormat::Uchar,
            0x10000 => BandFormat::Ushort,
            0 => BandFormat::Float,
            _ => panic!("Unknown type of pixel"),
        };
        let src_vips_image = VipsImage::new_from_memory(
            src_image_data.buffer(),
            src_width,
            src_height,
            P::count_of_components() as i32,
            band_format,
        )
        .unwrap();
        let hshrink = src_width as f64 / NEW_WIDTH as f64;
        let vshrink = src_height as f64 / NEW_HEIGHT as f64;

        for alg_name in ALG_NAMES {
            let kernel = match alg_name {
                "Nearest" => Kernel::Nearest,
                "Box" => {
                    bench(bench_group, SAMPLE_SIZE, "libvips", alg_name, |bencher| {
                        if has_alpha {
                            bencher.iter(|| {
                                let premultiplied = ops::premultiply(&src_vips_image).unwrap();
                                let resized =
                                    ops::shrink(&premultiplied, hshrink, vshrink).unwrap();
                                let result = ops::unpremultiply(&resized).unwrap();
                                let result = ops::cast(&result, band_format).unwrap();
                                let res_bytes = result.image_write_to_memory();
                                black_box(&res_bytes);
                            })
                        } else {
                            bencher.iter(|| {
                                let result =
                                    ops::shrink(&src_vips_image, hshrink, vshrink).unwrap();
                                let res_bytes = result.image_write_to_memory();
                                black_box(&res_bytes);
                            })
                        }
                    });
                    continue;
                }
                "Bilinear" => Kernel::Linear,
                "Bicubic" => Kernel::Cubic,
                "Lanczos3" => Kernel::Lanczos3,
                _ => continue,
            };
            let options = ReduceOptions { kernel, gap: 0. };
            bench(bench_group, SAMPLE_SIZE, "libvips", alg_name, |bencher| {
                if has_alpha && alg_name != "Nearest" {
                    bencher.iter(|| {
                        let premultiplied = ops::premultiply(&src_vips_image).unwrap();
                        let resized =
                            ops::reduce_with_opts(&premultiplied, hshrink, vshrink, &options)
                                .unwrap();
                        let result = ops::unpremultiply(&resized).unwrap();
                        let result = ops::cast(&result, band_format).unwrap();
                        let res_bytes = result.image_write_to_memory();
                        black_box(&res_bytes);
                    })
                } else {
                    bencher.iter(|| {
                        let result =
                            ops::reduce_with_opts(&src_vips_image, hshrink, vshrink, &options)
                                .unwrap();
                        let res_bytes = result.image_write_to_memory();
                        black_box(&res_bytes);
                    })
                }
            });
        }
    }
}

/// Resize image with help of "fast_imager_resize" crate
pub fn fir_resize<P: PixelTestingExt>(bench_group: &mut BenchGroup, use_alpha: bool) {
    let src_image_data = P::load_big_src_image();
    let mut dst_image = Image::new(NEW_WIDTH, NEW_HEIGHT, src_image_data.pixel_type());
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
                "Nearest" => ResizeAlg::Nearest,
                "Box" => ResizeAlg::Convolution(FilterType::Box),
                "Bilinear" => ResizeAlg::Convolution(FilterType::Bilinear),
                "Bicubic" => ResizeAlg::Convolution(FilterType::CatmullRom),
                "Lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
                _ => continue,
            };
            let mut fast_resizer = Resizer::new();
            unsafe {
                fast_resizer.set_cpu_extensions(cpu_ext);
            }
            let sample_size = 100;
            let resize_options = ResizeOptions::new()
                .resize_alg(resize_alg)
                .use_alpha(use_alpha);

            bench(
                bench_group,
                sample_size,
                format!("fir {}", cpu_ext_into_str(cpu_ext)),
                alg_name,
                |bencher| {
                    bencher.iter(|| {
                        fast_resizer
                            .resize(&src_image_data, &mut dst_image, &resize_options)
                            .unwrap()
                    })
                },
            );
        }
    }
}
