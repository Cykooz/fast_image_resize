use fast_image_resize::pixels::F32x2;

mod utils;

pub fn bench_downscale_la_f32(bench_group: &mut utils::BenchGroup) {
    type P = F32x2;
    utils::libvips_resize::<P>(bench_group, true);
    utils::fir_resize::<P>(bench_group, true);
}

fn main() {
    let res = utils::run_bench(bench_downscale_la_f32, "Compare resize of LA-F32 image");
    utils::print_and_write_compare_result(&res);
}
