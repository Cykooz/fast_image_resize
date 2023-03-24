use fast_image_resize as fr;
use fast_image_resize::pixels::U8;
use testing::nonzero;

#[test]
fn create_image_from_small_buffer() {
    let width = nonzero(64);
    let height = nonzero(32);
    let mut buffer = vec![0; 64 * 30];

    let res = fr::Image::from_slice_u8(width, height, &mut buffer, fr::PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = fr::Image::from_vec_u8(width, height, buffer, fr::PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_view_from_small_buffer() {
    let width = nonzero(64);
    let height = nonzero(32);
    let mut buffer = vec![0; 64 * 30];

    let res = fr::ImageViewMut::<U8>::from_buffer(width, height, &mut buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = fr::ImageView::<U8>::from_buffer(width, height, &buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_from_big_buffer() {
    let width = nonzero(64);
    let height = nonzero(32);
    let mut buffer = vec![0; 65 * 32];

    let res = fr::Image::from_slice_u8(width, height, &mut buffer, fr::PixelType::U8);
    assert!(res.is_ok());

    let res = fr::Image::from_vec_u8(width, height, buffer, fr::PixelType::U8);
    assert!(res.is_ok());
}

#[test]
fn create_image_view_from_big_buffer() {
    let width = nonzero(64);
    let height = nonzero(32);
    let mut buffer = vec![0; 65 * 32];

    let res = fr::ImageViewMut::<U8>::from_buffer(width, height, &mut buffer);
    assert!(res.is_ok());

    let res = fr::ImageView::<U8>::from_buffer(width, height, &buffer);
    assert!(res.is_ok());
}

#[test]
fn crop_view_mut() {
    // White source image
    let src_image = fr::Image::from_vec_u8(
        nonzero(64),
        nonzero(32),
        vec![255; 64 * 32],
        fr::PixelType::U8,
    )
    .unwrap();
    // Black destination image
    let mut dst_image = fr::Image::new(nonzero(64), nonzero(32), fr::PixelType::U8);

    let mut cropped_dst_view = dst_image
        .view_mut()
        .crop(fr::CropBox {
            left: 10,
            top: 10,
            width: nonzero(44),
            height: nonzero(12),
        })
        .unwrap();
    assert_eq!(cropped_dst_view.width().get(), 44);
    assert_eq!(cropped_dst_view.height().get(), 12);

    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Nearest);
    resizer
        .resize(&src_image.view(), &mut cropped_dst_view)
        .unwrap();

    let row_size: usize = 64;
    let dst_buffer = dst_image.buffer();

    let black_block = vec![0u8; 10 * row_size];
    // Top border
    assert_eq!(dst_buffer[0..10 * row_size], black_block);

    // Middle rows
    let mut middle_row = vec![0u8; 10];
    middle_row.extend(vec![255u8; 44]);
    middle_row.extend(vec![0u8; 10]);
    for row in dst_buffer.chunks_exact(row_size).skip(10).take(12) {
        assert_eq!(row, middle_row);
    }

    // Bottom border
    assert_eq!(dst_buffer[22 * row_size..], black_block);
}
