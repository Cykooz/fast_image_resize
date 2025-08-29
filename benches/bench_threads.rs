use fast_image_resize::images::Image;
use fast_image_resize::pixels::U8x3;
use fast_image_resize::{CpuExtensions, FilterType, ResizeAlg, ResizeOptions, Resizer};
use utils::testing::PixelTestingExt;

use crate::utils::{bench, build_md_table, BenchGroup};

mod utils;

pub fn fir_resize<P: PixelTestingExt>(bench_group: &mut BenchGroup, use_alpha: bool) {
    let src_sizes: Vec<u32> = vec![2, 10, 100, 500, 1000, 5000, 10000, 65536];

    let mut resizer = Resizer::new();
    unsafe {
        resizer.set_cpu_extensions(CpuExtensions::None);
    }
    let resize_options = ResizeOptions::new()
        .resize_alg(ResizeAlg::Convolution(FilterType::Bilinear))
        .use_alpha(false);

    for &src_width in &src_sizes {
        for &src_height in &src_sizes {
            let dst_width = src_width / 2;
            let src_image = Image::new(src_width, src_height, P::pixel_type());
            let mut dst_image = Image::new(dst_width, src_height, src_image.pixel_type());

            for thread_count in 1..=8 {
                bench(
                    bench_group,
                    10,
                    format!("{src_width}x{src_height}"),
                    format!("{thread_count}"),
                    |bencher| {
                        let mut builder = rayon::ThreadPoolBuilder::new();
                        builder = builder.num_threads(thread_count);
                        let pool = builder.build().unwrap();
                        pool.install(|| {
                            bencher.iter(|| {
                                resizer
                                    .resize(&src_image, &mut dst_image, &resize_options)
                                    .unwrap()
                            })
                        })
                    },
                );
            }
        }
    }
}

pub fn bench_threads(bench_group: &mut BenchGroup) {
    type P = U8x3;
    fir_resize::<P>(bench_group, false);
}

fn main() {
    let res = utils::run_bench(bench_threads, "Compare resize by width images with threads");
    let md_table = build_md_table(&res);
    println!("{}", md_table);
}
