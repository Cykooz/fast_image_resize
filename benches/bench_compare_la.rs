use fast_image_resize::pixels::U8x2;

mod utils;

pub fn bench_downscale_la(bench_group: &mut utils::BenchGroup) {
    utils::fir_resize_with_alpha::<U8x2>(bench_group);
}

fn main() {
    let res = utils::run_bench(bench_downscale_la, "Compare resize of LA image");
    utils::print_and_write_compare_result(&res);
}
