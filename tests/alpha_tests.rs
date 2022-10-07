use std::num::NonZeroU32;

use fast_image_resize::pixels::{PixelExt, U16x2, U16x4, U8x2, U8x4};
use fast_image_resize::{
    CpuExtensions, DynamicImageView, DynamicImageViewMut, Image, ImageView, ImageViewMut, MulDiv,
    PixelType,
};

const fn p2(l: u8, a: u8) -> U8x2 {
    U8x2::new(u16::from_le_bytes([l, a]))
}

const fn p4(r: u8, g: u8, b: u8, a: u8) -> U8x4 {
    U8x4::new(u32::from_le_bytes([r, g, b, a]))
}

enum Oper {
    Mul,
    Div,
}

// Multiplies by alpha

fn mul_div_alpha_test<P>(oper: Oper, src_pixel: P, result_pixel: P, cpu_extensions: CpuExtensions)
where
    P: PixelExt + 'static,
    for<'a> ImageView<'a, P>: Into<DynamicImageView<'a>>,
    for<'a> ImageViewMut<'a, P>: Into<DynamicImageViewMut<'a>>,
{
    let width: u32 = 8 + 8 + 7;
    let height: u32 = 3;

    let src_size = width as usize * height as usize;
    let mut src_pixels = vec![src_pixel; src_size];

    let src_dyn_image_view = ImageView::from_pixels(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        &src_pixels,
    )
    .unwrap()
    .into();

    let mut dst_image = Image::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        P::pixel_type(),
    );
    let mut dst_image_view = dst_image.view_mut();

    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    match oper {
        Oper::Mul => alpha_mul_div
            .multiply_alpha(&src_dyn_image_view, &mut dst_image_view)
            .unwrap(),
        Oper::Div => alpha_mul_div
            .divide_alpha(&src_dyn_image_view, &mut dst_image_view)
            .unwrap(),
    }

    let res_pixels: Vec<P> = vec![result_pixel; src_size];
    let res_buffer = unsafe { res_pixels.align_to::<u8>().1 };
    let dst_buffer = dst_image.buffer();
    assert!(
        dst_buffer.iter().zip(res_buffer).all(|(&d, &r)| d == r),
        "failed test: src={:?}, expected_result={:?}",
        src_pixel,
        result_pixel
    );

    // Inplace
    let mut image_view = ImageViewMut::from_pixels(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        &mut src_pixels,
    )
    .unwrap()
    .into();

    match oper {
        Oper::Mul => alpha_mul_div
            .multiply_alpha_inplace(&mut image_view)
            .unwrap(),
        Oper::Div => alpha_mul_div.divide_alpha_inplace(&mut image_view).unwrap(),
    }

    let src_buffer = unsafe { src_pixels.align_to::<u8>().1 };
    assert!(
        src_buffer.iter().zip(res_buffer).all(|(&s, &r)| s == r),
        "failed inplace test: src={:?}, expected_result={:?}",
        src_pixel,
        result_pixel
    );
}

#[cfg(test)]
mod multiply_alpha_u8x4 {
    use super::*;

    const SRC_PIXELS: [U8x4; 3] = [
        p4(255, 128, 0, 128),
        p4(255, 128, 0, 255),
        p4(255, 128, 0, 0),
    ];
    const RES_PIXELS: [U8x4; 3] = [p4(128, 64, 0, 128), p4(255, 128, 0, 255), p4(0, 0, 0, 0)];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::None);
        }
    }
}

#[cfg(test)]
mod multiply_alpha_u8x2 {
    use super::*;

    const SRC_PIXELS: [U8x2; 9] = [
        p2(255, 128),
        p2(128, 128),
        p2(0, 128),
        p2(255, 255),
        p2(128, 255),
        p2(0, 255),
        p2(255, 0),
        p2(128, 0),
        p2(0, 0),
    ];
    const RES_PIXELS: [U8x2; 9] = [
        p2(128, 128),
        p2(64, 128),
        p2(0, 128),
        p2(255, 255),
        p2(128, 255),
        p2(0, 255),
        p2(0, 0),
        p2(0, 0),
        p2(0, 0),
    ];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::None);
        }
    }
}

#[cfg(test)]
mod multiply_alpha_u16x2 {
    use super::*;
    use fast_image_resize::pixels::U16x2;

    const SRC_PIXELS: [U16x2; 9] = [
        U16x2::new([0xffff, 0x8000]),
        U16x2::new([0x8000, 0x8000]),
        U16x2::new([0, 0x8000]),
        U16x2::new([0xffff, 0xffff]),
        U16x2::new([0x8000, 0xffff]),
        U16x2::new([0, 0xffff]),
        U16x2::new([0xffff, 0]),
        U16x2::new([0x8000, 0]),
        U16x2::new([0, 0]),
    ];
    const RES_PIXELS: [U16x2; 9] = [
        U16x2::new([0x8000, 0x8000]),
        U16x2::new([0x4000, 0x8000]),
        U16x2::new([0, 0x8000]),
        U16x2::new([0xffff, 0xffff]),
        U16x2::new([0x8000, 0xffff]),
        U16x2::new([0, 0xffff]),
        U16x2::new([0, 0]),
        U16x2::new([0, 0]),
        U16x2::new([0, 0]),
    ];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::None);
        }
    }
}

#[cfg(test)]
mod multiply_alpha_u16x4 {
    use super::*;
    use fast_image_resize::pixels::U16x4;

