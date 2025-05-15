use fast_image_resize::images::{Image, TypedImage, TypedImageRef};
use fast_image_resize::{CpuExtensions, MulDiv, PixelTrait};
use testing::{cpu_ext_into_str, PixelTestingExt};
mod testing;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Oper {
    Mul,
    Div,
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

    let height: u32 = 3;
    for width in [1, 9, 17, 25, 33, 41, 49, 57, 65] {
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

        let cpu_ext_str = cpu_ext_into_str(cpu_extensions);
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
                "failed test for {oper_str} alpha with '{cpu_ext_str}' CPU extensions \
                and image width {width}: src={s:?}, result={r:?}, expected_result={e:?}",
            );
        }

        // Inplace
        let mut src_pixels_clone = src_pixels.clone();
        let mut image =
            TypedImage::from_pixels_slice(width, height, &mut src_pixels_clone).unwrap();

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
                "failed inplace test for {oper_str} alpha with '{cpu_ext_str}' CPU extensions \
                and image width {width}: src={s:?}, result={r:?}, expected_result={e:?}",
            );
        }
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

    for cpu_extensions in P::cpu_extensions() {
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

mod u8_tests {
    use super::*;

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
                                let tmp =
                                    (color as u32 * recip_alpha + ROUND_CORRECTION) >> PRECISION;
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

    #[cfg(not(feature = "only_u8x4"))]
    #[cfg(test)]
    mod u8x2 {
        use fast_image_resize::pixels::U8x2;

        use super::*;

        type P = U8x2;

        const fn new_pixel(l: u8, a: u8) -> P {
            P::new([l, a])
        }

        #[test]
        fn mul_div_alpha_test() {
            for cpu_extensions in P::cpu_extensions() {
                full_mul_div_alpha_test_u8(new_pixel, cpu_extensions);
            }
        }

        #[test]
        fn multiply_real_image() {
            run_tests_with_real_image_u8::<P, 2>(Oper::Mul, [4177920, 8355840]);
        }

        #[test]
        fn divide_real_image() {
            run_tests_with_real_image_u8::<P, 2>(Oper::Div, [12452343, 8355840]);
        }
    }

    #[cfg(test)]
    mod u8x4 {
        use fast_image_resize::pixels::U8x4;

        use super::*;

        type P = U8x4;

        const fn new_pixel(c: u8, a: u8) -> P {
            P::new([c, c, c, a])
        }

        #[test]
        fn mul_div_alpha_test() {
            for cpu_extensions in P::cpu_extensions() {
                full_mul_div_alpha_test_u8(new_pixel, cpu_extensions);
            }
        }

        #[test]
        fn multiply_real_image() {
            run_tests_with_real_image_u8::<P, 4>(Oper::Mul, [4177920, 4177920, 4177920, 8355840]);
        }

        #[test]
        fn divide_real_image() {
            run_tests_with_real_image_u8::<P, 4>(
                Oper::Div,
                [12452343, 12452343, 12452343, 8355840],
            );
        }
    }
}

#[cfg(not(feature = "only_u8x4"))]
mod u16_tests {
    use super::*;

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

    fn get_mul_test_cases_u16<P>(create_pixel: fn(u16, u16) -> P) -> (Vec<P>, Vec<P>)
    where
        P: PixelTrait<Component = u16>,
    {
        let test_cases = [
            new_case_16(0xffff, 0x8000, 0x8000),
            new_case_16(0x8000, 0x8000, 0x4000),
            new_case_16(0, 0x8000, 0),
            new_case_16(0xffff, 0xffff, 0xffff),
            new_case_16(0x8000, 0xffff, 0x8000),
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
            new_case_16(0xffff, 0xc0c0, 0xffff),
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
    mod u16x2 {
        use fast_image_resize::pixels::U16x2;

        use super::*;

        type P = U16x2;

        const fn new_pixel(l: u16, a: u16) -> P {
            P::new([l, a])
        }

        #[test]
        fn multiple_alpha_test() {
            let (scr_pixels, expected_pixels) = get_mul_test_cases_u16(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Mul, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }

        #[test]
        fn divide_alpha_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Div, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }
    }

    #[cfg(test)]
    mod u16x4 {
        use fast_image_resize::pixels::U16x4;

        use super::*;

        type P = U16x4;

        const fn new_pixel(c: u16, a: u16) -> P {
            P::new([c, c, c, a])
        }

        #[test]
        fn multiple_alpha_test() {
            let (scr_pixels, expected_pixels) = get_mul_test_cases_u16(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Mul, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }

        #[test]
        fn divide_alpha_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_u16(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Div, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }
    }
}

#[cfg(not(feature = "only_u8x4"))]
mod f32_tests {
    use super::*;

    struct TestCaseF32 {
        pub color: f32,
        pub alpha: f32,
        pub expected_color: f32,
    }

    const fn new_case_f32(c: f32, a: f32, e: f32) -> TestCaseF32 {
        TestCaseF32 {
            color: c,
            alpha: a,
            expected_color: e,
        }
    }

    fn get_mul_test_cases_f32<P>(create_pixel: fn(f32, f32) -> P) -> (Vec<P>, Vec<P>)
    where
        P: PixelTrait<Component = f32>,
    {
        let test_cases = [
            new_case_f32(1., 0.5, 0.5),
            new_case_f32(0.5, 0.5, 0.25),
            new_case_f32(0., 0.5, 0.),
            new_case_f32(1., 1., 1.),
            new_case_f32(0.5, 1., 0.5),
            new_case_f32(0., 1., 0.),
            new_case_f32(1., 0., 0.),
            new_case_f32(0.5, 0., 0.),
            new_case_f32(0., 0., 0.),
        ];
        let mut scr_pixels = vec![];
        let mut expected_pixels = vec![];
        for case in test_cases {
            scr_pixels.push(create_pixel(case.color, case.alpha));
            expected_pixels.push(create_pixel(case.expected_color, case.alpha));
        }
        (scr_pixels, expected_pixels)
    }

    fn get_div_test_cases_f32<P>(create_pixel: fn(f32, f32) -> P) -> (Vec<P>, Vec<P>)
    where
        P: PixelTrait<Component = f32>,
    {
        let test_cases = [
            new_case_f32(0.5, 0.5, 1.),
            new_case_f32(0.25, 0.5, 0.5),
            new_case_f32(0., 0.5, 0.),
            new_case_f32(1., 1., 1.),
            new_case_f32(0.5, 1., 0.5),
            new_case_f32(0.00001, 0.00002, 0.00001 / 0.00002),
            new_case_f32(0., 1., 0.),
            new_case_f32(1., 0., 0.),
            new_case_f32(0.5, 0., 0.),
            new_case_f32(0., 0., 0.),
            // f32 can afford to have a value greater than 1.0
            new_case_f32(1., 0.7, 1. / 0.7),
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
    mod f32x2 {
        use fast_image_resize::pixels::F32x2;

        use super::*;

        type P = F32x2;

        const fn new_pixel(c: f32, a: f32) -> P {
            P::new([c, a])
        }

        #[test]
        fn multiple_alpha_test() {
            let (scr_pixels, expected_pixels) = get_mul_test_cases_f32(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Mul, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }

        #[test]
        fn divide_alpha_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_f32(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Div, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }
    }

    #[cfg(test)]
    mod f32x4 {
        use fast_image_resize::pixels::F32x4;

        use super::*;

        type P = F32x4;

        const fn new_pixel(c: f32, a: f32) -> P {
            P::new([c, c, c, a])
        }

        #[test]
        fn multiple_alpha_test() {
            let (scr_pixels, expected_pixels) = get_mul_test_cases_f32(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Mul, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }

        #[test]
        fn divide_alpha_test() {
            let (scr_pixels, expected_pixels) = get_div_test_cases_f32(new_pixel);
            for cpu_extensions in P::cpu_extensions() {
                mul_div_alpha_test(Oper::Div, &scr_pixels, &expected_pixels, cpu_extensions);
            }
        }
    }
}
