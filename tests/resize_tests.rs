use std::cmp::Ordering;
use std::num::NonZeroU32;

use fast_image_resize::pixels::*;
use fast_image_resize::{
    CpuExtensions, CropBox, DifferentTypesOfPixelsError, FilterType, Image, ImageView, PixelType,
    ResizeAlg, Resizer,
};
use testing::PixelExt;

fn get_new_height(src_image: &ImageView, new_width: u32) -> u32 {
    let scale = new_width as f32 / src_image.width().get() as f32;
    (src_image.height().get() as f32 * scale).round() as u32
}

const NEW_WIDTH: u32 = 255;
const NEW_BIG_WIDTH: u32 = 5016;

#[test]
fn try_resize_to_other_pixel_type() {
    let src_image = U8x4::load_big_src_image();
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    let mut dst_image = Image::new(
        NonZeroU32::new(1024).unwrap(),
        NonZeroU32::new(256).unwrap(),
        PixelType::U8,
    );
    assert!(matches!(
        resizer.resize(&src_image.view(), &mut dst_image.view_mut()),
        Err(DifferentTypesOfPixelsError)
    ));
}

#[test]
fn resize_to_same_size() {
    let width = NonZeroU32::new(100).unwrap();
    let height = NonZeroU32::new(80).unwrap();
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
    let width = NonZeroU32::new(100).unwrap();
    let height = NonZeroU32::new(80).unwrap();
    let src_width = NonZeroU32::new(120).unwrap();
    let src_height = NonZeroU32::new(100).unwrap();
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

fn downscale_test<P: PixelExt>(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions) -> Vec<u8> {
    let image = P::load_big_src_image();
    assert_eq!(image.pixel_type(), P::pixel_type());

    let mut resizer = Resizer::new(resize_alg);
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    let new_height = get_new_height(&image.view(), NEW_WIDTH);
    let mut result = Image::new(
        NonZeroU32::new(NEW_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
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
        "downscale-{}-{}-{}",
        P::pixel_type_str(),
        alg_name,
        testing::cpu_ext_into_str(cpu_extensions),
    );
    testing::save_result(&result, &name);
    result.buffer().to_owned()
}

fn upscale_test<P: PixelExt>(resize_alg: ResizeAlg, cpu_extensions: CpuExtensions) -> Vec<u8> {
    let image = P::load_small_src_image();
    assert_eq!(image.pixel_type(), P::pixel_type());

    let mut resizer = Resizer::new(resize_alg);
    unsafe {
        resizer.set_cpu_extensions(cpu_extensions);
    }
    let new_height = get_new_height(&image.view(), NEW_BIG_WIDTH);
    let mut result = Image::new(
        NonZeroU32::new(NEW_BIG_WIDTH).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
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
        P::pixel_type_str(),
        alg_name,
        testing::cpu_ext_into_str(cpu_extensions),
    );
    testing::save_result(&result, &name);
    result.buffer().to_owned()
}

#[test]
fn downscale_u8() {
    type P = U8;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(testing::image_checksum::<1>(&buffer), [2920348]);

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(testing::image_checksum::<1>(&buffer), [2923557]);
    }
}

#[test]
fn upscale_u8() {
    type P = U8;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(testing::image_checksum::<1>(&buffer), [1148754010]);

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(testing::image_checksum::<1>(&buffer), [1148811406]);
    }
}

#[test]
fn downscale_u8x2() {
    type P = U8x2;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(testing::image_checksum::<2>(&buffer), [2920348, 6121802]);

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(testing::image_checksum::<2>(&buffer), [2923557, 6122818]);
    }
}

#[test]
fn upscale_u8x2() {
    type P = U8x2;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_checksum::<2>(&buffer),
        [1146218632, 2364895380]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_checksum::<2>(&buffer),
            [1146283728, 2364890194]
        );
    }
}

#[test]
fn downscale_u8x3() {
    type P = U8x3;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_checksum::<3>(&buffer),
        [2937940, 2945380, 2882679]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_checksum::<3>(&buffer),
            [2942479, 2947850, 2885072]
        );
    }
}

#[test]
fn upscale_u8x3() {
    type P = U8x3;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_checksum::<3>(&buffer),
        [1156008260, 1158417906, 1135087540]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_checksum::<3>(&buffer),
            [1156107005, 1158443335, 1135101759]
        );
    }
}

