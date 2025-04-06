use fast_image_resize::pixels::U8;
use resize::Pixel::Gray8;
use rgb::FromSlice;
use utils::testing::PixelTestingExt;

mod utils;

pub fn bench_compare_l(bench_group: &mut utils::BenchGroup) {
    type P = U8;
    let src_image = P::load_big_image();
    utils::image_resize(bench_group, &src_image);
    utils::resize_resize(
        bench_group,
        Gray8,
        src_image.as_raw().as_gray(),
        src_image.width(),
        src_image.height(),
    );
    utils::libvips_resize::<P>(bench_group, false);
    utils::fir_resize::<P>(bench_group, false);
}

fn main() {
    let res = utils::run_bench(bench_compare_l, "Compare resize of U8 image");
    utils::print_and_write_compare_result(&res);
}
