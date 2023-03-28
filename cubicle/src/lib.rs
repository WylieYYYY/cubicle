use js_sys::JsString;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    console::log_1(&JsString::from("Hello world"));
    Ok(())
}
