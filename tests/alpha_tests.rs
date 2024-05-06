use fast_image_resize::images::{Image, TypedImage, TypedImageRef};
use fast_image_resize::{CpuExtensions, MulDiv, PixelTrait};
use testing::{cpu_ext_into_str, PixelTestingExt};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Oper {
    Mul,
    Div,
}

struct TestCaseU16 {
    pub color: u16,
    pub alpha: u16,
    pub expected_color: u16,
}

const fn new_case_16(c: u16, a: u16, e: u16) -> TestCaseU16 {
    TestCaseU16 {
        color: c,
        alpha: a,
        expected_color: e,
    }
}

fn full_mul_div_alpha_test_u8<P: PixelTrait<Component = u8>>(
    create_pixel: fn(u8, u8) -> P,
    cpu_extensions: CpuExtensions,
) {
    const PRECISION: u32 = 8;
    const ALPHA_SCALE: u32 = 255u32 * (1 << (PRECISION + 1));
    const ROUND_CORRECTION: u32 = 1 << (PRECISION - 1);

    for oper in [Oper::Mul, Oper::Div] {
        for color in 0u8..=255u8 {
            for alpha in 0u8..=255u8 {
                let result_color = if alpha == 0 {
                    0
                } else {
                    match oper {
                        Oper::Mul => {
                            let tmp = color as u32 * alpha as u32 + 128;
                            (((tmp >> 8) + tmp) >> 8) as u8
                        }
                        Oper::Div => {
                            let recip_alpha = ((ALPHA_SCALE / alpha as u32) + 1) >> 1;
                            let tmp = (color as u32 * recip_alpha + ROUND_CORRECTION) >> PRECISION;
                            tmp.min(255) as u8
                        }
                    }
                };
                let src = [create_pixel(color, alpha)];
                let res = [create_pixel(result_color, alpha)];
                mul_div_alpha_test(oper, &src, &res, cpu_extensions);
            }
        }
    }
}

fn mul_div_alpha_test<P: PixelTrait>(
    oper: Oper,
    src_pixels_tpl: &[P],
    expected_pixels_tpl: &[P],
    cpu_extensions: CpuExtensions,
) {
    assert_eq!(src_pixels_tpl.len(), expected_pixels_tpl.len());
    if !cpu_extensions.is_supported() {
        println!(
            "Cpu Extensions '{}' not supported by your CPU",
            cpu_ext_into_str(cpu_extensions)
        );
        return;
    }

    let width: u32 = 8 + 8 + 7;
    let height: u32 = 3;

    let src_size = width as usize * height as usize;
    let src_pixels: Vec<P> = src_pixels_tpl
        .iter()
        .copied()
        .cycle()
        .take(src_size)
        .collect();
    let mut dst_pixels = src_pixels.clone();

    let src_image = TypedImageRef::new(width, height, &src_pixels).unwrap();
    let mut dst_image = TypedImage::from_pixels_slice(width, height, &mut dst_pixels).unwrap();

    let mut alpha_mul_div: MulDiv = Default::default();
    unsafe {
        alpha_mul_div.set_cpu_extensions(cpu_extensions);
    }

    match oper {
        Oper::Mul => alpha_mul_div
            .multiply_alpha_typed(&src_image, &mut dst_image)
            .unwrap(),
        Oper::Div => alpha_mul_div
            .divide_alpha_typed(&src_image, &mut dst_image)
            .unwrap(),
    }

    let oper_str = if oper == Oper::Mul {
        "multiple"
    } else {
        "divide"
    };

    let expected_pixels: Vec<P> = expected_pixels_tpl
        .iter()
        .copied()
        .cycle()
        .take(src_size)
        .collect();
    for ((s, r), e) in src_pixels
        .iter()
        .zip(dst_pixels)
        .zip(expected_pixels.iter())
    {
        assert_eq!(
            r, *e,
            "failed test for {oper_str} alpha: src={s:?}, result={r:?}, expected_result={e:?}",
        );
    }

    // Inplace
    let mut src_pixels_clone = src_pixels.clone();
    let mut image = TypedImage::from_pixels_slice(width, height, &mut src_pixels_clone).unwrap();

    match oper {
        Oper::Mul => alpha_mul_div
            .multiply_alpha_inplace_typed(&mut image)
            .unwrap(),
        Oper::Div => alpha_mul_div
            .divide_alpha_inplace_typed(&mut image)
            .unwrap(),
    }

    for ((s, r), e) in src_pixels.iter().zip(src_pixels_clone).zip(expected_pixels) {
        assert_eq!(
            r, e,
            "failed inplace test for {oper_str} alpha: src={s:?}, result={r:?}, expected_result={e:?}",
        );
    }
}