#[test]
fn downscale_u8x4() {
    type P = U8x4;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_checksum::<4>(&buffer),
        [2937940, 2945380, 2882679, 6121802]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_checksum::<4>(&buffer),
            [2942479, 2947850, 2885072, 6122818]
        );

        downscale_test::<P>(
            ResizeAlg::SuperSampling(FilterType::Lanczos3, 2),
            cpu_extensions,
        );
    }
}

#[test]
fn upscale_u8x4() {
    type P = U8x4;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_checksum::<4>(&buffer),
        [1155096957, 1152644783, 1123285879, 2364895380]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_checksum::<4>(&buffer),
            [1155201788, 1152688479, 1123328716, 2364890194]
        );
    }
}

#[test]
fn downscale_u16() {
    type P = U16;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(testing::image_u16_checksum::<1>(&buffer), [750529436]);

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<1>(&buffer),
            [751401243],
            "Error in checksum for {:?}",
            cpu_extensions
        );
    }
}

#[test]
fn upscale_u16() {
    type P = U16;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(testing::image_u16_checksum::<1>(&buffer), [295229780570]);

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<1>(&buffer),
            [295246940755],
            "Error in checksum for {:?}",
            cpu_extensions
        );
    }
}

#[test]
fn downscale_u16x2() {
    type P = U16x2;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_u16_checksum::<2>(&buffer),
        [750529436, 1573303114]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<2>(&buffer),
            [751401243, 1573563971]
        );
    }
}

#[test]
fn upscale_u16x2() {
    type P = U16x2;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_u16_checksum::<2>(&buffer),
        [294578188424, 607778112660]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<2>(&buffer),
            [294597368766, 607776760273]
        );
    }
}

#[test]
fn downscale_u16x3() {
    type P = U16x3;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_u16_checksum::<3>(&buffer),
        [755050580, 756962660, 740848503]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<3>(&buffer),
            [756269847, 757632467, 741478612]
        );
    }
}

#[test]
fn upscale_u16x3() {
    type P = U16x3;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_u16_checksum::<3>(&buffer),
        [297094122820, 297713401842, 291717497780]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<3>(&buffer),
            [297122154090, 297723994984, 291725294637]
        );
    }
}

#[test]
fn downscale_u16x4() {
    type P = U16x4;
    let buffer = downscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_u16_checksum::<4>(&buffer),
        [755050580, 756962660, 740848503, 1573303114]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            downscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<4>(&buffer),
            [756269847, 757632467, 741478612, 1573563971]
        );
    }
}

#[test]
fn resizer_u16x4() {
    use resize::px::RGBA;
    use resize::Pixel::RGBA16P;
    use rgb::FromSlice;

    let src_image = U16x4::load_big_image().to_rgba16();
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let resize_src_image = src_image.as_raw().as_rgba();
    let mut dst =
        vec![RGBA::new(0u16, 0u16, 0u16, 0u16); (new_width.get() * new_height.get()) as usize];
    let filter = resize::Type::Triangle;
    let mut resize = resize::new(
        src_image.width() as usize,
        src_image.height() as usize,
        new_width.get() as usize,
        new_height.get() as usize,
        RGBA16P,
        filter,
    )
    .unwrap();
    resize.resize(resize_src_image, &mut dst).unwrap();
}

#[test]
fn upscale_u16x4() {
    type P = U16x4;
    let buffer = upscale_test::<P>(ResizeAlg::Nearest, CpuExtensions::None);
    assert_eq!(
        testing::image_u16_checksum::<4>(&buffer),
        [296859917949, 296229709231, 288684470903, 607778112660]
    );

    let mut cpu_extensions_vec = vec![CpuExtensions::None];
    #[cfg(target_arch = "x86_64")]
    {
        cpu_extensions_vec.push(CpuExtensions::Sse4_1);
        cpu_extensions_vec.push(CpuExtensions::Avx2);
    }
    for cpu_extensions in cpu_extensions_vec {
        let buffer =
            upscale_test::<P>(ResizeAlg::Convolution(FilterType::Lanczos3), cpu_extensions);
        assert_eq!(
            testing::image_u16_checksum::<4>(&buffer),
            [296888688348, 296243667797, 288698172180, 607776760273],
        );
    }
}
