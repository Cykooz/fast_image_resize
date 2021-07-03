use std::num::NonZeroU32;

use fast_image_resize::{
    CropBox, FilterType, ImageData, PixelType, ResizeAlg, Resizer, SrcImageView,
};

fn resize_lanczos3(src_pixels: &[u8], width: NonZeroU32, height: NonZeroU32) -> Vec<u8> {
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    let src_image = ImageData::new(width, height, src_pixels, PixelType::U8x4).unwrap();
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_image.pixel_type());

    let src_view = src_image.src_view();
    let mut dst_view = dst_image.dst_view();
    resizer.resize(&src_view, &mut dst_view);

    dst_image.get_buffer().to_owned()
}

fn crop_and_resize_image(mut src_view: SrcImageView) -> ImageData<Vec<u8>> {
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    src_view
        .set_crop_box(CropBox {
            left: 10,
            top: 10,
            width: NonZeroU32::new(100).unwrap(),
            height: NonZeroU32::new(200).unwrap(),
        })
        .unwrap();
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_view.pixel_type());

    let mut dst_view = dst_image.dst_view();
    resizer.resize(&src_view, &mut dst_view);

    dst_image
}
