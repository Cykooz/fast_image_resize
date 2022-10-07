use glassbench::*;
use std::ops::Deref;

use fast_image_resize::color::mappers::SRGB_TO_RGB;
use fast_image_resize::pixels::U8x3;
use fast_image_resize::Image;
use testing::PixelTestingExt;

mod utils;

pub fn bench_color_mapper(bench: &mut Bench) {
    let src_image = U8x3::load_big_src_image();
    let mut dst_image = Image::new(
        src_image.width(),
        src_image.height(),
        src_image.pixel_type(),
    );
    let src_view = src_image.view();
    let mut dst_view = dst_image.view_mut();
    let mapper = SRGB_TO_RGB.deref();
    bench.task("SRGB U8x3 => RGB U8x3", |task| {
        task.iter(|| {
            mapper.forward_map(&src_view, &mut dst_view).unwrap();
        })
    });
}

bench_main!("Bench color mappers", bench_color_mapper,);
