//! Wrappers around the `browser.storage.local` API.
//! Most fails are represented by
//! [FailedStorageOperation](CustomError::FailedStorageOperation).

use js_sys::{Object, Promise, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::interop;
use crate::util::errors::CustomError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="get")]
    fn storage_get(keys: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="set")]
    fn storage_set(keys: &JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "storage", "local"], js_name="remove")]
    fn storage_remove(keys: &JsValue) -> Promise;
}

/// Gets all stored entries as an object,
/// fails if the browser indicates so.
pub async fn get_all() -> Result<Object, CustomError> {
    JsFuture::from(storage_get(&JsValue::NULL))
        .await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("load from"),
        }))
        .map(Object::from)
}

/// Removes all entries with the given collection of keys,
/// fails if the browser indicates so.
pub async fn remove_entries<S, K>(keys: &S) -> Result<(), CustomError>
where
    S: IntoIterator<Item = K> + Serialize,
    K: Serialize,
{
    JsFuture::from(storage_remove(&interop::to_jsvalue(keys)))
        .await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("remove from"),
        }))?;
    Ok(())
}

/// Populates a structure with values, fails if the browser indicates so.
/// Can fail with [StandardMismatch](CustomError::StandardMismatch) if some
/// types of returned values are not compatible to the structure.
pub async fn get_with_keys<T>(keys: &mut T) -> Result<(), CustomError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let got = JsFuture::from(storage_get(&interop::to_jsvalue(keys)))
        .await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("load from"),
        }))?;
    *keys = interop::cast_or_standard_mismatch(got)?;
    Ok(())
}

/// Sets values with a structural representation,
/// fails if the browser indicates so.
pub async fn set_with_serde_keys<T>(keys: &T) -> Result<(), CustomError>
where
    T: Serialize,
{
    set_with_value_keys(&interop::to_jsvalue(keys)).await
}

/// Sets values with a [JsValue] in a structural representation,
/// fails if the browser indicates so.
pub async fn set_with_value_keys(keys: &JsValue) -> Result<(), CustomError> {
    JsFuture::from(storage_set(keys))
        .await
        .or(Err(CustomError::FailedStorageOperation {
            verb_prep: String::from("store to"),
        }))?;
    Ok(())
}

/// Sets a single value with a key, fails if the browser indicates so.
pub async fn store_single_entry<K, V>(key: &K, value: &V) -> Result<(), CustomError>
where
    K: Serialize + ?Sized,
    V: Serialize,
{
    let keys = Object::new();
    Reflect::set(
        &keys,
        &interop::to_jsvalue(key),
        &interop::to_jsvalue(value),
    )
    .expect("inline construction");
    set_with_value_keys(&keys).await
}
