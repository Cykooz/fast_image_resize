use fast_image_resize::{CpuExtensions, DstImageView, ImageData, MulDiv, PixelType, SrcImageView};
use std::num::NonZeroU32;

const fn p(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from_le_bytes([r, g, b, a])
}

// Multiplies by alpha

fn multiply_alpha_test(cpu_extensions: CpuExtensions) {
    let width: u32 = 8 + 8 + 7;
    let height: u32 = 3;

    let src_pixels = [p(255, 128, 0, 128), p(255, 128, 0, 255), p(255, 128, 0, 0)];
    let res_pixels = [p(128, 64, 0, 128), p(255, 128, 0, 255), p(0, 0, 0, 0)];

    let mut src_rows: [Vec<u32>; 3] = [
        vec![src_pixels[0]; width as usize],
        vec![src_pixels[1]; width as usize],
        vec![src_pixels[2]; width as usize],
    ];

    let rows: Vec<&[u32]> = src_rows.iter().map(|r| r.as_ref()).collect();
    let src_image_view = SrcImageView::from_rows(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        rows,
        PixelType::U8x4,
    )
    .unwrap();

    let mut dst_image = ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        PixelType::U8x4,
    );
    let mut dst_image_view = dst_image.dst_view();

    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    alpha_mul_div
        .multiply_alpha(&src_image_view, &mut dst_image_view)
        .unwrap();

    let dst_pixels: Vec<u32> = dst_image.get_pixels().to_vec();
    let dst_rows = dst_pixels.chunks_exact(width as usize);
    for (row, &valid_pixel) in dst_rows.zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel.to_le_bytes(), valid_pixel.to_le_bytes());
        }
    }

    // Inplace
    let rows: Vec<&mut [u32]> = src_rows.iter_mut().map(|r| r.as_mut()).collect();
    let mut image_view = DstImageView::from_rows(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        rows,
        PixelType::U8x4,
    )
    .unwrap();
    alpha_mul_div
        .multiply_alpha_inplace(&mut image_view)
        .unwrap();

    for (row, &valid_pixel) in src_rows.iter().zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel.to_le_bytes(), valid_pixel.to_le_bytes());
        }
    }
}

#[test]
fn multiply_alpha_avx2_test() {
    multiply_alpha_test(CpuExtensions::Avx2);
}

#[test]
fn multiply_alpha_sse2_test() {
    multiply_alpha_test(CpuExtensions::Sse2);
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

    let mut src_rows: [Vec<u32>; 3] = [
        vec![src_pixels[0]; width as usize],
        vec![src_pixels[1]; width as usize],
        vec![src_pixels[2]; width as usize],
    ];

    let rows: Vec<&[u32]> = src_rows.iter().map(|r| r.as_ref()).collect();
    let src_image_view = SrcImageView::from_rows(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        rows,
        PixelType::U8x4,
    )
    .unwrap();

    let mut dst_image = ImageData::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        PixelType::U8x4,
    );
    let mut dst_image_view = dst_image.dst_view();

    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    alpha_mul_div
        .divide_alpha(&src_image_view, &mut dst_image_view)
        .unwrap();

    let dst_pixels: Vec<u32> = dst_image.get_pixels().to_vec();
    let dst_rows = dst_pixels.chunks_exact(width as usize);
    for (row, &valid_pixel) in dst_rows.zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel.to_le_bytes(), valid_pixel.to_le_bytes());
        }
    }

    // Inplace
    let rows: Vec<&mut [u32]> = src_rows.iter_mut().map(|r| r.as_mut()).collect();
    let mut image_view = DstImageView::from_rows(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        rows,
        PixelType::U8x4,
    )
    .unwrap();
    alpha_mul_div.divide_alpha_inplace(&mut image_view).unwrap();

    for (row, &valid_pixel) in src_rows.iter().zip(res_pixels.iter()) {
        for &pixel in row.iter() {
            assert_eq!(pixel.to_le_bytes(), valid_pixel.to_le_bytes());
        }
    }
}

#[test]
fn divide_alpha_avx2_test() {
    divide_alpha_test(CpuExtensions::Avx2);
}

#[test]
fn divide_alpha_sse2_test() {
    divide_alpha_test(CpuExtensions::Sse2);
}

#[test]
fn divide_alpha_native_test() {
    divide_alpha_test(CpuExtensions::None);
}
