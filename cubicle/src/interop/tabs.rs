use std::collections::HashMap;

use js_sys::Promise;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use super::MAP_SERIALIZER;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="create")]
    fn tab_create(create_properties: JsValue) -> Promise;
}

pub fn create_tab(_: Box<[JsValue]>) -> Promise {
    let mut args = HashMap::new();
    args.insert("url", "https://wylieyyyy.gitlab.io");
    match args.serialize(MAP_SERIALIZER) {
        Ok(serialized) => tab_create(serialized),
        Err(err) => Promise::reject(&JsValue::from(err))
    }
}
