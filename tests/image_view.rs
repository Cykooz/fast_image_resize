use fast_image_resize::images::TypedImage;
use fast_image_resize::pixels::U8;
use fast_image_resize::{ImageView, ImageViewMut};
use testing::non_zero_u32;

#[test]
fn split_by_width() {
    let mut img: TypedImage<U8> = TypedImage::new(512, 384);
    for num_parts in 1..16 {
        let res = img
            .split_by_width(0, non_zero_u32(512), non_zero_u32(num_parts))
            .unwrap();
        assert_eq!(res.len() as u32, num_parts);
        let sum_width = res.iter().map(|v| v.width()).sum::<u32>();
        assert_eq!(sum_width, 512);
        drop(res);

        let res = img
            .split_by_width_mut(0, non_zero_u32(512), non_zero_u32(num_parts))
            .unwrap();
        assert_eq!(res.len() as u32, num_parts);
        let sum_width = res.iter().map(|v| v.width()).sum::<u32>();
        assert_eq!(sum_width, 512);
    }
}

#[test]
fn split_by_height() {
    let mut img: TypedImage<U8> = TypedImage::new(384, 512);
    for num_parts in 1..16 {
        let res = img
            .split_by_height(0, non_zero_u32(512), non_zero_u32(num_parts))
            .unwrap();
        assert_eq!(res.len() as u32, num_parts);
        let sum_height = res.iter().map(|v| v.height()).sum::<u32>();
        assert_eq!(sum_height, 512);
        drop(res);

        let res = img
            .split_by_height_mut(0, non_zero_u32(512), non_zero_u32(num_parts))
            .unwrap();
        assert_eq!(res.len() as u32, num_parts);
        let sum_height = res.iter().map(|v| v.height()).sum::<u32>();
        assert_eq!(sum_height, 512);
    }
}