    const SRC_PIXELS: [U16x4; 3] = [
        U16x4::new([0xffff, 0x8000, 0, 0x8000]),
        U16x4::new([0xffff, 0x8000, 0, 0xffff]),
        U16x4::new([0xffff, 0x8000, 0, 0]),
    ];
    const RES_PIXELS: [U16x4; 3] = [
        U16x4::new([0x8000, 0x4000, 0, 0x8000]),
        U16x4::new([0xffff, 0x8000, 0, 0xffff]),
        U16x4::new([0, 0, 0, 0]),
    ];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(Oper::Mul, s, r, CpuExtensions::None);
        }
    }
}

// Divides by alpha

#[cfg(test)]
mod divide_alpha_u8x4 {
    use super::*;

    const OPER: Oper = Oper::Div;
    const SRC_PIXELS: [U8x4; 3] = [
        p4(128, 64, 0, 128),
        p4(255, 128, 0, 255),
        p4(255, 128, 0, 0),
    ];
    const RES_PIXELS: [U8x4; 3] = [p4(255, 127, 0, 128), p4(255, 128, 0, 255), p4(0, 0, 0, 0)];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::None);
        }
    }
}

#[cfg(test)]
mod divide_alpha_u8x2 {
    use super::*;

    const OPER: Oper = Oper::Div;
    const SRC_PIXELS: [U8x2; 9] = [
        p2(128, 128),
        p2(64, 128),
        p2(0, 128),
        p2(255, 255),
        p2(128, 255),
        p2(0, 255),
        p2(255, 0),
        p2(128, 0),
        p2(0, 0),
    ];
    const RES_PIXELS: [U8x2; 9] = [
        p2(255, 128),
        p2(127, 128),
        p2(0, 128),
        p2(255, 255),
        p2(128, 255),
        p2(0, 255),
        p2(0, 0),
        p2(0, 0),
        p2(0, 0),
    ];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::None);
        }
    }
}

#[cfg(test)]
mod divide_alpha_u16x2 {
    use super::*;

    const OPER: Oper = Oper::Div;
    const SRC_PIXELS: [U16x2; 9] = [
        U16x2::new([0x8000, 0x8000]),
        U16x2::new([0x4000, 0x8000]),
        U16x2::new([0, 0x8000]),
        U16x2::new([0xffff, 0xffff]),
        U16x2::new([0x8000, 0xffff]),
        U16x2::new([0, 0xffff]),
        U16x2::new([0xffff, 0]),
        U16x2::new([0x8000, 0]),
        U16x2::new([0, 0]),
    ];
    const RES_PIXELS: [U16x2; 9] = [
        U16x2::new([0xffff, 0x8000]),
        U16x2::new([0x7fff, 0x8000]),
        U16x2::new([0, 0x8000]),
        U16x2::new([0xffff, 0xffff]),
        U16x2::new([0x8000, 0xffff]),
        U16x2::new([0, 0xffff]),
        U16x2::new([0, 0]),
        U16x2::new([0, 0]),
        U16x2::new([0, 0]),
    ];
    const SIMD_RES_PIXELS: [U16x2; 9] = [
        U16x2::new([0xffff, 0x8000]),
        U16x2::new([0x8000, 0x8000]),
        U16x2::new([0, 0x8000]),
        U16x2::new([0xffff, 0xffff]),
        U16x2::new([0x8000, 0xffff]),
        U16x2::new([0, 0xffff]),
        U16x2::new([0, 0]),
        U16x2::new([0, 0]),
        U16x2::new([0, 0]),
    ];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(SIMD_RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(SIMD_RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::None);
        }
    }
}

#[cfg(test)]
mod divide_alpha_u16x4 {
    use super::*;

    const OPER: Oper = Oper::Div;
    const SRC_PIXELS: [U16x4; 3] = [
        U16x4::new([0x8000, 0x4000, 0, 0x8000]),
        U16x4::new([0xffff, 0x8000, 0, 0xffff]),
        U16x4::new([0xffff, 0x8000, 0, 0]),
    ];
    const RES_PIXELS: [U16x4; 3] = [
        U16x4::new([0xffff, 0x7fff, 0, 0x8000]),
        U16x4::new([0xffff, 0x8000, 0, 0xffff]),
        U16x4::new([0, 0, 0, 0]),
    ];
    const SIMD_RES_PIXELS: [U16x4; 3] = [
        U16x4::new([0xffff, 0x8000, 0, 0x8000]),
        U16x4::new([0xffff, 0x8000, 0, 0xffff]),
        U16x4::new([0, 0, 0, 0]),
    ];

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(SIMD_RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Avx2);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(SIMD_RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::Sse4_1);
        }
    }

    #[test]
    fn native_test() {
        for (s, r) in SRC_PIXELS.into_iter().zip(RES_PIXELS) {
            mul_div_alpha_test(OPER, s, r, CpuExtensions::None);
        }
    }
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

        let name = format!(
            "multiple_alpha-{}",
            testing::cpu_ext_into_str(cpu_extensions)
        );
        testing::save_result(&dst_image, &name);

        let checksum = testing::image_checksum::<4>(dst_image.buffer());
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

        let name = format!("divide_alpha-{}", testing::cpu_ext_into_str(cpu_extensions));
        testing::save_result(&dst_image, &name);

        let checksum = testing::image_checksum::<4>(dst_image.buffer());
        assert_eq!(checksum, [8292504, 8292504, 8292504, 8355840]);
    }
}
