use std::cmp::Ordering;
use std::fmt::Debug;

use image::io::Reader as ImageReader;

use fast_image_resize::pixels::*;
use fast_image_resize::{
    testing as fr_testing, CpuExtensions, CropBox, CropBoxError, DifferentTypesOfPixelsError,
    DynamicImageView, Filter, FilterType, Image, ImageView, PixelType, ResizeAlg, Resizer,
};
use testing::{cpu_ext_into_str, image_checksum, nonzero, save_result, PixelTestingExt};

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
    let buffer: Vec<u8> = (0..8000)
        .map(|v| (v & 0xff) as u8)
        .flat_map(|v| [v; 4])
        .collect();
    let src_image = Image::from_vec_u8(width, height, buffer, PixelType::U8x4).unwrap();
    let mut dst_image = Image::new(width, height, PixelType::U8x4);
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
    let buffer: Vec<u8> = (0..12000)
        .map(|v| (v & 0xff) as u8)
        .flat_map(|v| [v; 4])
        .collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, PixelType::U8x4).unwrap();
    let mut src_view = src_image.view();
    src_view
        .set_crop_box(CropBox {
            top: 10.,
            left: 10.,
            width: width.get() as _,
            height: height.get() as _,
        })
        .unwrap();

    let mut dst_image = Image::new(width, height, PixelType::U8x4);
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
        .flat_map(|v| [v; 4])
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
            left: 10.,
            top: 0.,
            width: width.get() as _,
            height: src_height.get() as _,
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
            left: 0.,
            top: 10.,
            width: src_width.get() as _,
            height: height.get() as _,
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
    #[cfg(target_arch = "wasm32")]
    {
        cpu_extensions_vec.push(CpuExtensions::Simd128);
    }
    for cpu_extensions in cpu_extensions_vec {
        if !cpu_extensions.is_supported() {
            continue;
        }
        resize_to_same_width(PixelType::U8x4, cpu_extensions, |v| [v; 4]);
        #[cfg(not(feature = "only_u8x4"))]
        {
            resize_to_same_width(PixelType::U8, cpu_extensions, |v| [v]);
            resize_to_same_width(PixelType::U8x2, cpu_extensions, |v| [v; 2]);
            resize_to_same_width(PixelType::U8x3, cpu_extensions, |v| [v; 3]);
            resize_to_same_width(PixelType::U16, cpu_extensions, |v| [v, 0]);
            resize_to_same_width(PixelType::U16x2, cpu_extensions, |v| [v, 0, v, 0]);
            resize_to_same_width(PixelType::U16x3, cpu_extensions, |v| [v, 0, v, 0, v, 0]);
            resize_to_same_width(PixelType::U16x4, cpu_extensions, |v| {
                [v, 0, v, 0, v, 0, v, 0]
            });
        }
        resize_to_same_height(PixelType::U8x4, cpu_extensions, |v| [v; 4]);
        #[cfg(not(feature = "only_u8x4"))]
        {
            resize_to_same_height(PixelType::U8, cpu_extensions, |v| [v]);
            resize_to_same_height(PixelType::U8x2, cpu_extensions, |v| [v; 2]);
            resize_to_same_height(PixelType::U8x3, cpu_extensions, |v| [v; 3]);
            resize_to_same_height(PixelType::U16, cpu_extensions, |v| [v, 0]);
            resize_to_same_height(PixelType::U16x2, cpu_extensions, |v| [v, 0, v, 0]);
            resize_to_same_height(PixelType::U16x3, cpu_extensions, |v| [v, 0, v, 0, v, 0]);
            resize_to_same_height(PixelType::U16x4, cpu_extensions, |v| {
                [v, 0, v, 0, v, 0, v, 0]
            });
        }
    }
    #[cfg(not(feature = "only_u8x4"))]
    {
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
            "Error in checksum for {cpu_extensions:?}",
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

#[cfg(not(feature = "only_u8x4"))]
mod not_u8x4 {
    use super::*;

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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [2923555],
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [1148811829],
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [2923555, 6122718],
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [1146284886, 2364890085],
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [2942547, 2947799, 2885025],
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [1156107445, 1158443938, 1135102297],
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
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [296888688348, 296243667797, 288698172180, 607776760273],
            );
        }
    }

    #[test]
    fn fractional_cropping() {
        let mut src_buf = [0, 0, 0, 0, 255, 0, 0, 0, 0];
        let src_image =
            Image::from_slice_u8(nonzero(3), nonzero(3), &mut src_buf, PixelType::U8).unwrap();
        let mut dst_image = Image::new(nonzero(1), nonzero(1), PixelType::U8);
        let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Box));

        // Resize without cropping
        resizer
            .resize(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();
        assert_eq!(dst_image.buffer()[0], (255.0f32 / 9.0).round() as u8);

        // Resize with fractional cropping
        let mut src_view = src_image.view();
        src_view
            .set_crop_box(CropBox {
                left: 0.5,
                top: 0.5,
                width: 2.,
                height: 2.,
            })
            .unwrap();
        resizer
            .resize(&src_view, &mut dst_image.view_mut())
            .unwrap();
        assert_eq!(dst_image.buffer()[0], (255.0f32 / 4.0).round() as u8);

        // Resize with integer cropping
        src_view
            .set_crop_box(CropBox {
                left: 1.,
                top: 1.,
                width: 1.,
                height: 1.,
            })
            .unwrap();
        resizer
            .resize(&src_view, &mut dst_image.view_mut())
            .unwrap();
        assert_eq!(dst_image.buffer()[0], 255);
    }
}

