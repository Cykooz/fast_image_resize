use std::cmp::Ordering;
use std::fmt::Debug;

use fast_image_resize::images::Image;
use fast_image_resize::pixels::*;
use fast_image_resize::{
    testing as fr_testing, CpuExtensions, CropBoxError, Filter, FilterType, IntoImageView,
    PixelTrait, PixelType, ResizeAlg, ResizeError, ResizeOptions, Resizer,
};
use testing::{cpu_ext_into_str, image_checksum, save_result, PixelTestingExt};

mod testing;

fn get_new_height(src_image: &impl IntoImageView, new_width: u32) -> u32 {
    let scale = new_width as f32 / src_image.width() as f32;
    (src_image.height() as f32 * scale).round() as u32
}

const NEW_WIDTH: u32 = 255;
const NEW_BIG_WIDTH: u32 = 5016;

#[test]
fn try_resize_to_other_pixel_type() {
    let mut resizer = Resizer::new();
    let src_image = U8x4::load_big_src_image();
    let mut dst_image = Image::new(1024, 256, PixelType::U8);
    assert!(matches!(
        resizer.resize(
            &src_image,
            &mut dst_image,
            &ResizeOptions::new().resize_alg(ResizeAlg::Nearest),
        ),
        Err(ResizeError::PixelTypesAreDifferent)
    ));
}

#[test]
fn resize_to_same_size() {
    let width = 100;
    let height = 80;
    let buffer: Vec<u8> = (0..8000)
        .map(|v| (v & 0xff) as u8)
        .flat_map(|v| [v; 4])
        .collect();
    let src_image = Image::from_vec_u8(width, height, buffer, PixelType::U8x4).unwrap();
    let mut dst_image = Image::new(width, height, PixelType::U8x4);
    Resizer::new()
        .resize(&src_image, &mut dst_image, None)
        .unwrap();
    assert!(matches!(
        src_image.buffer().cmp(dst_image.buffer()),
        Ordering::Equal
    ));
}

#[test]
fn resize_to_same_size_after_cropping() {
    let width = 100;
    let height = 80;
    let src_width = 120;
    let src_height = 100;
    let buffer: Vec<u8> = (0..12000)
        .map(|v| (v & 0xff) as u8)
        .flat_map(|v| [v; 4])
        .collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, PixelType::U8x4).unwrap();
    let mut dst_image = Image::new(width, height, PixelType::U8x4);

    let options = ResizeOptions::new().crop(10., 10., width as _, height as _);
    Resizer::new()
        .resize(&src_image, &mut dst_image, &options)
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
/// if the width of destination image is equal to the width of cropped source image.
fn resize_to_same_width<const C: usize>(
    pixel_type: PixelType,
    cpu_extensions: CpuExtensions,
    create_pixel: fn(v: u8) -> [u8; C],
) {
    fr_testing::clear_log();
    let width = 100;
    let height = 80;
    let src_width = 120;
    let src_height = 100;
    // Image columns are made up of pixels of the same color.
    let buffer: Vec<u8> = (0..12000)
        .flat_map(|v| create_pixel((v % 120) as u8))
        .collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, pixel_type).unwrap();
    let mut dst_image = Image::new(width, height, pixel_type);

    let mut resizer = Resizer::new();
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    resizer
        .resize(
            &src_image,
            &mut dst_image,
            &ResizeOptions::new()
                .crop(10., 0., width as _, src_height as _)
                .use_alpha(false),
        )
        .unwrap();

    assert!(fr_testing::logs_contain(
        "compute vertical convolution coefficients"
    ));
    assert!(!fr_testing::logs_contain(
        "compute horizontal convolution coefficients"
    ));

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
}

