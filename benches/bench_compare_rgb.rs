use fast_image_resize::pixels::U8x3;
use resize::Pixel::RGB8;
use rgb::FromSlice;
use utils::testing::PixelTestingExt;

mod utils;

pub fn bench_downscale_rgb(bench_group: &mut utils::BenchGroup) {
    type P = U8x3;
    let src_image = P::load_big_image();
    utils::image_resize(bench_group, &src_image);
    utils::resize_resize(
        bench_group,
        RGB8,
        src_image.as_raw().as_rgb(),
        src_image.width(),
        src_image.height(),
    );
    utils::libvips_resize::<P>(bench_group, false);
    utils::fir_resize::<P>(bench_group, false);
}

fn main() {
    let res = utils::run_bench(bench_downscale_rgb, "Compare resize of RGB image");
    utils::print_and_write_compare_result(&res);
}
