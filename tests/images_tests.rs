use fast_image_resize as fr;
use fast_image_resize::pixels::U8;
use std::num::NonZeroU32;

#[test]
fn create_image_from_small_buffer() {
    let width = NonZeroU32::new(64).unwrap();
    let height = NonZeroU32::new(32).unwrap();
    let mut buffer = vec![0; 64 * 30];

    let res = fr::Image::from_slice_u8(width, height, &mut buffer, fr::PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = fr::Image::from_vec_u8(width, height, buffer, fr::PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_view_from_small_buffer() {
    let width = NonZeroU32::new(64).unwrap();
    let height = NonZeroU32::new(32).unwrap();
    let mut buffer = vec![0; 64 * 30];

    let res = fr::ImageViewMut::<U8>::from_buffer(width, height, &mut buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = fr::ImageView::<U8>::from_buffer(width, height, &buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_from_big_buffer() {
    let width = NonZeroU32::new(64).unwrap();
    let height = NonZeroU32::new(32).unwrap();
    let mut buffer = vec![0; 65 * 32];

    let res = fr::Image::from_slice_u8(width, height, &mut buffer, fr::PixelType::U8);
    assert!(res.is_ok());

    let res = fr::Image::from_vec_u8(width, height, buffer, fr::PixelType::U8);
    assert!(res.is_ok());
}

#[test]
fn create_image_view_from_big_buffer() {
    let width = NonZeroU32::new(64).unwrap();
    let height = NonZeroU32::new(32).unwrap();
    let mut buffer = vec![0; 65 * 32];

    let res = fr::ImageViewMut::<U8>::from_buffer(width, height, &mut buffer);
    assert!(res.is_ok());

    let res = fr::ImageView::<U8>::from_buffer(width, height, &buffer);
    assert!(res.is_ok());
}