/// In this test, we check that resizer won't use vertical convolution
/// if the height of destination image is equal to the height of cropped source image.
fn resize_to_same_height<const C: usize>(
    pixel_type: PixelType,
    cpu_extensions: CpuExtensions,
    create_pixel: fn(v: u8) -> [u8; C],
) {
    fr_testing::clear_log();
    let width = 100;
    let height = 80;
    let src_width = 120;
    let src_height = 100;
    // Image rows are made up of pixels of the same color.
    let buffer: Vec<u8> = (0..12000)
        .flat_map(|v| create_pixel((v / 120) as u8))
        .collect();
    let src_image = Image::from_vec_u8(src_width, src_height, buffer, pixel_type).unwrap();
    let mut dst_image = Image::new(width, height, pixel_type);

    let mut resizer = Resizer::new();
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    resizer
        .resize(
            &src_image,
            &mut dst_image,
            &ResizeOptions::new()
                .crop(0., 10., src_width as _, height as _)
                .use_alpha(false),
        )
        .unwrap();

    assert!(!fr_testing::logs_contain(
        "compute vertical convolution coefficients"
    ));
    assert!(fr_testing::logs_contain(
        "compute horizontal convolution coefficients"
    ));

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
}

#[test]
fn resize_to_same_width_or_height_after_cropping() {
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
    Self: PixelTestingExt + PixelTrait,
    T: Sized + Copy + Clone + Debug + PartialEq + Default + 'static,
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

        let mut resizer = Resizer::new();
        unsafe {
            resizer.set_cpu_extensions(cpu_extensions);
        }
        let new_height = get_new_height(&image, NEW_WIDTH);
        let mut result = Image::new(NEW_WIDTH, new_height, image.pixel_type());
        resizer
            .resize(
                &image,
                &mut result,
                &ResizeOptions::new().resize_alg(resize_alg).use_alpha(false),
            )
            .unwrap();

        let alg_name = match resize_alg {
            ResizeAlg::Nearest => "nearest",
            ResizeAlg::Convolution(filter) => match filter {
                FilterType::Box => "box",
                FilterType::Bilinear => "bilinear",
                FilterType::Hamming => "hamming",
                FilterType::Mitchell => "mitchell",
                FilterType::CatmullRom => "catmullrom",
                FilterType::Gaussian => "gaussian",
                FilterType::Lanczos3 => "lanczos3",
                _ => "unknown",
            },
            ResizeAlg::Interpolation(filter) => match filter {
                FilterType::Box => "inter_box",
                FilterType::Bilinear => "inter_bilinear",
                FilterType::Hamming => "inter_hamming",
                FilterType::Mitchell => "inter_mitchell",
                FilterType::CatmullRom => "inter_catmullrom",
                FilterType::Gaussian => "inter_gaussian",
                FilterType::Lanczos3 => "inter_lanczos3",
                _ => "inter_unknown",
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
        save_result(&result, &name);
        assert_eq!(
            image_checksum::<Self, CC>(&result),
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

        let mut resizer = Resizer::new();
        unsafe {
            resizer.set_cpu_extensions(cpu_extensions);
        }
        let new_height = get_new_height(&image, NEW_BIG_WIDTH);
        let mut result = Image::new(NEW_BIG_WIDTH, new_height, image.pixel_type());
        resizer
            .resize(
                &image,
                &mut result,
                &ResizeOptions::new().resize_alg(resize_alg).use_alpha(false),
            )
            .unwrap();

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
        save_result(&result, &name);
        assert_eq!(
            image_checksum::<Self, CC>(&result),
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [1156107445, 1158443938, 1135102297],
            );
        }
    }

    #[test]
    fn resize_u8x3_interpolation() {
        type P = U8x3;
        P::downscale_test(
            ResizeAlg::Interpolation(FilterType::Bilinear),
            CpuExtensions::None,
            [2938733, 2946338, 2883813],
        );

        P::upscale_test(
            ResizeAlg::Interpolation(FilterType::Bilinear),
            CpuExtensions::None,
            [1156013474, 1158419787, 1135090328],
        );
    }

    #[test]
    fn downscale_u16() {
        type P = U16;
        P::downscale_test(ResizeAlg::Nearest, CpuExtensions::None, [750529436]);

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
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

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [296888688348, 296243667797, 288698172180, 607776760273],
            );
        }
    }

    // I32
    #[test]
    fn downscale_i32() {
        type P = I32;
        P::downscale_test(ResizeAlg::Nearest, CpuExtensions::None, [24593724281554]);

        for cpu_extensions in P::cpu_extensions() {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [36889044005199],
            );
        }
    }

    #[test]
    fn upscale_i32() {
        type P = I32;
        P::upscale_test(ResizeAlg::Nearest, CpuExtensions::None, [9674237252903955]);

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [11090415545881916],
            );
        }
    }

    // F32
    #[test]
    fn downscale_f32() {
        type P = F32;
        P::downscale_test(ResizeAlg::Nearest, CpuExtensions::None, [28891951209032]);

        for cpu_extensions in P::cpu_extensions() {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [41687319249443],
            );
        }
    }

    #[test]
    fn upscale_f32() {
        type P = F32;
        P::upscale_test(ResizeAlg::Nearest, CpuExtensions::None, [11165019414549868]);

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [12506894762090128],
            );
        }
    }

    // F32x2
    #[test]
    fn downscale_f32x2() {
        type P = F32x2;
        P::downscale_test(
            ResizeAlg::Nearest,
            CpuExtensions::None,
            [28891951209032, 26023210300788],
        );

        for cpu_extensions in P::cpu_extensions() {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [41687319249443, 29873206892121],
            );
        }
    }

    #[test]
    fn upscale_f32x2() {
        type P = F32x2;
        P::upscale_test(
            ResizeAlg::Nearest,
            CpuExtensions::None,
            [9941292360529429, 10060767588318486],
        );

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [10426687457795354, 10465695788378423],
            );
        }
    }

    // F32x3
    #[test]
    fn downscale_f32x3() {
        type P = F32x3;
        P::downscale_test(
            ResizeAlg::Nearest,
            CpuExtensions::None,
            [28271357515050, 34102344731602, 34154875278897],
        );

        for cpu_extensions in P::cpu_extensions() {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [41676905227663, 40314876714373, 40146438250679],
            );
        }
    }

    #[test]
    fn upscale_f32x3() {
        type P = F32x3;
        P::upscale_test(
            ResizeAlg::Nearest,
            CpuExtensions::None,
            [10945976142696393, 13185868359050104, 13307431189096686],
        );

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [12058434766196853, 14081191473477964, 14079890133382920],
            );
        }
    }

    // F32x4
    #[test]
    fn downscale_f32x4() {
        type P = F32x4;
        P::downscale_test(
            ResizeAlg::Nearest,
            CpuExtensions::None,
            [
                28271357515050,
                34102344731602,
                34154875278897,
                26023210300788,
            ],
        );

        for cpu_extensions in P::cpu_extensions() {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [
                    41676905227663,
                    40314876714373,
                    40146438250679,
                    29873206892121,
                ],
            );
        }
    }

    #[test]
    fn upscale_f32x4() {
        type P = F32x4;
        P::upscale_test(
            ResizeAlg::Nearest,
            CpuExtensions::None,
            [
                9939217321658853,
                9944847383085392,
                9940281434784023,
                10060767588318486,
            ],
        );

        for cpu_extensions in P::cpu_extensions() {
            P::upscale_test(
                ResizeAlg::Convolution(FilterType::Lanczos3),
                cpu_extensions,
                [
                    10431959951229359,
                    10423928285570087,
                    10420567450069105,
                    10465695788378423,
                ],
            );
        }
    }

    #[test]
    fn fractional_cropping() {
        let mut src_buf = [0, 0, 0, 0, 255, 0, 0, 0, 0];
        let src_image = Image::from_slice_u8(3, 3, &mut src_buf, PixelType::U8).unwrap();
        let mut dst_image = Image::new(1, 1, PixelType::U8);
        let mut resizer = Resizer::new();
        let options = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Box));
        // Resize without cropping
        resizer
            .resize(&src_image, &mut dst_image, &options)
            .unwrap();
        assert_eq!(dst_image.buffer()[0], (255.0f32 / 9.0).round() as u8);

        // Resize with fractional cropping
        resizer
            .resize(&src_image, &mut dst_image, &options.crop(0.5, 0.5, 2., 2.))
            .unwrap();
        assert_eq!(dst_image.buffer()[0], (255.0f32 / 4.0).round() as u8);

        // Resize with integer cropping
        resizer
            .resize(&src_image, &mut dst_image, &options.crop(1., 1., 1., 1.))
            .unwrap();
        assert_eq!(dst_image.buffer()[0], 255);
    }
}

