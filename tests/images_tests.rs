use fast_image_resize as fr;
use fast_image_resize::images::{
    CroppedImage, CroppedImageMut, Image, ImageRef, TypedCroppedImage, TypedCroppedImageMut,
    TypedImage, TypedImageRef,
};
use fast_image_resize::pixels::{U8x4, U8};
use fast_image_resize::{ImageView, IntoImageView, PixelType, ResizeOptions};

#[test]
fn create_image_ref_from_small_buffer() {
    let width = 64;
    let height = 32;
    let buffer = vec![0; 64 * 30];

    let res = ImageRef::new(width, height, &buffer, PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_from_small_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 64 * 30];

    let res = Image::from_slice_u8(width, height, &mut buffer, PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = Image::from_vec_u8(width, height, buffer, PixelType::U8);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_image_from_big_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 65 * 32];

    let res = Image::from_slice_u8(width, height, &mut buffer, PixelType::U8);
    assert!(res.is_ok());

    let res = Image::from_vec_u8(width, height, buffer, PixelType::U8);
    assert!(res.is_ok());
}

#[test]
fn create_type_image_ref_from_small_buffer() {
    let width = 64;
    let height = 32;
    let buffer = vec![U8::new(0); 64 * 30];

    let res = TypedImageRef::<U8>::new(width, height, &buffer);
    assert!(matches!(res, Err(fr::InvalidPixelsSize)));
}

#[test]
fn create_typed_image_from_small_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 64 * 30];

    let res = TypedImage::<U8>::from_buffer(width, height, &mut buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);

    let res = TypedImageRef::<U8>::from_buffer(width, height, &buffer);
    assert_eq!(res.unwrap_err(), fr::ImageBufferError::InvalidBufferSize);
}

#[test]
fn create_typed_image_from_big_buffer() {
    let width = 64;
    let height = 32;
    let mut buffer = vec![0; 65 * 32];

    let res = TypedImage::<U8>::from_buffer(width, height, &mut buffer);
    assert!(res.is_ok());

    let res = TypedImageRef::<U8>::from_buffer(width, height, &buffer);
    assert!(res.is_ok());
}

#[test]
fn typed_cropped_image() {
    const BLACK: U8x4 = U8x4::new([0; 4]);
    const WHITE: U8x4 = U8x4::new([255; 4]);

    let mut source_pixels = Vec::with_capacity(64 * 64);
    source_pixels.extend((0..64 * 64).map(|i| {
        let y = i / 64;
        if (10..54).contains(&y) {
            let x = i % 64;
            if (10..54).contains(&x) {
                return WHITE;
            }
        }
        BLACK
    }));

    // Black source image with white square inside
    let src_image = TypedImage::<U8x4>::from_pixels(64, 64, source_pixels).unwrap();
    // Black destination image
    let mut dst_image = TypedImage::<U8x4>::new(40, 40);

    let cropped_src_image = TypedCroppedImage::from_ref(&src_image, 10, 10, 44, 44).unwrap();
    assert_eq!(cropped_src_image.width(), 44);
    assert_eq!(cropped_src_image.height(), 44);

    let mut resizer = fr::Resizer::new();
    resizer
        .resize_typed(
            &cropped_src_image,
            &mut dst_image,
            &ResizeOptions::new().resize_alg(fr::ResizeAlg::Nearest),
        )
        .unwrap();

    let white_block = vec![WHITE; 40 * 40];
    assert_eq!(dst_image.pixels(), white_block);
}

