use std::ops::Deref;

use fast_image_resize::images::Image;
use fast_image_resize::{FilterType, ResizeAlg, ResizeOptions, Resizer};
use image::{ImageBuffer, imageops};

use super::load_images::{PixelTestingExt, cpu_ext_into_str};
use super::utils::now;

const ALG_NAMES: [&str; 5] = ["Nearest", "Box", "Bilinear", "Bicubic", "Lanczos3"];
const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

/// Resize the image with the help of "image" crate (https://crates.io/crates/image)
pub fn image_resize<P, C>(
    src_image: &ImageBuffer<P, C>,
) -> impl Iterator<Item = (String, String, f64)>
where
    P: image::Pixel + 'static,
    C: Deref<Target = [P::Subpixel]>,
{
    let lib_name = "image".to_string();
    ALG_NAMES.iter().map(move |&alg_name| {
        let (filter, sample_size) = match alg_name {
            "Nearest" => (imageops::Nearest, 80),
            "Bilinear" => (imageops::Triangle, 50),
            "Bicubic" => (imageops::CatmullRom, 30),
            "Lanczos3" => (imageops::Lanczos3, 20),
            _ => return (lib_name.clone(), alg_name.to_string(), 0.0),
        };
        let start = now();
        for _ in 0..sample_size {
            imageops::resize(src_image, NEW_WIDTH, NEW_HEIGHT, filter);
        }
        let speed = (now() - start) / sample_size as f64;
        (lib_name.clone(), alg_name.to_string(), speed)
    })
}

/// Resize image with help of "resize" crate (https://crates.io/crates/resize)
pub fn resize_resize<Format, Out>(
    pixel_format: Format,
    src_image: &[Format::InputPixel],
    src_width: u32,
    src_height: u32,
) -> Vec<(String, String, f64)>
where
    Out: Clone,
    Format: resize::PixelFormat<OutputPixel = Out> + Copy,
{
    let lib_name = "resize".to_string();
    let mut results = Vec::with_capacity(ALG_NAMES.len());
    for alg_name in ALG_NAMES {
        let mut dst =
            vec![pixel_format.into_pixel(Format::new()); (NEW_WIDTH * NEW_HEIGHT) as usize];
        let sample_size = if alg_name == "Lanczos3" { 60 } else { 100 };

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
        let start = now();
        for _ in 0..sample_size {
            resizer.resize(src_image, &mut dst).unwrap();
        }
        let speed = (now() - start) / sample_size as f64;
        results.push((lib_name.clone(), alg_name.to_string(), speed));
    }
    results
}

/// Resize image with the help of "fast_imager_resize" crate
pub fn fir_resize<P: PixelTestingExt>(
    src_image_data: Image<'static>,
    use_alpha: bool,
) -> Vec<(String, String, f64)> {
    let mut results = Vec::with_capacity(ALG_NAMES.len());
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
            let sample_size = 100;
            let resize_options = ResizeOptions::new()
                .resize_alg(resize_alg)
                .use_alpha(use_alpha);
            let start = now();
            for _ in 0..sample_size {
                fast_resizer
                    .resize(&src_image_data, &mut dst_image, &resize_options)
                    .unwrap()
            }
            let speed = (now() - start) / sample_size as f64;
            results.push((lib_name.clone(), alg_name.to_string(), speed));
        }
    }
    results
}
