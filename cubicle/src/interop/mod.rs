mod bits;
pub mod contextual_identities;
pub mod fetch;
pub mod tabs;

use js_sys::Promise;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "./main.js")]
extern "C" {
    pub fn addRuntimeListener(event: &str,
        handler: &Closure<dyn Fn(Box<[JsValue]>) -> Promise>);
}

const MAP_SERIALIZER: &Serializer = &Serializer::new()
    .serialize_maps_as_objects(true);
