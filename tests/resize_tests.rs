use std::cmp::Ordering;
use std::fmt::Debug;

use fast_image_resize::pixels::*;
use fast_image_resize::{
    testing as fr_testing, CpuExtensions, CropBox, DifferentTypesOfPixelsError, DynamicImageView,
    FilterType, Image, PixelType, ResizeAlg, Resizer,
};
use testing::{cpu_ext_into_str, nonzero, PixelTestingExt};

fn get_new_height(src_image: &DynamicImageView, new_width: u32) -> u32 {
    let scale = new_width as f32 / src_image.width().get() as f32;
    (src_image.height().get() as f32 * scale).round() as u32
}

const NEW_WIDTH: u32 = 255;
const NEW_BIG_WIDTH: u32 = 5016;

#[test]
fn try_resize_to_other_pixel_type() {
    let src_image = U8x4::load_big_src_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    let mut dst_image = Image::new(nonzero(1024), nonzero(256), PixelType::U8);
    assert!(matches!(
        resizer.resize(&src_image.view(), &mut dst_image.view_mut()),
        Err(DifferentTypesOfPixelsError)
    ));
}

#[test]
fn resize_to_same_size() {
    let width = nonzero(100);
    let height = nonzero(80);
    let buffer: Vec<u8> = (0..8000).map(|v| (v & 0xff) as u8).collect();
    let src_image = Image::from_vec_u8(width, height, buffer, PixelType::U8).unwrap();
    let mut dst_image = Image::new(width, height, PixelType::U8);
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    resizer
        .resize(&src_image.view(), &mut dst_image.view_mut())
        .unwrap();
    assert!(matches!(
        src_image.buffer().cmp(dst_image.buffer()),
        Ordering::Equal
    ));
}

#[test]
fn resize_to_same_size_after_cropping() {
    let width = nonzero(100);
    let height = nonzero(80);
    let src_width = nonzero(120);
    let src_height = nonzero(100);
    let buffer: Vec<u8> = (0..12000).map(|v| (v & 0xff) as u8).collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, PixelType::U8).unwrap();
    let mut src_view = src_image.view();
    src_view
        .set_crop_box(CropBox {
            top: 10,
            left: 10,
            width,
            height,
        })
        .unwrap();

    let mut dst_image = Image::new(width, height, PixelType::U8);
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    resizer
        .resize(&src_view, &mut dst_image.view_mut())
        .unwrap();

    let cropped_buffer: Vec<u8> = (0..12000u32)
        .filter_map(|v| {
            let row = v / 120;
            let col = v % 120;
            if (10..90u32).contains(&row) && (10..110u32).contains(&col) {
                Some((v & 0xff) as u8)
            } else {
                None
            }
        })
        .collect();
    let dst_buffer = dst_image.into_vec();
    assert!(matches!(cropped_buffer.cmp(&dst_buffer), Ordering::Equal));
}

/// In this test, we check that resizer won't use horizontal convolution
/// if width of destination image is equal to width of cropped source image.
fn resize_to_same_width<const C: usize>(
    pixel_type: PixelType,
    cpu_extensions: CpuExtensions,
    create_pixel: fn(v: u8) -> [u8; C],
) {
    fr_testing::clear_log();
    let width = nonzero(100);
    let height = nonzero(80);
    let src_width = nonzero(120);
    let src_height = nonzero(100);
    // Image columns are made up of pixels of the same color.
    let buffer: Vec<u8> = (0..12000)
        .flat_map(|v| create_pixel((v % 120) as u8))
        .collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, pixel_type).unwrap();
    let mut src_view = src_image.view();
    src_view
        .set_crop_box(CropBox {
            left: 10,
            top: 0,
            width,
            height: src_height,
        })
        .unwrap();

    let mut dst_image = Image::new(width, height, pixel_type);
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    resizer
        .resize(&src_view, &mut dst_image.view_mut())
        .unwrap();
    let expected_result: Vec<u8> = (0..8000u32)
        .flat_map(|v| create_pixel((10 + v % 100) as u8))
        .collect();
    let dst_buffer = dst_image.into_vec();
    assert!(
        matches!(expected_result.cmp(&dst_buffer), Ordering::Equal),
        "Resizing result is not equal to expected ones ({:?}, {:?})",
        pixel_type,
        cpu_extensions
    );

    assert!(fr_testing::logs_contain(
        "compute vertical convolution coefficients"
    ));
    assert!(!fr_testing::logs_contain(
        "compute horizontal convolution coefficients"
    ));
}