fn run_tests_with_real_image_u8<P, const N: usize>(oper: Oper, expected_checksum: [u64; N])
where
    P: PixelTrait<Component = u8> + PixelTestingExt,
{
    let mut pixels = vec![0u8; 256 * 256 * N];
    let mut i: usize = 0;
    for alpha in 0..=255u8 {
        for color in 0..=255u8 {
            let pixel = pixels.get_mut(i..i + N).unwrap();
            for comp in pixel.iter_mut().take(N - 1) {
                *comp = color;
            }
            if let Some(c) = pixel.iter_mut().last() {
                *c = alpha;
            }
            i += N;
        }
    }
    let size = 256;
    let src_image = Image::from_vec_u8(size, size, pixels, P::pixel_type()).unwrap();
    let mut dst_image = Image::new(size, size, P::pixel_type());

    let mut alpha_mul_div: MulDiv = Default::default();

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    #[cfg(target_arch = "aarch64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Neon);
    }
    #[cfg(target_arch = "wasm32")]
    {
        cpu_extensions_vec.push(CpuExtensions::Simd128);
    }
    for cpu_extensions in cpu_extensions_vec {
        if !cpu_extensions.is_supported() {
            println!(
                "Cpu Extensions '{}' not supported by your CPU",
                cpu_ext_into_str(cpu_extensions)
            );
            continue;
        }
        unsafe {
            alpha_mul_div.set_cpu_extensions(cpu_extensions);
        }

        match oper {
            Oper::Mul => {
                alpha_mul_div
                    .multiply_alpha(&src_image, &mut dst_image)
                    .unwrap();
            }
            Oper::Div => {
                alpha_mul_div
                    .divide_alpha(&src_image, &mut dst_image)
                    .unwrap();
            }
        }

        let oper_str = if oper == Oper::Mul {
            "multiple"
        } else {
            "divide"
        };

        let pixel_type_str = P::pixel_type_str();
        let cpu_ext_str = cpu_ext_into_str(cpu_extensions);
        let name = format!("{oper_str}_alpha_{pixel_type_str}-{cpu_ext_str}");
        testing::save_result(&dst_image, &name);

        let checksum = testing::image_checksum::<P, N>(&dst_image);
        assert_eq!(
            checksum, expected_checksum,
            "failed test for {oper_str} alpha real image: \
            pixel_type={pixel_type_str}, cpu_extensions={cpu_ext_str}",
        );
    }
}

mod u8x4 {
    use fast_image_resize::pixels::U8x4;

    use super::*;

    const fn new_u8x4(c: u8, a: u8) -> U8x4 {
        U8x4::new([c, c, c, a])
    }

