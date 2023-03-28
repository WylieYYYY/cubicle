mod interop;

use js_sys::JsString;
use wasm_bindgen::prelude::*;
use web_sys::console;
use crate::interop::tabs;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    let tab_creator = Closure::new(tabs::create_tab);
    interop::addRuntimeListener("onInstalled", &tab_creator);
    tab_creator.forget();
    console::log_1(&JsString::from("Hello world"));
    Ok(())
}
