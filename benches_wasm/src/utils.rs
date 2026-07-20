use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}

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

/// Returns a high-resolution timestamp in milliseconds.
pub fn now() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}