mod u8x4 {
    use std::f64::consts::PI;

    use super::*;

    type P = U8x4;

    #[test]
    fn set_crop_box() {
        let mut view: ImageView<P> =
            ImageView::from_buffer(nonzero(1), nonzero(1), &[0, 0, 0, 0]).unwrap();

        for (left, top) in [(1., 0.), (0., 1.)] {
            assert_eq!(
                view.set_crop_box(CropBox {
                    left,
                    top,
                    width: 1.,
                    height: 1.,
                }),
                Err(CropBoxError::PositionIsOutOfImageBoundaries)
            );
        }
        for (width, height) in [(2., 1.), (1., 2.)] {
            assert_eq!(
                view.set_crop_box(CropBox {
                    left: 0.,
                    top: 0.,
                    width,
                    height,
                }),
                Err(CropBoxError::SizeIsOutOfImageBoundaries)
            );
        }
        for (width, height) in [(0., 1.), (1., 0.), (-1., 1.), (1., -1.)] {
            assert_eq!(
                view.set_crop_box(CropBox {
                    left: 0.,
                    top: 0.,
                    width,
                    height,
                }),
                Err(CropBoxError::WidthOrHeightLessOrEqualToZero)
            );
        }
    }

    #[test]
    fn downscale_u8x4() {
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [2942547, 2947799, 2885025, 6122718],
            );

            P::downscale_test(
                ResizeAlg::SuperSampling(FilterType::Lanczos3, 2),
                cpu_extensions,
                [2942426, 2947750, 2884861, 6123019],
            );
        }
    }

    #[test]
    fn upscale_u8x4() {
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
        #[cfg(target_arch = "wasm32")]
        {
            cpu_extensions_vec.push(CpuExtensions::Simd128);
        }
        for cpu_extensions in cpu_extensions_vec {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [1155201879, 1152689565, 1123329272, 2364890085],
            );
        }
    }

    #[test]
    fn custom_filter_u8x4() {
        std::env::set_var("DONT_SAVE_RESULT", "1");
        const LANCZOS3_RESULT: [u64; 4] = [2942547, 2947799, 2885025, 6122718];
        const LANCZOS4_RESULT: [u64; 4] = [2943083, 2948315, 2885436, 6122629];

        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Lanczos3),
            CpuExtensions::None,
            LANCZOS3_RESULT,
        );

        fn sinc_filter(mut x: f64) -> f64 {
            if x == 0.0 {
                1.0
            } else {
                x *= PI;
                x.sin() / x
            }
        }

        fn lanczos3_filter(x: f64) -> f64 {
            if (-3.0..3.0).contains(&x) {
                sinc_filter(x) * sinc_filter(x / 3.)
            } else {
                0.0
            }
        }

        for bad_support in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(Filter::new("bad_support", lanczos3_filter, bad_support).is_err());
        }

        let my_lanczos3 = Filter::new("MyLanczos3", lanczos3_filter, 3.0).unwrap();
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Custom(my_lanczos3)),
            CpuExtensions::None,
            LANCZOS3_RESULT,
        );

        fn lanczos4_filter(x: f64) -> f64 {
            if (-4.0..4.0).contains(&x) {
                sinc_filter(x) * sinc_filter(x / 4.)
            } else {
                0.0
            }
        }

        let my_lanczos4 = Filter::new("MyLanczos4", lanczos4_filter, 4.0).unwrap();
        P::downscale_test(
            ResizeAlg::Convolution(FilterType::Custom(my_lanczos4)),
            CpuExtensions::None,
            LANCZOS4_RESULT,
        );
    }

    #[test]
    fn cropping() {
        let img = ImageReader::open("./data/crop_test.png")
            .unwrap()
            .decode()
            .unwrap();
        let width = nonzero(img.width());
        let height = nonzero(img.height());
        let src_image =
            Image::from_vec_u8(width, height, img.to_rgba8().into_raw(), PixelType::U8x4).unwrap();

        let mut src_view = src_image.view();
        src_view
            .set_crop_box(CropBox {
                left: 521.,
                top: 1414.,
                width: 1485.,
                height: 1486.,
            })
            .unwrap();

        let mut dst_image = Image::new(nonzero(1279), nonzero(1280), PixelType::U8x4);

        let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));

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
        let mut results = vec![];
        for cpu_extensions in cpu_extensions_vec {
            unsafe {
                resizer.set_cpu_extensions(cpu_extensions);
            }
            let mut dst_view = dst_image.view_mut();
            resizer.resize(&src_view, &mut dst_view).unwrap();
            let cpu_ext_str = cpu_ext_into_str(cpu_extensions);
            save_result(&dst_image, &format!("crop_test-{}.png", cpu_ext_str));
            results.push((image_checksum::<U8x4, 4>(&dst_image), cpu_ext_str));
        }

        for (checksum, cpu_ext_str) in results {
            assert_eq!(
                checksum,
                [0, 236287962, 170693682, 417465600],
                "checksum of result image was resized with cpu_extensions={} is incorrect",
                cpu_ext_str
            );
        }
    }
}
