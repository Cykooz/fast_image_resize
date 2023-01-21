use fast_image_resize::pixels::U8x3;
use fast_image_resize::{create_srgb_mapper, Image};
use testing::PixelTestingExt;

mod utils;

pub fn bench_color_mapper(bench_group: &mut utils::BenchGroup) {
    let src_image = U8x3::load_big_src_image();
    let mut dst_image = Image::new(
        src_image.width(),
        src_image.height(),
        src_image.pixel_type(),
    );
    let src_view = src_image.view();
    let mut dst_view = dst_image.view_mut();
    let mapper = create_srgb_mapper();
    bench_group.bench_function("SRGB U8x3 => RGB U8x3", |bencher| {
        bencher.iter(|| {
            mapper.forward_map(&src_view, &mut dst_view).unwrap();
        })
    });
}

fn main() {
    utils::run_bench(bench_color_mapper, "Color mapper");
}