/// In this test, we check that resizer won't use vertical convolution
/// if height of destination image is equal to height of cropped source image.
fn resize_to_same_height<const C: usize>(
    pixel_type: PixelType,
    cpu_extensions: CpuExtensions,
    create_pixel: fn(v: u8) -> [u8; C],
) {
    fr_testing::clear_log();
    let width = nonzero(100);
    let height = nonzero(80);
    let src_width = nonzero(120);
    let src_height = nonzero(100);
    // Image rows are made up of pixels of the same color.
    let buffer: Vec<u8> = (0..12000)
        .flat_map(|v| create_pixel((v / 120) as u8))
        .collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, pixel_type).unwrap();
    let mut src_view = src_image.view();
    src_view
        .set_crop_box(CropBox {
            left: 0,
            top: 10,
            width: src_width,
            height,
        })
        .unwrap();

    let mut dst_image = Image::new(width, height, pixel_type);
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    resizer
        .resize(&src_view, &mut dst_image.view_mut())
        .unwrap();
    let expected_result: Vec<u8> = (0..8000u32)
        .flat_map(|v| create_pixel((10 + v / 100) as u8))
        .collect();
    let dst_buffer = dst_image.into_vec();
    assert!(
        matches!(expected_result.cmp(&dst_buffer), Ordering::Equal),
        "Resizing result is not equal to expected ones ({:?}, {:?})",
        pixel_type,
        cpu_extensions
    );

    assert!(!fr_testing::logs_contain(
        "compute vertical convolution coefficients"
    ));
    assert!(fr_testing::logs_contain(
        "compute horizontal convolution coefficients"
    ));
}

#[test]
fn resize_to_same_width_after_cropping() {
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
    for cpu_extensions in cpu_extensions_vec {
        if !cpu_extensions.is_supported() {
            continue;
        }
        resize_to_same_width(PixelType::U8, cpu_extensions, |v| [v]);
        resize_to_same_width(PixelType::U8x2, cpu_extensions, |v| [v; 2]);
        resize_to_same_width(PixelType::U8x3, cpu_extensions, |v| [v; 3]);
        resize_to_same_width(PixelType::U8x4, cpu_extensions, |v| [v; 4]);
        resize_to_same_width(PixelType::U16, cpu_extensions, |v| [v, 0]);
        resize_to_same_width(PixelType::U16x2, cpu_extensions, |v| [v, 0, v, 0]);
        resize_to_same_width(PixelType::U16x3, cpu_extensions, |v| [v, 0, v, 0, v, 0]);
        resize_to_same_width(PixelType::U16x4, cpu_extensions, |v| {
            [v, 0, v, 0, v, 0, v, 0]
        });
        resize_to_same_height(PixelType::U8, cpu_extensions, |v| [v]);
        resize_to_same_height(PixelType::U8x2, cpu_extensions, |v| [v; 2]);
        resize_to_same_height(PixelType::U8x3, cpu_extensions, |v| [v; 3]);
        resize_to_same_height(PixelType::U8x4, cpu_extensions, |v| [v; 4]);
        resize_to_same_height(PixelType::U16, cpu_extensions, |v| [v, 0]);
        resize_to_same_height(PixelType::U16x2, cpu_extensions, |v| [v, 0, v, 0]);
        resize_to_same_height(PixelType::U16x3, cpu_extensions, |v| [v, 0, v, 0, v, 0]);
        resize_to_same_height(PixelType::U16x4, cpu_extensions, |v| {
            [v, 0, v, 0, v, 0, v, 0]
        });
    }
    resize_to_same_width(PixelType::I32, CpuExtensions::None, |v| {
        (v as i32).to_le_bytes()
    });
    resize_to_same_width(PixelType::F32, CpuExtensions::None, |v| {
        (v as f32).to_le_bytes()
    });
    resize_to_same_height(PixelType::I32, CpuExtensions::None, |v| {
        (v as i32).to_le_bytes()
    });
    resize_to_same_height(PixelType::F32, CpuExtensions::None, |v| {
        (v as f32).to_le_bytes()
    });
}