mod u8x4 {
    use std::f64::consts::PI;

    use fast_image_resize::ResizeError;
    use image::ImageReader;

    use super::*;

    type P = U8x4;

    #[test]
    fn invalid_crop_box() {
        let mut resizer = Resizer::new();
        let src_image = Image::new(1, 1, P::pixel_type());
        let mut dst_image = Image::new(2, 2, P::pixel_type());

        let mut options = ResizeOptions::new().resize_alg(ResizeAlg::Nearest);

        for (left, top) in [(1., 0.), (0., 1.)] {
            options = options.crop(left, top, 1., 1.);
            assert_eq!(
                resizer.resize(&src_image, &mut dst_image, &options),
                Err(ResizeError::SrcCroppingError(
                    CropBoxError::PositionIsOutOfImageBoundaries
                ))
            );
        }
        for (width, height) in [(2., 1.), (1., 2.)] {
            options = options.crop(0., 0., width, height);
            assert_eq!(
                resizer.resize(&src_image, &mut dst_image, &options),
                Err(ResizeError::SrcCroppingError(
                    CropBoxError::SizeIsOutOfImageBoundaries
                ))
            );
        }
        for (width, height) in [(-1., 1.), (1., -1.)] {
            options = options.crop(0., 0., width, height);
            assert_eq!(
                resizer.resize(&src_image, &mut dst_image, &options),
                Err(ResizeError::SrcCroppingError(
                    CropBoxError::WidthOrHeightLessThanZero
                ))
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

        for cpu_extensions in P::cpu_extensions() {
            P::downscale_test(
                ResizeAlg::Convolution(FilterType::Gaussian),
                cpu_extensions,
                [2939881, 2946811, 2884299, 6122867],
            );

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

        for cpu_extensions in P::cpu_extensions() {
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
        let width = img.width();
        let height = img.height();
        let src_image =
            Image::from_vec_u8(width, height, img.to_rgba8().into_raw(), PixelType::U8x4).unwrap();

        let options = ResizeOptions::new().crop(521., 1414., 1485., 1486.);

        let mut dst_image = Image::new(1279, 1280, PixelType::U8x4);

        let mut resizer = Resizer::new();

        let mut results = vec![];
        for cpu_extensions in U8x4::cpu_extensions() {
            unsafe {
                resizer.set_cpu_extensions(cpu_extensions);
            }
            resizer
                .resize(&src_image, &mut dst_image, &options)
                .unwrap();
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

#[cfg(feature = "rayon")]
#[test]
fn split_image_on_different_number_of_parts() {
    let src_image = Image::new(2176, 4608, PixelType::U8x4);
    let mut dst_image = Image::new(582, 552, src_image.pixel_type());
    let options = ResizeOptions::new()
        .use_alpha(false)
        .resize_alg(ResizeAlg::Convolution(FilterType::Box))
        .crop(740.0, 1645.2, 58.200000000000045, 55.299999999999955);

    for num in 2..32 {
        let mut builder = rayon::ThreadPoolBuilder::new();
        builder = builder.num_threads(num);
        let pool = builder.build().unwrap();

        pool.install(|| {
            let mut resizer = Resizer::new();
            resizer
                .resize(&src_image, &mut dst_image, &options)
                .expect("resize failed with {num} threads");
        });
    }
}