    #[test]
    fn native_test() {
        full_mul_div_alpha_test_u8(new_u8x4, CpuExtensions::None);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn sse4_test() {
        full_mul_div_alpha_test_u8(new_u8x4, CpuExtensions::Sse4_1);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx2_test() {
        full_mul_div_alpha_test_u8(new_u8x4, CpuExtensions::Avx2);
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_test() {
        full_mul_div_alpha_test_u8(new_u8x4, CpuExtensions::Neon);
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn wasm32_test() {
        full_mul_div_alpha_test_u8(new_u8x4, CpuExtensions::Simd128);
    }

    #[test]
    fn multiply_real_image() {
        run_tests_with_real_image_u8::<U8x4, 4>(Oper::Mul, [4177920, 4177920, 4177920, 8355840]);
    }

    #[test]
    fn divide_real_image() {
        run_tests_with_real_image_u8::<U8x4, 4>(Oper::Div, [12452343, 12452343, 12452343, 8355840]);
    }
}

#[cfg(not(feature = "only_u8x4"))]
mod not_u8x4 {
    use fast_image_resize::pixels::{U16x2, U16x4, U8x2};

    use super::*;

    const fn new_u16x2(l: u16, a: u16) -> U16x2 {
        U16x2::new([l, a])
    }

    const fn new_u16x4(c: u16, a: u16) -> U16x4 {
        U16x4::new([c, c, c, a])
    }

    fn get_div_test_cases_u16<P>(create_pixel: fn(u16, u16) -> P) -> (Vec<P>, Vec<P>)
    where
        P: PixelTrait<Component = u16>,
    {
        let test_cases = [
            new_case_16(0x8000, 0x8000, 0xffff),
            new_case_16(0x4000, 0x8000, 0x8000),
            new_case_16(0, 0x8000, 0),
            new_case_16(0xffff, 0xffff, 0xffff),
            new_case_16(0x8000, 0xffff, 0x8000),
            new_case_16(1, 2, 32768),
            new_case_16(0, 0xffff, 0),
            new_case_16(0xffff, 0, 0),
            new_case_16(0x8000, 0, 0),
            new_case_16(0, 0, 0),
        ];
        let mut scr_pixels = vec![];
        let mut expected_pixels = vec![];
        for case in test_cases {
            scr_pixels.push(create_pixel(case.color, case.alpha));
            expected_pixels.push(create_pixel(case.expected_color, case.alpha));
        }
        (scr_pixels, expected_pixels)
    }

    #[cfg(test)]
    mod u8x2 {
        use super::*;

        const fn new_u8x2(l: u8, a: u8) -> U8x2 {
            U8x2::new([l, a])
        }

        #[test]
        fn native_test() {
            full_mul_div_alpha_test_u8(new_u8x2, CpuExtensions::None);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn sse4_test() {
            full_mul_div_alpha_test_u8(new_u8x2, CpuExtensions::Sse4_1);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn avx2_test() {
            full_mul_div_alpha_test_u8(new_u8x2, CpuExtensions::Avx2);
        }

        #[cfg(target_arch = "aarch64")]
        #[test]
        fn neon_test() {
            full_mul_div_alpha_test_u8(new_u8x2, CpuExtensions::Neon);
        }

        #[cfg(target_arch = "wasm32")]
        #[test]
        fn wasm32_test() {
            full_mul_div_alpha_test_u8(new_u8x2, CpuExtensions::Simd128);
        }

        #[test]
        fn multiply_real_image() {
            run_tests_with_real_image_u8::<U8x2, 2>(Oper::Mul, [4177920, 8355840]);
        }

        #[test]
        fn divide_real_image() {
            run_tests_with_real_image_u8::<U8x2, 2>(Oper::Div, [12452343, 8355840]);
        }
    }

    #[cfg(test)]
    mod multiply_alpha_u16x2 {
        use fast_image_resize::pixels::U16x2;

        use super::*;

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
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Avx2);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn sse4_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Sse4_1);
        }

        #[cfg(target_arch = "aarch64")]
        #[test]
        fn neon_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Neon);
        }

        #[cfg(target_arch = "wasm32")]
        #[test]
        fn wasm32_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Simd128);
        }

        #[test]
        fn native_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::None);
        }
    }

    #[cfg(test)]
    mod multiply_alpha_u16x4 {
        use fast_image_resize::pixels::U16x4;

        use super::*;

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
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Avx2);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn sse4_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Sse4_1);
        }

        #[cfg(target_arch = "aarch64")]
        #[test]
        fn neon_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Neon);
        }

        #[cfg(target_arch = "wasm32")]
        #[test]
        fn wasm32_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::Simd128);
        }

        #[test]
        fn native_test() {
            mul_div_alpha_test(Oper::Mul, &SRC_PIXELS, &RES_PIXELS, CpuExtensions::None);
        }
    }

    #[cfg(test)]
    mod divide_alpha_u16x2 {
        use super::*;

        const OPER: Oper = Oper::Div;

        #[test]
        fn native_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x2);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::None);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn sse4_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x2);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Sse4_1);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn avx2_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x2);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Avx2);
        }

        #[cfg(target_arch = "aarch64")]
        #[test]
        fn neon_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x2);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Neon);
        }

        #[cfg(target_arch = "wasm32")]
        #[test]
        fn wasm32_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x2);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Simd128);
        }
    }

    #[cfg(test)]
    mod divide_alpha_u16x4 {
        use super::*;

        const OPER: Oper = Oper::Div;

        #[test]
        fn native_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x4);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::None);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn sse4_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x4);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Sse4_1);
        }

        #[cfg(target_arch = "x86_64")]
        #[test]
        fn avx2_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x4);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Avx2);
        }

        #[cfg(target_arch = "aarch64")]
        #[test]
        fn neon_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x4);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Neon);
        }

        #[cfg(target_arch = "wasm32")]
        #[test]
        fn wasm32_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_u16x4);
            mul_div_alpha_test(OPER, &scr_pixels, &expected_pixels, CpuExtensions::Simd128);
        }
    }
}
