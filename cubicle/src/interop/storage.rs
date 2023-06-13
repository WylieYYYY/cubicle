use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::interop;
use crate::util::{self, errors::CustomError};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="get")]
    fn storage_get(keys: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="set")]
    fn storage_set(keys: &JsValue) -> Promise;
}

pub async fn get_with_keys<T>(keys: &mut T) -> Result<(), CustomError>
where T: for <'de> Deserialize<'de> + Serialize {
    let got = JsFuture::from(storage_get(&util::to_jsvalue(keys))).await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("load from")
        }))?;
    *keys = interop::cast_or_standard_mismatch(got)?;
    Ok(())
}

pub async fn set_with_serde_keys<T>(keys: &T) -> Result<(), CustomError>
where T: Serialize {
    set_with_value_keys(&util::to_jsvalue(keys)).await
}

pub async fn set_with_value_keys(keys: &JsValue) -> Result<(), CustomError> {
    JsFuture::from(storage_set(keys)).await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("store to")
        }))?;
    Ok(())
}