trait ResizeTest<const CC: usize> {
    fn downscale_test(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions, checksum: [u64; CC]);
    fn upscale_test(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions, checksum: [u64; CC]);
}

impl<T, C, const CC: usize> ResizeTest<CC> for Pixel<T, C, CC>
where
    Self: PixelTestingExt,
    T: Sized + Copy + Clone + Debug + PartialEq + 'static,
    C: PixelComponent,
{
    fn downscale_test(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions, checksum: [u64; CC]) {
        if !cpu_extensions.is_supported() {
            println!(
                "Cpu Extensions '{}' not supported by your CPU",
                cpu_ext_into_str(cpu_extensions)
            );
            return;
        }

        let image = Self::load_big_src_image();
        assert_eq!(image.pixel_type(), Self::pixel_type());

        let mut resizer = Resizer::new(resize_alg);
        unsafe {
            resizer.set_cpu_extensions(cpu_extensions);
        }
        let image_view = image.view();
        let new_height = get_new_height(&image_view, NEW_WIDTH);
        let mut result = Image::new(nonzero(NEW_WIDTH), nonzero(new_height), image.pixel_type());
        assert!(resizer.resize(&image_view, &mut result.view_mut()).is_ok());

        let alg_name = match resize_alg {
            ResizeAlg::Nearest => "nearest",
            ResizeAlg::Convolution(filter) => match filter {
                FilterType::Box => "box",
                FilterType::Bilinear => "bilinear",
                FilterType::Hamming => "hamming",
                FilterType::Mitchell => "mitchell",
                FilterType::CatmullRom => "catmullrom",
                FilterType::Lanczos3 => "lanczos3",
                _ => "unknown",
            },
            ResizeAlg::SuperSampling(_, _) => "supersampling",
            _ => "unknown",
        };

        let name = format!(
            "downscale-{}-{}-{}",
            Self::pixel_type_str(),
            alg_name,
            cpu_ext_into_str(cpu_extensions),
        );
        testing::save_result(&result, &name);
        assert_eq!(
            testing::image_checksum::<Self, CC>(&result),
            checksum,
            "Error in checksum for {:?}",
            cpu_extensions
        );
    }

    fn upscale_test(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions, checksum: [u64; CC]) {
        if !cpu_extensions.is_supported() {
            println!(
                "Cpu Extensions '{}' not supported by your CPU",
                cpu_ext_into_str(cpu_extensions)
            );
            return;
        }

        let image = Self::load_small_src_image();
        assert_eq!(image.pixel_type(), Self::pixel_type());

        let mut resizer = Resizer::new(resize_alg);
        unsafe {
            resizer.set_cpu_extensions(cpu_extensions);
        }
        let new_height = get_new_height(&image.view(), NEW_BIG_WIDTH);
        let mut result = Image::new(
            nonzero(NEW_BIG_WIDTH),
            nonzero(new_height),
            image.pixel_type(),
        );
        assert!(resizer
            .resize(&image.view(), &mut result.view_mut())
            .is_ok());

        let alg_name = match resize_alg {
            ResizeAlg::Nearest => "nearest",
            ResizeAlg::Convolution(filter) => match filter {
                FilterType::Box => "box",
                FilterType::Bilinear => "bilinear",
                FilterType::Hamming => "hamming",
                FilterType::Mitchell => "mitchell",
                FilterType::CatmullRom => "catmullrom",
                FilterType::Lanczos3 => "lanczos3",
                _ => "unknown",
            },
            ResizeAlg::SuperSampling(_, _) => "supersampling",
            _ => "unknown",
        };

        let name = format!(
            "upscale-{}-{}-{}",
            Self::pixel_type_str(),
            alg_name,
            cpu_ext_into_str(cpu_extensions),
        );
        testing::save_result(&result, &name);
        assert_eq!(
            testing::image_checksum::<Self, CC>(&result),
            checksum,
            "Error in checksum for {:?}",
            cpu_extensions
        );
    }
}

