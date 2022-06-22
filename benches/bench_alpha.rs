use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::MulDiv;
use fast_image_resize::PixelType;
use fast_image_resize::{CpuExtensions, Image};

mod utils;

// Multiplies by alpha

fn get_src_image(
    width: NonZeroU32,
    height: NonZeroU32,
    pixel_type: PixelType,
    pixel: &[u8],
) -> Image<'static> {
    let pixels_count = (width.get() * height.get()) as usize;
    let buffer = (0..pixels_count)
        .flat_map(|_| pixel.iter().copied())
        .collect();
    Image::from_vec_u8(width, height, buffer, pixel_type).unwrap()
}

fn multiplies_alpha(bench: &mut Bench, pixel_type: PixelType, cpu_extensions: CpuExtensions) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let pixel: &[u8] = match pixel_type {
        PixelType::U8x4 => &[255, 128, 0, 128],
        PixelType::U8x2 => &[255, 128],
        PixelType::U16x2 => &[255, 255, 0, 128],
        _ => unreachable!(),
    };
    let src_data = get_src_image(width, height, pixel_type, pixel);
    let mut dst_data = Image::new(width, height, pixel_type);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    bench.task(
        format!("Multiplies alpha {:?} {:?}", pixel_type, cpu_extensions),
        |task| {
            task.iter(|| {
                alpha_mul_div
                    .multiply_alpha(&src_view, &mut dst_view)
                    .unwrap();
            })
        },
    );
}

fn divides_alpha(bench: &mut Bench, pixel_type: PixelType, cpu_extensions: CpuExtensions) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let pixel: &[u8] = match pixel_type {
        PixelType::U8x4 => &[128, 64, 0, 128],
        PixelType::U8x2 => &[128, 128],
        PixelType::U16x2 => &[0, 128, 0, 128],
        _ => unreachable!(),
    };
    let src_data = get_src_image(width, height, pixel_type, pixel);
    let mut dst_data = Image::new(width, height, pixel_type);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    bench.task(
        format!("Divides alpha {:?} {:?}", pixel_type, cpu_extensions),
        |task| {
            task.iter(|| {
                alpha_mul_div
                    .divide_alpha(&src_view, &mut dst_view)
                    .unwrap();
            })
        },
    );
}

fn bench_alpha(bench: &mut Bench) {
    let pixel_types = [PixelType::U8x4, PixelType::U8x2, PixelType::U16x2];
    let mut cpu_extensions = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions.push(CpuExtensions::Sse4_1);
        cpu_extensions.push(CpuExtensions::Avx2);
    }
    for pixel_type in pixel_types {
        for &extensions in cpu_extensions.iter() {
            println!("Mul {:?} {:?}", pixel_type, extensions);
            multiplies_alpha(bench, pixel_type, extensions);
        }
    }
    for pixel_type in pixel_types {
        for &extensions in cpu_extensions.iter() {
            println!("Div {:?} {:?}", pixel_type, extensions);
            divides_alpha(bench, pixel_type, extensions);
        }
    }
}

bench_main!("Bench Alpha", bench_alpha,);
