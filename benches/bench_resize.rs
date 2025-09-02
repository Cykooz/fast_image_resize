use fast_image_resize::images::Image;
use fast_image_resize::pixels::*;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use utils::testing::{cpu_ext_into_str, PixelTestingExt};

mod utils;

const NEW_SIZE: u32 = 695;

fn native_nearest_u8x4_bench(bench_group: &mut utils::BenchGroup) {
    let src_image = U8x4::load_big_square_src_image();
    let mut dst_image = Image::new(NEW_SIZE, NEW_SIZE, PixelType::U8x4);
    let mut resizer = Resizer::new();
    let options = ResizeOptions::new().resize_alg(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    utils::bench(bench_group, 100, "U8x4 Nearest", "rust", |bencher| {
        bencher.iter(|| {
            resizer
                .resize(&src_image, &mut dst_image, &options)
                .unwrap()
        })
    });
}

#[cfg(not(feature = "only_u8x4"))]
fn native_nearest_u8_bench(bench_group: &mut utils::BenchGroup) {
    let src_image = U8::load_big_square_src_image();
    let mut dst_image = Image::new(NEW_SIZE, NEW_SIZE, PixelType::U8);
    let mut resizer = Resizer::new();
    let options = ResizeOptions::new().resize_alg(ResizeAlg::Nearest);
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    utils::bench(bench_group, 100, "U8 Nearest", "rust", |bencher| {
        bencher.iter(|| {
            resizer
                .resize(&src_image, &mut dst_image, &options)
                .unwrap()
        })
    });
}

fn downscale_bench(
    bench_group: &mut utils::BenchGroup,
    image: &Image<'static>,
    cpu_extensions: CpuExtensions,
    filter_type: FilterType,
    dst_width: u32,
    dst_height: u32,
    name_prefix: &str,
) {
    let mut res_image = Image::new(dst_width, dst_height, image.pixel_type());
    let mut resizer = Resizer::new();
    let options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(filter_type))
        .use_alpha(false);
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    let prefix = if name_prefix.is_empty() {
        "".to_string()
    } else {
        format!(" {}", name_prefix)
    };
    utils::bench(
        bench_group,
        100,
        format!("{:?} {:?}", image.pixel_type(), filter_type),
        format!("{}{}", cpu_ext_into_str(cpu_extensions), prefix),
        |bencher| bencher.iter(|| resizer.resize(image, &mut res_image, &options).unwrap()),
    );
}

pub fn resize_in_one_dimension_bench(bench_group: &mut utils::BenchGroup) {
    let pixel_types = [
        PixelType::U8,
        PixelType::U8x2,
        PixelType::U8x3,
        PixelType::U8x4,
        PixelType::U16,
        PixelType::U16x2,
        PixelType::U16x3,
        PixelType::U16x4,
        PixelType::F32,
        PixelType::F32x2,
        PixelType::F32x3,
        PixelType::F32x4,
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
        #[cfg(feature = "only_u8x4")]
        if pixel_type != PixelType::U8x4 {
            continue;
        }
        for &cpu_extension in cpu_extensions.iter() {
            #[cfg(not(feature = "only_u8x4"))]
            let image = match pixel_type {
                PixelType::U8 => U8::load_big_square_src_image(),
                PixelType::U8x2 => U8x2::load_big_square_src_image(),
                PixelType::U8x3 => U8x3::load_big_square_src_image(),
                PixelType::U8x4 => U8x4::load_big_square_src_image(),
                PixelType::U16 => U16::load_big_square_src_image(),
                PixelType::U16x2 => U16x2::load_big_square_src_image(),
                PixelType::U16x3 => U16x3::load_big_square_src_image(),
                PixelType::U16x4 => U16x4::load_big_square_src_image(),
                PixelType::I32 => I32::load_big_square_src_image(),
                PixelType::F32 => F32::load_big_square_src_image(),
                PixelType::F32x2 => F32x2::load_big_square_src_image(),
                PixelType::F32x3 => F32x3::load_big_square_src_image(),
                PixelType::F32x4 => F32x4::load_big_square_src_image(),
                _ => unreachable!(),
            };
            #[cfg(feature = "only_u8x4")]
            let image = match pixel_type {
                PixelType::U8x4 => U8x4::load_big_square_src_image(),
                _ => unreachable!(),
            };
            downscale_bench(
                bench_group,
                &image,
                cpu_extension,
                FilterType::Lanczos3,
                NEW_SIZE,
                image.height(),
                "H",
            );
            downscale_bench(
                bench_group,
                &image,
                cpu_extension,
                FilterType::Lanczos3,
                image.height(),
                NEW_SIZE,
                "V",
            );
        }
    }
}

pub fn resize_bench(bench_group: &mut utils::BenchGroup) {
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
        PixelType::F32,
        PixelType::F32x2,
        PixelType::F32x3,
        PixelType::F32x4,
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
        #[cfg(feature = "only_u8x4")]
        if pixel_type != PixelType::U8x4 {
            continue;
        }
        for &cpu_extension in cpu_extensions.iter() {
            #[cfg(not(feature = "only_u8x4"))]
            let image = match pixel_type {
                PixelType::U8 => U8::load_big_square_src_image(),
                PixelType::U8x2 => U8x2::load_big_square_src_image(),
                PixelType::U8x3 => U8x3::load_big_square_src_image(),
                PixelType::U8x4 => U8x4::load_big_square_src_image(),
                PixelType::U16 => U16::load_big_square_src_image(),
                PixelType::U16x2 => U16x2::load_big_square_src_image(),
                PixelType::U16x3 => U16x3::load_big_square_src_image(),
                PixelType::U16x4 => U16x4::load_big_square_src_image(),
                PixelType::I32 => I32::load_big_square_src_image(),
                PixelType::F32 => F32::load_big_square_src_image(),
                PixelType::F32x2 => F32x2::load_big_square_src_image(),
                PixelType::F32x3 => F32x3::load_big_square_src_image(),
                PixelType::F32x4 => F32x4::load_big_square_src_image(),
                _ => unreachable!(),
            };
            #[cfg(feature = "only_u8x4")]
            let image = match pixel_type {
                PixelType::U8x4 => U8x4::load_big_square_src_image(),
                _ => unreachable!(),
            };
            downscale_bench(
                bench_group,
                &image,
                cpu_extension,
                FilterType::Lanczos3,
                NEW_SIZE,
                NEW_SIZE,
                "",
            );
        }
    }

    native_nearest_u8x4_bench(bench_group);
    #[cfg(not(feature = "only_u8x4"))]
    native_nearest_u8_bench(bench_group);
}

fn main1() {
    let results = utils::run_bench(resize_bench, "Resize");
    println!("{}", utils::build_md_table(&results));
}

fn main() {
    let results = utils::run_bench(resize_in_one_dimension_bench, "Resize one dimension");
    println!("{}", utils::build_md_table(&results));
}
