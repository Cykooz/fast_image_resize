use std::num::NonZeroU32;

use fast_image_resize::pixels::U8x4;
use fast_image_resize::{
    CpuExtensions, Image, ImageRows, ImageRowsMut, ImageView, ImageViewMut, MulDiv, PixelType,
};
use utils::{cpu_ext_into_str, image_checksum};

mod utils;

const fn p(r: u8, g: u8, b: u8, a: u8) -> U8x4 {
    U8x4(u32::from_le_bytes([r, g, b, a]))
}

// Multiplies by alpha

fn multiply_alpha_test(cpu_extensions: CpuExtensions) {
    let width: u32 = 8 + 8 + 7;
    let height: u32 = 3;

    let src_pixels = [p(255, 128, 0, 128), p(255, 128, 0, 255), p(255, 128, 0, 0)];
    let res_pixels = [p(128, 64, 0, 128), p(255, 128, 0, 255), p(0, 0, 0, 0)];

    let mut src_rows: [Vec<U8x4>; 3] = [
        vec![src_pixels[0]; width as usize],
        vec![src_pixels[1]; width as usize],
        vec![src_pixels[2]; width as usize],
    ];

    let rows: Vec<&[U8x4]> = src_rows.iter().map(|r| r.as_ref()).collect();
    let src_image_view = ImageView::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        ImageRows::U8x4(rows),
    )
    .unwrap();

    let mut dst_image = Image::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        PixelType::U8x4,
    );
    let mut dst_image_view = dst_image.view_mut();

    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    alpha_mul_div
        .multiply_alpha(&src_image_view, &mut dst_image_view)
        .unwrap();

    let dst_pixels = unsafe { dst_image.buffer().align_to::<u32>().1 };
    let dst_rows = dst_pixels.chunks_exact(width as usize);
    for (row, &valid_pixel) in dst_rows.zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel, valid_pixel.0);
        }
    }

    // Inplace
    let rows: Vec<&mut [U8x4]> = src_rows.iter_mut().map(|r| r.as_mut()).collect();
    let mut image_view = ImageViewMut::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        ImageRowsMut::U8x4(rows),
    )
    .unwrap();
    alpha_mul_div
        .multiply_alpha_inplace(&mut image_view)
        .unwrap();

    for (row, &valid_pixel) in src_rows.iter().zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel, valid_pixel);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn multiply_alpha_avx2_test() {
    multiply_alpha_test(CpuExtensions::Avx2);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn multiply_alpha_sse4_test() {
    multiply_alpha_test(CpuExtensions::Sse4_1);
}

#[test]
fn multiply_alpha_native_test() {
    multiply_alpha_test(CpuExtensions::None);
}

// Divides by alpha

fn divide_alpha_test(cpu_extensions: CpuExtensions) {
    let width: u32 = 8 + 8 + 7;
    let height: u32 = 3;

    let src_pixels = [p(128, 64, 0, 128), p(255, 128, 0, 255), p(255, 128, 0, 0)];
    let res_pixels = [p(255, 127, 0, 128), p(255, 128, 0, 255), p(0, 0, 0, 0)];

    let mut src_rows: [Vec<U8x4>; 3] = [
        vec![src_pixels[0]; width as usize],
        vec![src_pixels[1]; width as usize],
        vec![src_pixels[2]; width as usize],
    ];

    let rows: Vec<&[U8x4]> = src_rows.iter().map(|r| r.as_ref()).collect();
    let src_image_view = ImageView::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        ImageRows::U8x4(rows),
    )
    .unwrap();

    let mut dst_image = Image::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        PixelType::U8x4,
    );
    let mut dst_image_view = dst_image.view_mut();

    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    alpha_mul_div
        .divide_alpha(&src_image_view, &mut dst_image_view)
        .unwrap();

    let dst_pixels = unsafe { dst_image.buffer().align_to::<u32>().1 };
    let dst_rows = dst_pixels.chunks_exact(width as usize);
    for (row, &valid_pixel) in dst_rows.zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel, valid_pixel.0);
        }
    }

    // Inplace
    let rows: Vec<&mut [U8x4]> = src_rows.iter_mut().map(|r| r.as_mut()).collect();
    let mut image_view = ImageViewMut::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        ImageRowsMut::U8x4(rows),
    )
    .unwrap();
    alpha_mul_div.divide_alpha_inplace(&mut image_view).unwrap();

    for (row, &valid_pixel) in src_rows.iter().zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel, valid_pixel);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn divide_alpha_avx2_test() {
    divide_alpha_test(CpuExtensions::Avx2);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn divide_alpha_sse4_test() {
    divide_alpha_test(CpuExtensions::Sse4_1);
}

#[test]
fn divide_alpha_native_test() {
    divide_alpha_test(CpuExtensions::None);
}

#[test]
fn multiply_alpha_real_image_test() {
    let mut pixels = vec![0u8; 256 * 256 * 4];
    let mut i: usize = 0;
    for alpha in 0..=255u8 {
        for color in 0..=255u8 {
            let pixel = pixels.get_mut(i..i + 4).unwrap();
            pixel.copy_from_slice(&[color, color, color, alpha]);
            i += 4;
        }
    }
    let size = NonZeroU32::new(256).unwrap();
    let src_image = Image::from_vec_u8(size, size, pixels, PixelType::U8x4).unwrap();
    let mut dst_image = Image::new(size, size, PixelType::U8x4);

    let mut alpha_mul_div: MulDiv = Default::default();

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        unsafe {
            alpha_mul_div.set_cpu_extensions(cpu_extensions);
        }
        alpha_mul_div
            .multiply_alpha(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let name = format!("multiple_alpha-{}", cpu_ext_into_str(cpu_extensions));
        utils::save_result(&dst_image, &name);

        let checksum = image_checksum::<4>(dst_image.buffer());
        assert_eq!(checksum, [4177920, 4177920, 4177920, 8355840]);
    }
}

#[test]
fn divide_alpha_real_image_test() {
    let mut pixels = vec![0u8; 256 * 256 * 4];
    let mut i: usize = 0;
    for alpha in 0..=255u8 {
        for color in 0..=255u8 {
            let multiplied_color = (color as f64 * (alpha as f64 / 255.)).round().min(255.) as u8;
            let pixel = pixels.get_mut(i..i + 4).unwrap();
            pixel.copy_from_slice(&[multiplied_color, multiplied_color, multiplied_color, alpha]);
            i += 4;
        }
    }
    let size = NonZeroU32::new(256).unwrap();
    let src_image = Image::from_vec_u8(size, size, pixels, PixelType::U8x4).unwrap();
    let mut dst_image = Image::new(size, size, PixelType::U8x4);

    let mut alpha_mul_div: MulDiv = Default::default();

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        unsafe {
            alpha_mul_div.set_cpu_extensions(cpu_extensions);
        }
        alpha_mul_div
            .divide_alpha(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let name = format!("divide_alpha-{}", cpu_ext_into_str(cpu_extensions));
        utils::save_result(&dst_image, &name);

        let checksum = image_checksum::<4>(dst_image.buffer());
        assert_eq!(checksum, [8292504, 8292504, 8292504, 8355840]);
    }
}
