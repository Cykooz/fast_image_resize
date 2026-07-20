#![cfg(target_arch = "wasm32")]

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response};
pub mod load_images;
pub mod resize_functions;

pub async fn load_file(path: &str) -> Result<Vec<u8>, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(path, &opts)?;

    let window = web_sys::window().unwrap();
    let resp = window.fetch_with_request(&request).await?;

    let resp: Response = resp.dyn_into()?;

    let buffer = resp.array_buffer()?.await?;
    let bytes = Uint8Array::new(&buffer).to_vec();

    Ok(bytes)
}
