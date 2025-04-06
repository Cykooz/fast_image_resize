use fast_image_resize::create_srgb_mapper;
use fast_image_resize::images::Image;
use fast_image_resize::pixels::U8x3;
use utils::pin_process_to_cpu0;
use utils::testing::PixelTestingExt;

mod utils;

pub fn bench_color_mapper(bench_group: &mut utils::BenchGroup) {
    let src_image = U8x3::load_big_src_image();
    let mut dst_image = Image::new(
        src_image.width(),
        src_image.height(),
        src_image.pixel_type(),
    );
    let mapper = create_srgb_mapper();
    bench_group
        .criterion_group
        .bench_function("SRGB U8x3 => RGB U8x3", |bencher| {
            bencher.iter(|| {
                mapper.forward_map(&src_image, &mut dst_image).unwrap();
            })
        });
}

fn main() {
    pin_process_to_cpu0();
    utils::run_bench(bench_color_mapper, "Color mapper");
}
