use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::pixels::U8x2;
use fast_image_resize::{CpuExtensions, FilterType, Image, MulDiv, PixelType, ResizeAlg, Resizer};
use testing::PixelExt;

mod utils;

pub fn bench_downscale_la(bench: &mut Bench) {
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
            let resize_alg = match alg_name {
                "Nearest" => ResizeAlg::Nearest,
                "Bilinear" => ResizeAlg::Convolution(FilterType::Bilinear),
                "CatmullRom" => ResizeAlg::Convolution(FilterType::CatmullRom),
                "Lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
                _ => return,
            };
            let src_image = U8x2::load_big_src_image();
            let src_view = src_image.view();
            let mut premultiplied_src_image =
                Image::new(src_image.width(), src_image.height(), src_view.pixel_type());
            let mut dst_image = Image::new(new_width, new_height, PixelType::U8x2);
            let mut dst_view = dst_image.view_mut();
            let mut mul_div = MulDiv::default();

            let mut fast_resizer = Resizer::new(resize_alg);

            unsafe {
                fast_resizer.reset_internal_buffers();
                fast_resizer.set_cpu_extensions(cpu_ext);
                mul_div.set_cpu_extensions(cpu_ext);
            }

            bench.task(
                format!("fir {} - {}", ext_name, alg_name),
                |task| match resize_alg {
                    ResizeAlg::Nearest => {
                        task.iter(|| {
                            fast_resizer.resize(&src_view, &mut dst_view).unwrap();
                        });
                    }
                    _ => {
                        task.iter(|| {
                            mul_div
                                .multiply_alpha(&src_view, &mut premultiplied_src_image.view_mut())
                                .unwrap();
                            fast_resizer
                                .resize(&premultiplied_src_image.view(), &mut dst_view)
                                .unwrap();
                            mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
                        });
                    }
                },
            );
        }
    }

    utils::print_md_table(bench);
}

bench_main!("Compare resize of LA image", bench_downscale_la,);
