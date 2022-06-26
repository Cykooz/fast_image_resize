use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::pixels::*;
use fast_image_resize::Image;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};
use testing::{cpu_ext_into_str, PixelExt};

mod utils;

const NEW_WIDTH: u32 = 852;
const NEW_HEIGHT: u32 = 567;

fn native_nearest_u8x4_bench(bench: &mut Bench) {
    let image = U8x4::load_big_src_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("nearest wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn downscale_bench(
    bench: &mut Bench,
    image: &Image<'static>,
    cpu_extensions: CpuExtensions,
    filter_type: FilterType,
) {
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(filter_type));
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    let bench_name = &format!(
        "{:?}-{:?}-{}",
        image.pixel_type(),
        filter_type,
        cpu_ext_into_str(cpu_extensions),
    );
    bench.task(bench_name, |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

fn native_nearest_u8_bench(bench: &mut Bench) {
    let image = U8::load_big_src_image();
    let mut res_image = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(NEW_HEIGHT).unwrap(),
        image.pixel_type(),
    );
    let src_image = image.view();
    let mut dst_image = res_image.view_mut();
    let mut resizer = Resizer::new(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    bench.task("u8 nearest wo SIMD", |task| {
        task.iter(|| {
            resizer.resize(&src_image, &mut dst_image).unwrap();
        })
    });
}

pub fn main() {
    // Pin process to #0 CPU core
    let mut cpu_set = nix::sched::CpuSet::new();
    cpu_set.set(0).unwrap();
    nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();

    use glassbench::*;
    let name = env!("CARGO_CRATE_NAME");
    let cmd = Command::read();
    if cmd.include_bench(name) {
        let mut bench = create_bench(name, "Resize", &cmd);

        let pixel_types = [
            PixelType::U8,
            PixelType::U8x2,
            PixelType::U8x3,
            PixelType::U8x4,
            PixelType::U16,
            PixelType::U16x2,
            PixelType::U16x3,
            PixelType::U16x4,
            PixelType::I32,
        ];
        let mut cpu_extensions = vec![CpuExtensions::None];
        #[cfg(target_arch = "x86_64")]
        {
            cpu_extensions.push(CpuExtensions::Sse4_1);
            cpu_extensions.push(CpuExtensions::Avx2);
        }
        for pixel_type in pixel_types {
            for &cpu_extension in cpu_extensions.iter() {
                let image = match pixel_type {
                    PixelType::U8 => U8::load_big_src_image(),
                    PixelType::U8x2 => U8x2::load_big_src_image(),
                    PixelType::U8x3 => U8x3::load_big_src_image(),
                    PixelType::U8x4 => U8x4::load_big_src_image(),
                    PixelType::U16 => U16::load_big_src_image(),
                    PixelType::U16x2 => U16x2::load_big_src_image(),
                    PixelType::U16x3 => U16x3::load_big_src_image(),
                    PixelType::U16x4 => U16x4::load_big_src_image(),
                    PixelType::I32 => I32::load_big_src_image(),
                    _ => unreachable!(),
                };
                downscale_bench(&mut bench, &image, cpu_extension, FilterType::Lanczos3);
            }
        }

        native_nearest_u8x4_bench(&mut bench);
        native_nearest_u8_bench(&mut bench);

        if let Err(e) = after_bench(&mut bench, &cmd) {
            eprintln!("{:?}", e);
        }
    } else {
        println!("skipping bench {:?}", &name);
    }
}
