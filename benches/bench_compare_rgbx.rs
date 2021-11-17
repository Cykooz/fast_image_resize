use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::Image;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};

mod utils;

pub fn bench_downscale_rgbx(bench: &mut Bench) {
    let src_image = utils::get_big_rgb_image();
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let alg_names = ["Nearest", "Bilinear", "CatmullRom", "Lanczos3"];

    // fast_image_resize crate;
    let mut cpu_ext_and_name = vec![(CpuExtensions::None, "rust")];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_ext_and_name.push((CpuExtensions::Sse4_1, "sse4.1"));
        cpu_ext_and_name.push((CpuExtensions::Avx2, "avx2"));
    }
    for (cpu_ext, ext_name) in cpu_ext_and_name {
        for alg_name in alg_names {
            let src_rgba_image = utils::get_big_rgba_image();
            let src_image_data = Image::from_vec_u8(
                NonZeroU32::new(src_image.width()).unwrap(),
                NonZeroU32::new(src_image.height()).unwrap(),
                src_rgba_image.into_raw(),
                PixelType::U8x4,
            )
            .unwrap();
            let src_view = src_image_data.view();
            let mut dst_image = Image::new(new_width, new_height, PixelType::U8x4);
            let mut dst_view = dst_image.view_mut();

            let resize_alg = match alg_name {
                "Nearest" => ResizeAlg::Nearest,
                "Bilinear" => ResizeAlg::Convolution(FilterType::Bilinear),
                "CatmullRom" => ResizeAlg::Convolution(FilterType::CatmullRom),
                "Lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
                _ => return,
            };
            let mut fast_resizer = Resizer::new(resize_alg);

            unsafe {
                fast_resizer.reset_internal_buffers();
                fast_resizer.set_cpu_extensions(cpu_ext);
            }

            bench.task(format!("fir {} - {}", ext_name, alg_name), |task| {
                task.iter(|| {
                    fast_resizer.resize(&src_view, &mut dst_view).unwrap();
                })
            });
        }
    }

    utils::print_md_table(bench);
}

glassbench!("Compare resize of RGBx image", bench_downscale_rgbx,);
