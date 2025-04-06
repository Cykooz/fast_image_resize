use fast_image_resize::pixels::U8x4;
use resize::Pixel::RGBA8P;
use rgb::FromSlice;
use utils::testing::PixelTestingExt;

mod utils;

pub fn bench_downscale_rgba(bench_group: &mut utils::BenchGroup) {
    type P = U8x4;
    let src_image = P::load_big_image();
    utils::resize_resize(
        bench_group,
        RGBA8P,
        src_image.as_raw().as_rgba(),
        src_image.width(),
        src_image.height(),
    );
    utils::libvips_resize::<P>(bench_group, true);
    utils::fir_resize::<P>(bench_group, true);
}

fn main() {
    let res = utils::run_bench(bench_downscale_rgba, "Compare resize of RGBA image");
    utils::print_and_write_compare_result(&res);
}
