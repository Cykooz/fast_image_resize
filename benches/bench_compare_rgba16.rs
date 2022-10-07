use glassbench::*;
use resize::px::RGBA;
use resize::Pixel::RGBA16P;
use rgb::FromSlice;
use std::num::NonZeroU32;

use fast_image_resize::pixels::U16x4;
use fast_image_resize::{CpuExtensions, FilterType, Image, MulDiv, ResizeAlg, Resizer};
use testing::PixelTestingExt;

mod utils;

pub fn bench_downscale_rgba16(bench: &mut Bench) {
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let alg_names = ["Nearest", "Bilinear", "CatmullRom", "Lanczos3"];

    // resize crate
    // https://crates.io/crates/resize
    let src_image = U16x4::load_big_image().to_rgba16();
    for alg_name in alg_names {
        let resize_src_image = src_image.as_raw().as_rgba();
        let mut dst =
            vec![RGBA::new(0u16, 0u16, 0u16, 0u16); (new_width.get() * new_height.get()) as usize];
        bench.task(format!("resize - {}", alg_name), |task| {
            let filter = match alg_name {
                "Nearest" => {
                    // resizer doesn't support "nearest" algorithm
                    task.iter(|| {});
                    return;
                }
                "Bilinear" => resize::Type::Triangle,
                "CatmullRom" => resize::Type::Catrom,
                "Lanczos3" => resize::Type::Lanczos3,
                _ => return,
            };
            let mut resize = resize::new(
                src_image.width() as usize,
                src_image.height() as usize,
                new_width.get() as usize,
                new_height.get() as usize,
                RGBA16P,
                filter,
            )
            .unwrap();
            task.iter(|| {
                resize.resize(resize_src_image, &mut dst).unwrap();
            })
        });
    }

    // fast_image_resize crate;
    let src_image = U16x4::load_big_src_image();
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
            let mut dst_image = Image::new(new_width, new_height, src_image.pixel_type());
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
                            fast_resizer
                                .resize(&src_image.view(), &mut dst_view)
                                .unwrap();
                        });
                    }
                    _ => {
                        let mut work_image = src_image.copy();
                        task.iter(|| {
                            mul_div
                                .multiply_alpha_inplace(&mut work_image.view_mut())
                                .unwrap();
                            fast_resizer
                                .resize(&work_image.view(), &mut dst_view)
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

bench_main!("Compare resize of RGBA16 image", bench_downscale_rgba16,);
