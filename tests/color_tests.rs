use fast_image_resize as fr;
use fast_image_resize::pixels::*;
use testing::nonzero;

mod gamma_tests {
    use super::*;

    #[test]
    fn gamma22_into_linear_test() {
        let mapper = fr::create_gamma_22_mapper();
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8).unwrap();
        let src_checksum = testing::image_checksum::<U8, 1>(&src_image);
        assert_eq!(src_checksum, [32640]);

        // into U8
        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8);
        mapper
            .forward_map(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();
        let dst_checksum = testing::image_checksum::<U8, 1>(&dst_image);
        assert_eq!(dst_checksum, [20443]);

        // into U16
        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U16);
        mapper
            .forward_map(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();
        let dst_checksum = testing::image_checksum::<U16, 1>(&dst_image);
        assert_eq!(dst_checksum, [5255141]);
    }

    #[test]
    fn gamma22_into_linear_errors_test() {
        let mapper = fr::create_gamma_22_mapper();
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), PixelType::U8);
        let result = mapper.forward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x2);
        let result = mapper.forward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }

    #[test]
    fn linear_into_gamma22_test() {
        let mapper = fr::create_gamma_22_mapper();
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8).unwrap();
        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8);

        mapper
            .backward_map(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let src_checksum = testing::image_checksum::<U8, 1>(&src_image);
        assert_eq!(src_checksum, [32640]);
        let dst_checksum = testing::image_checksum::<U8, 1>(&dst_image);
        assert_eq!(dst_checksum, [44824]);
    }

    #[test]
    fn linear_into_gamma22_errors_test() {
        let mapper = fr::create_gamma_22_mapper();
        let buffer: Vec<u8> = (0u8..=255).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), PixelType::U8);
        let result = mapper.backward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x2);
        let result = mapper.backward_map(&src_image.view(), &mut dst_image.view_mut());
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
        let mapper = fr::create_srgb_mapper();
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8x3).unwrap();
        let src_checksum = testing::image_checksum::<U8x3, 3>(&src_image);
        assert_eq!(src_checksum, [32640, 32640, 32640]);

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x3);
        mapper
            .forward_map(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let dst_checksum = testing::image_checksum::<U8x3, 3>(&dst_image);
        assert_eq!(dst_checksum, [20304, 20304, 20304]);
    }

    #[test]
    fn srgba_into_rgba_test() {
        let mapper = fr::create_srgb_mapper();
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v, 255]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8x4).unwrap();
        let src_checksum = testing::image_checksum::<U8x4, 4>(&src_image);
        assert_eq!(src_checksum, [32640, 32640, 32640, 65280]);

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x4);
        mapper
            .forward_map(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let dst_checksum = testing::image_checksum::<U8x4, 4>(&dst_image);
        assert_eq!(dst_checksum, [20304, 20304, 20304, 65280]);
    }

    #[test]
    fn srgb_into_rgb_errors_test() {
        let mapper = fr::create_srgb_mapper();
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8x3).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), PixelType::U8x3);
        let result = mapper.forward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x2);
        let result = mapper.forward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }

    #[test]
    fn rgb_into_srgb_test() {
        let mapper = fr::create_srgb_mapper();
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8x3).unwrap();
        let src_checksum = testing::image_checksum::<U8x3, 3>(&src_image);
        assert_eq!(src_checksum, [32640, 32640, 32640]);

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x3);
        mapper
            .backward_map(&src_image.view(), &mut dst_image.view_mut())
            .unwrap();

        let dst_checksum = testing::image_checksum::<U8x3, 3>(&dst_image);
        assert_eq!(dst_checksum, [44981, 44981, 44981]);
    }

    #[test]
    fn rgb_into_srgb_errors_test() {
        let mapper = fr::create_srgb_mapper();
        let buffer: Vec<u8> = (0u8..=255).flat_map(|v| [v, v, v]).collect();
        let src_image =
            fr::Image::from_vec_u8(nonzero(16), nonzero(16), buffer, PixelType::U8x3).unwrap();

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(1), PixelType::U8x3);
        let result = mapper.backward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(result, Err(fr::MappingError::DifferentDimensions)));

        let mut dst_image = fr::Image::new(nonzero(16), nonzero(16), PixelType::U8x2);
        let result = mapper.backward_map(&src_image.view(), &mut dst_image.view_mut());
        assert!(matches!(
            result,
            Err(fr::MappingError::UnsupportedCombinationOfImageTypes)
        ));
    }
}
