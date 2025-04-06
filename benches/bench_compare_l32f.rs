use fast_image_resize::pixels::F32;
use resize::Pixel::GrayF32;
use rgb::FromSlice;
use utils::testing::PixelTestingExt;

mod utils;

pub fn bench_downscale_l32f(bench_group: &mut utils::BenchGroup) {
    type P = F32;
    let src_image = P::load_big_image();
    utils::image_resize(bench_group, &src_image);
    utils::resize_resize(
        bench_group,
        GrayF32,
        src_image.as_raw().as_gray(),
        src_image.width(),
        src_image.height(),
    );
    utils::libvips_resize::<P>(bench_group, false);
    utils::fir_resize::<P>(bench_group, false);
}

fn main() {
    let res = utils::run_bench(bench_downscale_l32f, "Compare resize of L32F image");
    utils::print_and_write_compare_result(&res);
}
