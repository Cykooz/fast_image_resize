use fast_image_resize as fr;
use std::num::NonZeroU32;

fn nonzero(v: u32) -> NonZeroU32 {
    NonZeroU32::new(v).unwrap()
}

mod gamma_tests {
    use super::*;

    #[test]
    fn gamma22_into_linear_test() {
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8).unwrap();
        let src_checksum = testing::image_checksum::<1>(src_image.buffer());
        assert_eq!(src_checksum, [32640]);

        // into U8
        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8);
        fr::color::gamma::gamma22_into_linear(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();
        let dst_checksum = testing::image_checksum::<1>(dst_image.buffer());
        assert_eq!(dst_checksum, [20443]);

        // into U16
        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U16);
        fr::color::gamma::gamma22_into_linear(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();
        let dst_checksum = testing::image_u16_checksum::<1>(dst_image.buffer());
        assert_eq!(dst_checksum, [5255141]);
    }

    #[test]
    fn gamma22_into_linear_errors_test() {
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), fr::PixelType::U8);
        let result =
            fr::color::gamma::gamma22_into_linear(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x2);
        let result =
            fr::color::gamma::gamma22_into_linear(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }

    #[test]
    fn linear_into_gamma22_test() {
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8).unwrap();
        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8);

        fr::color::gamma::linear_into_gamma22(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let src_checksum = testing::image_checksum::<1>(src_image.buffer());
        assert_eq!(src_checksum, [32640]);
        let dst_checksum = testing::image_checksum::<1>(dst_image.buffer());
        assert_eq!(dst_checksum, [44824]);
    }

    #[test]
    fn linear_into_gamma22_errors_test() {
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), fr::PixelType::U8);
        let result =
            fr::color::gamma::linear_into_gamma22(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x2);
        let result =
            fr::color::gamma::linear_into_gamma22(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }
}

mod srgb_tests {
    use super::*;

    #[test]
    fn srgb_into_rgb_test() {
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8x3).unwrap();
        let src_checksum = testing::image_checksum::<3>(src_image.buffer());
        assert_eq!(src_checksum, [32640, 32640, 32640]);

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x3);
        fr::color::srgb::srgb_into_rgb(&src_image.view(), &mut dst_image.view_mut()).unwrap();

        let dst_checksum = testing::image_checksum::<3>(dst_image.buffer());
        assert_eq!(dst_checksum, [20304, 20304, 20304]);
    }

    #[test]
    fn srgba_into_rgba_test() {
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v, 255]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8x4).unwrap();
        let src_checksum = testing::image_checksum::<4>(src_image.buffer());
        assert_eq!(src_checksum, [32640, 32640, 32640, 65280]);

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x4);
        fr::color::srgb::srgb_into_rgb(&src_image.view(), &mut dst_image.view_mut()).unwrap();

        let dst_checksum = testing::image_checksum::<4>(dst_image.buffer());
        assert_eq!(dst_checksum, [20304, 20304, 20304, 65280]);
    }

    #[test]
    fn srgb_into_rgb_errors_test() {
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8x3).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), fr::PixelType::U8x3);
        let result = fr::color::srgb::srgb_into_rgb(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x2);
        let result = fr::color::srgb::srgb_into_rgb(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }

    #[test]
    fn rgb_into_srgb_test() {
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8x3).unwrap();
        let src_checksum = testing::image_checksum::<3>(src_image.buffer());
        assert_eq!(src_checksum, [32640, 32640, 32640]);

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x3);
        fr::color::srgb::rgb_into_srgb(&src_image.view(), &mut dst_image.view_mut()).unwrap();

        let dst_checksum = testing::image_checksum::<3>(dst_image.buffer());
        assert_eq!(dst_checksum, [44981, 44981, 44981]);
    }

    #[test]
    fn rgb_into_srgb_errors_test() {
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, fr::PixelType::U8x3).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), fr::PixelType::U8x3);
        let result = fr::color::srgb::rgb_into_srgb(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), fr::PixelType::U8x2);
        let result = fr::color::srgb::rgb_into_srgb(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }
}
