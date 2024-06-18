use fast_image_resize::pixels::F32x4;

mod utils;

pub fn bench_downscale_rgba32f(bench_group: &mut utils::BenchGroup) {
    type P = F32x4;
    utils::libvips_resize::<P>(bench_group, true);
    utils::fir_resize::<P>(bench_group, true);
}

fn main() {
    let res = utils::run_bench(bench_downscale_rgba32f, "Compare resize of RGBA32F image");
    utils::print_and_write_compare_result(&res);
}
