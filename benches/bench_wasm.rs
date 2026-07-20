#![cfg(target_arch = "wasm32")]

use fast_image_resize::pixels::{
    F32x2, F32x3, F32x4, U16x2, U16x3, U16x4, U8x2, U8x3, U8x4, F32, U16, U8,
};
use pastey::paste;
use resize::Pixel::{Gray16, Gray8, GrayF32, RGB16, RGB8, RGBA16P, RGBA8P, RGBF32};
use rgb::FromSlice;
use wasm_bindgen_test::{wasm_bindgen_bench, wasm_bindgen_test_configure, Criterion};

use crate::wasm_utils::load_images::{load_big_image, load_big_src_image};
use crate::wasm_utils::resize_functions::*;
mod wasm_utils;

// Run in a browser.
wasm_bindgen_test_configure!(run_in_browser);

fn get_bencher() -> Criterion {
    Criterion::default()
        .with_location(file!(), module_path!())
        .sample_size(10)
}

macro_rules! image_benches {
    {$(($pixel_type:ty, $suffix:ident)),+} => {
        $(
            paste! {
                #[wasm_bindgen_bench]
                async fn [<bench_image_ $suffix>](_: &mut Criterion) {
                    let src_image = load_big_image::<$pixel_type>().await.unwrap();
                    image_resize(&mut get_bencher(), &src_image);
                }
            }
        )+
    }
}

image_benches! {
    (U8, l8),
    (U16, l16),
    (F32, l32f),
    (U8x3, rgb8),
    (U16x3, rgb16),
    (F32x3, rgb32f)
}

macro_rules! resize_benches {
    {$(($pixel_type:ty, $resize_pixel_type:expr, $resize_as_method:ident, $suffix:ident)),+} => {
        $(
            paste! {
                #[wasm_bindgen_bench]
                async fn [<bench_resize_ $suffix>](_: &mut Criterion) {
                    let src_image = load_big_image::<$pixel_type>().await.unwrap();
                    resize_resize(
                        &mut get_bencher(),
                        $resize_pixel_type,
                        src_image.as_raw().$resize_as_method(),
                        src_image.width(),
                        src_image.height(),
                    );
                }
            }
        )+
    }
}

resize_benches! {
    (U8, Gray8, as_gray, l),
    (U16, Gray16, as_gray, l16),
    (F32, GrayF32, as_gray, l32f),
    (U8x3, RGB8, as_rgb, rgb),
    (U16x3, RGB16, as_rgb, rgb16),
    (F32x3, RGBF32, as_rgb, rgb32f),
    (U8x4, RGBA8P, as_rgba, rgba),
    (U16x4, RGBA16P, as_rgba, rgba16)
}

macro_rules! fir_benches {
    {$(($pixel_type:ty, $use_alpha:expr, $suffix:ident)),+} => {
        $(
            paste! {
                #[wasm_bindgen_bench]
                async fn [<bench_fir_ $suffix>](_: &mut Criterion) {
                    let src_image = load_big_src_image::<$pixel_type>().await.unwrap();
                    fir_resize::<$pixel_type>(&mut get_bencher(), src_image, $use_alpha);
                }
            }
        )+
    }
}

fir_benches! {
    (U8, false, l),
    (U16, false, l16),
    (F32, false, l32f),
    (U8x2, true, la),
    (U16x2, true, la16),
    (F32x2, true, la32f),
    (U8x3, false, rgb),
    (U16x3, false, rgb16),
    (F32x3, false, rgb32f),
    (U8x4, true, rgba),
    (U16x4, true, rgba16),
    (F32x4, true, rgba32f)
}
