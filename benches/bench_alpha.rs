use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::MulDiv;
use fast_image_resize::PixelType;
use fast_image_resize::{CpuExtensions, Image};

const fn p(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from_le_bytes([r, g, b, a])
}

// Multiplies by alpha

fn get_src_image(width: NonZeroU32, height: NonZeroU32, pixel: u32) -> Image<'static> {
    let pixels_count = (width.get() * height.get()) as usize;
    let buffer = (0..pixels_count)
        .flat_map(|_| pixel.to_le_bytes())
        .collect();
    Image::from_vec_u8(width, height, buffer, PixelType::U8x4).unwrap()
}

#[cfg(target_arch = "x86_64")]
fn multiplies_alpha_avx2(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(255, 128, 0, 128));
    let mut dst_data = Image::new(width, height, PixelType::U8x4);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::Avx2);
    }

    bench.task("Multiplies alpha AVX2", |task| {
        task.iter(|| {
            alpha_mul_div
                .multiply_alpha(&src_view, &mut dst_view)
                .unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn multiplies_alpha_sse4(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(255, 128, 0, 128));
    let mut dst_data = Image::new(width, height, PixelType::U8x4);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::Sse4_1);
    }

    bench.task("Multiplies alpha SSE4.1", |task| {
        task.iter(|| {
            alpha_mul_div
                .multiply_alpha(&src_view, &mut dst_view)
                .unwrap();
        })
    });
}

fn multiplies_alpha_native(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(255, 128, 0, 128));
    let mut dst_data = Image::new(width, height, PixelType::U8x4);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::None);
    }

    bench.task("Multiplies alpha native", |task| {
        task.iter(|| {
            alpha_mul_div
                .multiply_alpha(&src_view, &mut dst_view)
                .unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn divides_alpha_avx2(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(128, 64, 0, 128));
    let mut dst_data = Image::new(width, height, PixelType::U8x4);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::Avx2);
    }

    bench.task("Divides alpha AVX2", |task| {
        task.iter(|| {
            alpha_mul_div
                .divide_alpha(&src_view, &mut dst_view)
                .unwrap();
        })
    });
}

#[cfg(target_arch = "x86_64")]
fn divides_alpha_sse4(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(128, 64, 0, 128));
    let mut dst_data = Image::new(width, height, PixelType::U8x4);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::Sse4_1);
    }

    bench.task("Divides alpha SSE4.1", |task| {
        task.iter(|| {
            alpha_mul_div
                .divide_alpha(&src_view, &mut dst_view)
                .unwrap();
        })
    });
}

fn divides_alpha_native(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(128, 64, 0, 128));
    let mut dst_data = Image::new(width, height, PixelType::U8x4);
    let src_view = src_data.view();
    let mut dst_view = dst_data.view_mut();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::None);
    }

    bench.task("Divides alpha native", |task| {
        task.iter(|| {
            alpha_mul_div
                .divide_alpha(&src_view, &mut dst_view)
                .unwrap();
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
        let mut bench = create_bench(name, "Alpha", &cmd);
        #[cfg(target_arch = "x86_64")]
        {
            multiplies_alpha_avx2(&mut bench);
            multiplies_alpha_sse4(&mut bench);
        }
        multiplies_alpha_native(&mut bench);
        #[cfg(target_arch = "x86_64")]
        {
            divides_alpha_avx2(&mut bench);
            divides_alpha_sse4(&mut bench);
        }
        divides_alpha_native(&mut bench);
        if let Err(e) = after_bench(&mut bench, &cmd) {
            eprintln!("{:?}", e);
        }
    } else {
        println!("skipping bench {:?}", &name);
    }
}
