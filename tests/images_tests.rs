use fast_image_resize as fr;
use fast_image_resize::images::{CroppedImageMut, Image, TypedImage, TypedImageMut};
use fast_image_resize::pixels::{U8x4, U8};
use fast_image_resize::ImageView;

#[test]
fn create_image_from_small_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 64 * 30];

    let res = Image::from_slice_u8(width, height, &mut buffer, fr::PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = Image::from_vec_u8(width, height, buffer, fr::PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_typed_image_from_small_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 64 * 30];

    let res = TypedImageMut::<U8>::from_buffer(width, height, &mut buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = TypedImage::<U8>::from_buffer(width, height, &buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_from_big_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 65 * 32];

    let res = Image::from_slice_u8(width, height, &mut buffer, fr::PixelType::U8);
    assert!(res.is_ok());

    let res = Image::from_vec_u8(width, height, buffer, fr::PixelType::U8);
    assert!(res.is_ok());
}

#[test]
fn create_typed_image_from_big_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 65 * 32];

    let res = TypedImageMut::<U8>::from_buffer(width, height, &mut buffer);
    assert!(res.is_ok());

    let res = TypedImage::<U8>::from_buffer(width, height, &buffer);
    assert!(res.is_ok());
}

#[test]
fn crop_view_mut() {
    // White source image
    let src_image =
        Image::from_vec_u8(64, 32, vec![255; 64 * 32 * 4], fr::PixelType::U8x4).unwrap();
    let src_image = src_image.typed_image::<U8x4>().unwrap();
    // Black destination image
    let mut dst_image = Image::new(64, 32, fr::PixelType::U8x4);

    let mut cropped_dst_image =
        CroppedImageMut::new(dst_image.typed_image_mut::<U8x4>().unwrap(), 10, 10, 44, 12).unwrap();
    assert_eq!(cropped_dst_image.width(), 44);
    assert_eq!(cropped_dst_image.height(), 12);

    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Nearest);
    resizer
        .resize_typed(&src_image, &mut cropped_dst_image, None)
        .unwrap();

    let row_size: usize = 64 * 4;
    let dst_buffer = dst_image.buffer();

    let black_block = vec![0u8; 10 * row_size];
    // Top border
    assert_eq!(dst_buffer[0..10 * row_size], black_block);

    // Middle rows
    let mut middle_row = vec![0u8; 10 * 4];
    middle_row.extend(vec![255u8; 44 * 4]);
    middle_row.extend(vec![0u8; 10 * 4]);
    for row in dst_buffer.chunks_exact(row_size).skip(10 * 4).take(12 * 4) {
        assert_eq!(row, middle_row);
    }

    // Bottom border
    assert_eq!(dst_buffer[22 * row_size..], black_block);
}
