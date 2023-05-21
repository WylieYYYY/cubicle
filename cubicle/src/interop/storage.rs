use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::util::errors::CustomError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="set")]
    fn storage_set(keys: JsValue) -> Promise;
}

pub async fn set_with_keys(keys: JsValue) -> Result<(), CustomError> {
    JsFuture::from(storage_set(keys)).await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("store to")
        }))?;
    Ok(())
}