#[test]
fn downscale_u8() {
    type P = U8;
    P::downscale_test(ResizeAlg::Nearest, CpuExtensions::None, [2920348]);

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [2923557],
        );
    }
}

#[test]
fn upscale_u8() {
    type P = U8;
    P::upscale_test(ResizeAlg::Nearest, CpuExtensions::None, [1148754010]);

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [1148811406],
        );
    }
}

#[test]
fn downscale_u8x2() {
    type P = U8x2;
    P::downscale_test(ResizeAlg::Nearest, CpuExtensions::None, [2920348, 6121802]);

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [2923557, 6122818],
        );
    }
}

#[test]
fn upscale_u8x2() {
    type P = U8x2;
    P::upscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [1146218632, 2364895380],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [1146283728, 2364890194],
        );
    }
}

#[test]
fn downscale_u8x3() {
    type P = U8x3;
    P::downscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [2937940, 2945380, 2882679],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [2942479, 2947850, 2885072],
        );
    }
}

#[test]
fn upscale_u8x3() {
    type P = U8x3;
    P::upscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [1156008260, 1158417906, 1135087540],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [1156107005, 1158443335, 1135101759],
        );
    }
}

#[test]
fn downscale_u8x4() {
    type P = U8x4;
    P::downscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [2937940, 2945380, 2882679, 6121802],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [2942479, 2947850, 2885072, 6122818],
        );

        P::downscale_test(
            ResizeAlg::SuperSampling(FilterType::Lanczos3, 2),
            cpu_extensions,
            [2942546, 2947627, 2884866, 6123158],
        );
    }
}

#[test]
fn upscale_u8x4() {
    type P = U8x4;
    P::upscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [1155096957, 1152644783, 1123285879, 2364895380],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [1155201788, 1152688479, 1123328716, 2364890194],
        );
    }
}

#[test]
fn downscale_u16() {
    type P = U16;
    P::downscale_test(ResizeAlg::Nearest, CpuExtensions::None, [750529436]);

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
        cpu_extensions_vec.push(CpuExtensions::Wasm32);
    }
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [751401243],
        );
    }
}

#[test]
fn upscale_u16() {
    type P = U16;
    P::upscale_test(ResizeAlg::Nearest, CpuExtensions::None, [295229780570]);

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [295246940755],
        );
    }
}

#[test]
fn downscale_u16x2() {
    type P = U16x2;
    P::downscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [750529436, 1573303114],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [751401243, 1573563971],
        );
    }
}

#[test]
fn upscale_u16x2() {
    type P = U16x2;
    P::upscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [294578188424, 607778112660],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [294597368766, 607776760273],
        );
    }
}

#[test]
fn downscale_u16x3() {
    type P = U16x3;
    P::downscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [755050580, 756962660, 740848503],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [756269847, 757632467, 741478612],
        );
    }
}

#[test]
fn upscale_u16x3() {
    type P = U16x3;
    P::upscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [297094122820, 297713401842, 291717497780],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [297122154090, 297723994984, 291725294637],
        );
    }
}

#[test]
fn downscale_u16x4() {
    type P = U16x4;
    P::downscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [755050580, 756962660, 740848503, 1573303114],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [756269847, 757632467, 741478612, 1573563971],
        );
    }
}

#[test]
fn upscale_u16x4() {
    type P = U16x4;
    P::upscale_test(
        ResizeAlg::Nearest,
        CpuExtensions::None,
        [296859917949, 296229709231, 288684470903, 607778112660],
    );

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
    for cpu_extensions in cpu_extensions_vec {
        P::upscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            cpu_extensions,
            [296888688348, 296243667797, 288698172180, 607776760273],
        );
    }
}
