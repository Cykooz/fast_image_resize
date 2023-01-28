use std::num::NonZeroU32;

use fast_image_resize::MulDiv;
use fast_image_resize::PixelType;
use fast_image_resize::{CpuExtensions, Image};
use testing::cpu_ext_into_str;

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

fn multiplies_alpha(
    bench_group: &mut utils::BenchGroup,
    pixel_type: PixelType,
    cpu_extensions: CpuExtensions,
) {
    let sample_size = 100;
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let pixel: &[u8] = match pixel_type {
        PixelType::U8x4 => &[255, 128, 0, 128],
        PixelType::U8x2 => &[255, 128],
        PixelType::U16x2 => &[255, 255, 0, 128],
        PixelType::U16x4 => &[0, 255, 0, 128, 0, 0, 0, 128],
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

    utils::bench(
        bench_group,
        sample_size,
        format!("Multiplies alpha {pixel_type:?}"),
        cpu_ext_into_str(cpu_extensions),
        |bencher| {
            bencher.iter(|| {
                alpha_mul_div
                    .multiply_alpha(&src_view, &mut dst_view)
                    .unwrap();
            })
        },
    );

    let src_image = get_src_image(width, height, pixel_type, pixel);
    utils::bench(
        bench_group,
        sample_size,
        format!("Multiplies alpha inplace {pixel_type:?}"),
        cpu_ext_into_str(cpu_extensions),
        |bencher| {
            let mut image = src_image.copy();
            let mut view = image.view_mut();
            bencher.iter(|| {
                alpha_mul_div.multiply_alpha_inplace(&mut view).unwrap();
            })
        },
    );
}

fn divides_alpha(
    bench_group: &mut utils::BenchGroup,
    pixel_type: PixelType,
    cpu_extensions: CpuExtensions,
) {
    let sample_size = 100;
    let width = NonZeroU32::new(4095).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let pixel: &[u8] = match pixel_type {
        PixelType::U8x4 => &[128, 64, 0, 128],
        PixelType::U8x2 => &[128, 128],
        PixelType::U16x2 => &[0, 128, 0, 128],
        PixelType::U16x4 => &[0, 128, 0, 64, 0, 0, 0, 128],
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

    utils::bench(
        bench_group,
        sample_size,
        format!("Divides alpha {pixel_type:?}"),
        cpu_ext_into_str(cpu_extensions),
        |bencher| {
            bencher.iter(|| {
                alpha_mul_div
                    .divide_alpha(&src_view, &mut dst_view)
                    .unwrap();
            })
        },
    );

    let src_image = get_src_image(width, height, pixel_type, pixel);
    utils::bench(
        bench_group,
        sample_size,
        format!("Divides alpha inplace {pixel_type:?}"),
        cpu_ext_into_str(cpu_extensions),
        |bencher| {
            let mut image = src_image.copy();
            let mut view = image.view_mut();
            bencher.iter(|| {
                alpha_mul_div.divide_alpha_inplace(&mut view).unwrap();
            })
        },
    );
}

fn bench_alpha(bench_group: &mut utils::BenchGroup) {
    let pixel_types = [
        PixelType::U8x2,
        PixelType::U8x4,
        PixelType::U16x2,
        PixelType::U16x4,
    ];
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
    for pixel_type in pixel_types {
        for &cpu_ext in cpu_extensions.iter() {
            multiplies_alpha(bench_group, pixel_type, cpu_ext);
        }
    }
    for pixel_type in pixel_types {
        for &cpu_ext in cpu_extensions.iter() {
            divides_alpha(bench_group, pixel_type, cpu_ext);
        }
    }
}

fn main() {
    let res = utils::run_bench(bench_alpha, "Bench Alpha");
    println!("{}", utils::build_md_table(&res));
}
