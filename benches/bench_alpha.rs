use std::num::NonZeroU32;

use glassbench::*;

use fast_image_resize::{CpuExtensions, ImageData, MulDiv, PixelType};

const fn p(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from_le_bytes([r, g, b, a])
}

// Multiplies by alpha

fn get_src_image(width: NonZeroU32, height: NonZeroU32, pixel: u32) -> ImageData<'static> {
    let buf_size = (width.get() * height.get()) as usize;
    let buffer = vec![pixel; buf_size];
    ImageData::from_vec_u32(width, height, buffer, PixelType::U8x4).unwrap()
}

fn multiplies_alpha_avx2(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(255, 128, 0, 128));
    let mut dst_data = ImageData::new(width, height, PixelType::U8x4);
    let src_view = src_data.src_view();
    let mut dst_view = dst_data.dst_view();
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

fn multiplies_alpha_sse2(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(255, 128, 0, 128));
    let mut dst_data = ImageData::new(width, height, PixelType::U8x4);
    let src_view = src_data.src_view();
    let mut dst_view = dst_data.dst_view();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::Sse2);
    }

    bench.task("Multiplies alpha SSE2", |task| {
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
    let mut dst_data = ImageData::new(width, height, PixelType::U8x4);
    let src_view = src_data.src_view();
    let mut dst_view = dst_data.dst_view();
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

fn divides_alpha_avx2(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(128, 64, 0, 128));
    let mut dst_data = ImageData::new(width, height, PixelType::U8x4);
    let src_view = src_data.src_view();
    let mut dst_view = dst_data.dst_view();
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

fn divides_alpha_sse2(bench: &mut Bench) {
    let width = NonZeroU32::new(4096).unwrap();
    let height = NonZeroU32::new(2048).unwrap();
    let src_data = get_src_image(width, height, p(128, 64, 0, 128));
    let mut dst_data = ImageData::new(width, height, PixelType::U8x4);
    let src_view = src_data.src_view();
    let mut dst_view = dst_data.dst_view();
    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(CpuExtensions::Sse2);
    }

    bench.task("Divides alpha SSE2", |task| {
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
    let mut dst_data = ImageData::new(width, height, PixelType::U8x4);
    let src_view = src_data.src_view();
    let mut dst_view = dst_data.dst_view();
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

glassbench!(
    "Alpha",
    multiplies_alpha_avx2,
    multiplies_alpha_sse2,
    multiplies_alpha_native,
    divides_alpha_avx2,
    divides_alpha_sse2,
    divides_alpha_native,
);
