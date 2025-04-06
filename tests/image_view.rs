use fast_image_resize::images::{TypedCroppedImageMut, TypedImage};
use fast_image_resize::pixels::U8;
use fast_image_resize::{ImageView, ImageViewMut};
use testing::non_zero_u32;

mod testing;

mod split_by_width {
    use super::*;
    use fast_image_resize::images::{TypedCroppedImage, TypedImageRef};

    fn split<T: ImageView>(img: &T) {
        for num_parts in 1..16 {
            let res = img
                .split_by_width(0, non_zero_u32(512), non_zero_u32(num_parts))
                .unwrap();
            assert_eq!(res.len() as u32, num_parts);
            let sum_width = res.iter().map(|v| v.width()).sum::<u32>();
            assert_eq!(sum_width, 512);
        }
    }

    fn split_mut<T: ImageViewMut>(img: &mut T) {
        for num_parts in 1..16 {
            let res = img
                .split_by_width_mut(0, non_zero_u32(512), non_zero_u32(num_parts))
                .unwrap();
            assert_eq!(res.len() as u32, num_parts);
            let sum_width = res.iter().map(|v| v.width()).sum::<u32>();
            assert_eq!(sum_width, 512);
        }
    }

    #[test]
    fn typed_image_ref() {
        let width = 512;
        let height = 384;
        let buffer = vec![U8::new(0); (width * height) as usize];
        let img = TypedImageRef::<U8>::new(width, height, &buffer).unwrap();
        split(&img);
    }

    #[test]
    fn typed_image() {
        let mut img = TypedImage::<U8>::new(512, 384);
        split(&img);
        split_mut(&mut img);
    }

    #[test]
    fn typed_cropped_image() {
        let img = TypedImage::<U8>::new(512 + 20, 384 + 20);
        let cropped_img = TypedCroppedImage::from_ref(&img, 10, 10, 512, 384).unwrap();
        split(&cropped_img);
    }

    #[test]
    fn typed_cropped_image_mut() {
        let mut img = TypedImage::<U8>::new(512 + 20, 384 + 20);
        let mut cropped_img = TypedCroppedImageMut::from_ref(&mut img, 10, 10, 512, 384).unwrap();
        split(&cropped_img);
        split_mut(&mut cropped_img);
    }
}

mod split_by_height {
    use super::*;
    use fast_image_resize::images::{TypedCroppedImage, TypedImageRef};

    fn split<T: ImageView>(img: &T) {
        for num_parts in 1..16 {
            let res = img
                .split_by_height(0, non_zero_u32(512), non_zero_u32(num_parts))
                .unwrap();
            assert_eq!(res.len() as u32, num_parts);
            let sum_height = res.iter().map(|v| v.height()).sum::<u32>();
            assert_eq!(sum_height, 512);
        }
    }

    fn split_mut<T: ImageViewMut>(img: &mut T) {
        for num_parts in 1..16 {
            let res = img
                .split_by_height_mut(0, non_zero_u32(512), non_zero_u32(num_parts))
                .unwrap();
            assert_eq!(res.len() as u32, num_parts);
            let sum_height = res.iter().map(|v| v.height()).sum::<u32>();
            assert_eq!(sum_height, 512);
        }
    }

    #[test]
    fn typed_image_ref() {
        let width = 384;
        let height = 512;
        let buffer = vec![U8::new(0); (width * height) as usize];
        let img = TypedImageRef::<U8>::new(width, height, &buffer).unwrap();
        split(&img);
    }

    #[test]
    fn typed_image() {
        let mut img: TypedImage<U8> = TypedImage::new(384, 512);
        split(&img);
        split_mut(&mut img);
    }

    #[test]
    fn typed_cropped_image() {
        let img = TypedImage::<U8>::new(384 + 20, 512 + 20);
        let cropped_img = TypedCroppedImage::from_ref(&img, 10, 10, 384, 512).unwrap();
        split(&cropped_img);
    }

    #[test]
    fn typed_cropped_image_mut() {
        let mut img: TypedImage<U8> = TypedImage::new(384 + 20, 512 + 20);
        let mut cropped_img = TypedCroppedImageMut::from_ref(&mut img, 10, 10, 384, 512).unwrap();
        split(&cropped_img);
        split_mut(&mut cropped_img);
    }
}
