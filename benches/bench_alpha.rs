use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::MulDiv;
use fast_image_resize::PixelType;
use fast_image_resize::{CpuExtensions, Image};

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
            multiplies_alpha(&mut bench, PixelType::U8x4, CpuExtensions::Avx2);
            multiplies_alpha(&mut bench, PixelType::U8x4, CpuExtensions::Sse4_1);
            multiplies_alpha(&mut bench, PixelType::U8x2, CpuExtensions::Avx2);
            multiplies_alpha(&mut bench, PixelType::U8x2, CpuExtensions::Sse4_1);
        }
        multiplies_alpha(&mut bench, PixelType::U8x4, CpuExtensions::None);
        multiplies_alpha(&mut bench, PixelType::U8x2, CpuExtensions::None);

        #[cfg(target_arch = "x86_64")]
        {
            divides_alpha(&mut bench, PixelType::U8x4, CpuExtensions::Avx2);
            divides_alpha(&mut bench, PixelType::U8x4, CpuExtensions::Sse4_1);
            divides_alpha(&mut bench, PixelType::U8x2, CpuExtensions::Avx2);
            divides_alpha(&mut bench, PixelType::U8x2, CpuExtensions::Sse4_1);
        }
        divides_alpha(&mut bench, PixelType::U8x4, CpuExtensions::None);
        divides_alpha(&mut bench, PixelType::U8x2, CpuExtensions::None);
        if let Err(e) = after_bench(&mut bench, &cmd) {
            eprintln!("{:?}", e);
        }
    } else {
        println!("skipping bench {:?}", &name);
    }
}