#[test]
fn cropped_image() {
    const BLACK: U8x4 = U8x4::new([0; 4]);
    const WHITE: U8x4 = U8x4::new([255; 4]);

    let mut source_pixels = Vec::with_capacity(64 * 64);
    source_pixels.extend((0..64 * 64).map(|i| {
        let y = i / 64;
        if (10..54).contains(&y) {
            let x = i % 64;
            if (10..54).contains(&x) {
                return WHITE;
            }
        }
        BLACK
    }));

    // Black source image with white square inside
    let src_image = ImageRef::from_pixels(64, 64, &source_pixels).unwrap();
    // Black destination image
    let mut dst_image = Image::new(40, 40, PixelType::U8x4);

    let cropped_src_image = CroppedImage::new(&src_image, 10, 10, 44, 44).unwrap();
    assert_eq!(cropped_src_image.width(), 44);
    assert_eq!(cropped_src_image.height(), 44);

    let mut resizer = fr::Resizer::new();
    resizer
        .resize(
            &cropped_src_image,
            &mut dst_image,
            &ResizeOptions::new().resize_alg(fr::ResizeAlg::Nearest),
        )
        .unwrap();

    let dst_typed_image = dst_image.typed_image().unwrap();
    let dst_pixels: &[U8x4] = dst_typed_image.pixels();
    let white_block = vec![WHITE; 40 * 40];
    assert_eq!(dst_pixels, white_block);
}

#[test]
fn typed_cropped_image_mut() {
    const BLACK: U8x4 = U8x4::new([0; 4]);
    const WHITE: U8x4 = U8x4::new([255; 4]);

    // White source image
    let src_image = TypedImage::from_pixels(64, 32, vec![WHITE; 64 * 32]).unwrap();
    // Black destination image
    let mut dst_image = TypedImage::<U8x4>::new(64, 32);

    let mut cropped_dst_image =
        TypedCroppedImageMut::from_ref(&mut dst_image, 10, 10, 44, 12).unwrap();
    assert_eq!(cropped_dst_image.width(), 44);
    assert_eq!(cropped_dst_image.height(), 12);

    let mut resizer = fr::Resizer::new();
    resizer
        .resize_typed(
            &src_image,
            &mut cropped_dst_image,
            &ResizeOptions::new().resize_alg(fr::ResizeAlg::Nearest),
        )
        .unwrap();

    let dst_pixels = dst_image.pixels();

    let row_size: usize = 64;
    let black_block = vec![BLACK; 10 * row_size];
    // Top border
    assert_eq!(dst_pixels[0..10 * row_size], black_block);

    // Middle rows
    let mut middle_row = vec![BLACK; 10];
    middle_row.extend(vec![WHITE; 44]);
    middle_row.extend(vec![BLACK; 10]);
    for row in dst_pixels.chunks_exact(row_size).skip(10).take(12) {
        assert_eq!(row, middle_row);
    }

    // Bottom border
    assert_eq!(dst_pixels[22 * row_size..], black_block);
}

#[test]
fn cropped_image_mut() {
    const BLACK: U8x4 = U8x4::new([0; 4]);
    const WHITE: U8x4 = U8x4::new([255; 4]);

    // White source image
    let src_pixels = vec![WHITE; 64 * 32];
    let src_image = ImageRef::from_pixels(64, 32, &src_pixels).unwrap();
    // Black destination image
    let mut dst_image = Image::new(64, 32, PixelType::U8x4);

    let mut cropped_dst_image = CroppedImageMut::new(&mut dst_image, 10, 10, 44, 12).unwrap();
    assert_eq!(cropped_dst_image.width(), 44);
    assert_eq!(cropped_dst_image.height(), 12);

    let mut resizer = fr::Resizer::new();
    resizer
        .resize(
            &src_image,
            &mut cropped_dst_image,
            &ResizeOptions::new().resize_alg(fr::ResizeAlg::Nearest),
        )
        .unwrap();

    let dst_typed_image = dst_image.typed_image().unwrap();
    let dst_pixels: &[U8x4] = dst_typed_image.pixels();

    let row_size: usize = 64;
    let black_block = vec![BLACK; 10 * row_size];
    // Top border
    assert_eq!(dst_pixels[0..10 * row_size], black_block);

    // Middle rows
    let mut middle_row = vec![BLACK; 10];
    middle_row.extend(vec![WHITE; 44]);
    middle_row.extend(vec![BLACK; 10]);
    for row in dst_pixels.chunks_exact(row_size).skip(10).take(12) {
        assert_eq!(row, middle_row);
    }

    // Bottom border
    assert_eq!(dst_pixels[22 * row_size..], black_block);
}
