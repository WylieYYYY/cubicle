mod bits;
pub mod contextual_identities;
pub mod fetch;
pub mod tabs;

use std::any;

use js_sys::{JsString, Promise, Reflect};
use serde::Deserialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::util::errors::CustomError;

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

pub fn get_or_standard_mismatch(target: &JsValue, key: &str)
-> Result<JsValue, CustomError> {
    Reflect::get(target, &JsString::from(key))
        .or(Err(CustomError::StandardMismatch {
        message: format!("key `{}` is missing", key)
    }))
}

pub fn cast_or_standard_mismatch<T>(target: JsValue) -> Result<T, CustomError>
where T: for <'de> Deserialize<'de> {
    Ok(serde_wasm_bindgen::from_value(target)
        .or(Err(CustomError::StandardMismatch {
        message: format!("`{}` expected", any::type_name::<T>())
    }))?)
}

const MAP_SERIALIZER: &Serializer = &Serializer::new()
    .serialize_maps_as_objects(true);
