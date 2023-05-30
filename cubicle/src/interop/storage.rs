use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::interop::{self, MAP_SERIALIZER};
use crate::util::errors::CustomError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="get")]
    fn storage_get(keys: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="set")]
    fn storage_set(keys: &JsValue) -> Promise;
}

pub async fn get_with_keys<T>(keys: &mut T) -> Result<(), CustomError>
where T: for <'de> Deserialize<'de> + Serialize {
    let passed_keys = keys.serialize(MAP_SERIALIZER)
        .expect("serialization fail unlikely");
    let got = JsFuture::from(storage_get(&passed_keys)).await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("load from")
        }))?;
    *keys = interop::cast_or_standard_mismatch(got)?;
    Ok(())
}

pub async fn set_with_serde_keys<T>(keys: &T) -> Result<(), CustomError>
where T: Serialize {
    let keys = keys.serialize(MAP_SERIALIZER)
        .expect("serialization fail unlikely");
    set_with_value_keys(&keys).await
}

pub async fn set_with_value_keys(keys: &JsValue) -> Result<(), CustomError> {
    JsFuture::from(storage_set(keys)).await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("store to")
        }))?;
    Ok(())
}
