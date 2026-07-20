use std::ops::Deref;

use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::{imageops, ImageBuffer};
use wasm_bindgen_test::Criterion;

use super::load_images::{cpu_ext_into_str, PixelTestingExt};

pub const ALG_NAMES: [&str; 5] = ["Nearest", "Box", "Bilinear", "Bicubic", "Lanczos3"];
const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

/// Resize the image with the help of "image" crate (https://crates.io/crates/image)
pub fn image_resize<P, C>(bencher: &mut Criterion, src_image: &ImageBuffer<P, C>)
where
    P: image::Pixel + 'static,
    C: Deref<Target = [P::Subpixel]>,
{
    for alg_name in ALG_NAMES {
        let filter = match alg_name {
            "Nearest" => imageops::Nearest,
            "Bilinear" => imageops::Triangle,
            "Bicubic" => imageops::CatmullRom,
            "Lanczos3" => imageops::Lanczos3,
            _ => return,
        };
        bencher.bench_function(&format!("image {alg_name}"), |b| {
            b.iter(|| imageops::resize(src_image, NEW_WIDTH, NEW_HEIGHT, filter))
        });
    }
}

/// Resize the image with the help of "resize" crate (https://crates.io/crates/resize)
pub fn resize_resize<Format, Out>(
    bencher: &mut Criterion,
    pixel_format: Format,
    src_image: &[Format::InputPixel],
    src_width: u32,
    src_height: u32,
) where
    Out: Clone,
    Format: resize::PixelFormat<OutputPixel = Out> + Copy,
{
    let mut dst = vec![pixel_format.into_pixel(Format::new()); (NEW_WIDTH * NEW_HEIGHT) as usize];

    for alg_name in ALG_NAMES {
        let filter = match alg_name {
            "Nearest" => resize::Type::Point,
            "Box" => resize::Type::Custom(resize::Filter::box_filter(0.5)),
            "Bilinear" => resize::Type::Triangle,
            "Bicubic" => resize::Type::Catrom,
            "Lanczos3" => resize::Type::Lanczos3,
            _ => continue,
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
        bencher.bench_function(&format!("resize {alg_name}"), |b| {
            b.iter(|| resizer.resize(src_image, &mut dst).unwrap())
        });
    }
}

/// Resize image with the help of "fast_imager_resize" crate
pub fn fir_resize<P: PixelTestingExt>(
    bencher: &mut Criterion,
    src_image_data: Image<'static>,
    use_alpha: bool,
) {
    let mut dst_image = Image::new(NEW_WIDTH, NEW_HEIGHT, src_image_data.pixel_type());
    let cpu_extensions = P::cpu_extensions();
    for cpu_ext in cpu_extensions {
        let lib_name = format!("fir {}", cpu_ext_into_str(cpu_ext));
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
            let resize_options = ResizeOptions::new()
                .resize_alg(resize_alg)
                .use_alpha(use_alpha);

            bencher.bench_function(&format!("{lib_name} {alg_name}"), |b| {
                b.iter(|| {
                    fast_resizer
                        .resize(&src_image_data, &mut dst_image, &resize_options)
                        .unwrap()
                })
            });
        }
    }
}
