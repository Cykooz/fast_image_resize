use fast_image_resize::pixels::U16x2;

mod utils;

pub fn bench_downscale_la16(bench_group: &mut utils::BenchGroup) {
    type P = U16x2;
    utils::libvips_resize::<P>(bench_group, true);
    utils::fir_resize::<P>(bench_group, true);
}

fn main() {
    let res = utils::run_bench(bench_downscale_la16, "Compare resize of LA16 image");
    utils::print_and_write_compare_result(&res);
}
