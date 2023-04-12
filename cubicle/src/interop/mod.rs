mod bits;
pub mod contextual_identities;
pub mod fetch;
pub mod tabs;

use js_sys::Promise;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(raw_module = "./background.js")]
extern "C" {
    #[wasm_bindgen(js_name="addRuntimeListener")]
    pub fn add_runtime_listener(event: &str,
        handler: &Closure<dyn Fn(Box<[JsValue]>) -> Promise>);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "runtime"], js_name="getURL")]
    fn prepend_extension_base_url(path: &str) -> String;
}

pub async fn fetch_extension_file(path: &str) -> String {
    JsFuture::from(fetch::get(&prepend_extension_base_url(path)).await
        .expect("valid and stable connection").text()
        .expect("standard does not define synchronous errors")).await
        .expect("assume consume body successful").as_string()
        .expect("body must be a valid string")
}

const MAP_SERIALIZER: &Serializer = &Serializer::new()
    .serialize_maps_as_objects(true);
