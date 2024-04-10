use fast_image_resize::pixels::U8x2;

mod utils;

pub fn bench_downscale_la(bench_group: &mut utils::BenchGroup) {
    type P = U8x2;
    utils::libvips_resize::<P>(bench_group, true);
    utils::fir_resize::<P>(bench_group, true);
}

fn main() {
    let res = utils::run_bench(bench_downscale_la, "Compare resize of LA image");
    utils::print_and_write_compare_result(&res);
}
