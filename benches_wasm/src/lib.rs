#![cfg(target_arch = "wasm32")]

use fast_image_resize::pixels::U16x3;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::load_images::{load_big_image, load_big_src_image};

pub mod load_images;
pub mod resize_functions;
mod utils;

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    Ok(())
}

#[wasm_bindgen]
pub async fn run_benches() -> Result<(), JsValue> {
    utils::set_panic_hook();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let code = document.create_element("pre")?;
    body.append_child(&code)?;

    code.set_inner_html("Hello from Rust!\n");

    bench_downscale_rgb16(&code).await?;

    Ok(())
}

async fn bench_downscale_rgb16(code: &Element) -> Result<(), JsValue> {
    use resize::Pixel::RGB16;
    use rgb::FromSlice;
    type P = U16x3;
    print(code, "Bench downscale RGB16\n").await;
    print(code, "'image' crate:\n").await;
    let src_image = load_big_image::<P>().await?;
    for res in resize_functions::image_resize(&src_image) {
        print_results(code, res).await;
    }
    // let res = resize_functions::resize_resize(
    //     RGB16,
    //     src_image.as_raw().as_rgb(),
    //     src_image.width(),
    //     src_image.height(),
    // );
    // print_results(code, res);
    // let src_image_data = load_big_src_image::<P>().await?;
    // let res = resize_functions::fir_resize::<P>(src_image_data, false);
    // print_results(code, res);
    Ok(())
}

async fn print_results(code: &Element, results: (String, String, f64)) {
    let (lib_name, alg_name, speed) = results;
    let line = &format!("{} {}: {:.2}ms\n", lib_name, alg_name, speed);
    print(code, line).await;
}

async fn print(code: &Element, line: &str) {
    code.append_with_str_1(line).unwrap();
    // Let the browser process events and repaint.
    TimeoutFuture::new(0).await;
}
