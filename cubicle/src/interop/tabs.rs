use std::collections::HashMap;

use js_sys::{Array, Promise};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::contextual_identities::CookieStoreId;
use super::MAP_SERIALIZER;
use crate::util::errors::CustomError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="create")]
    fn tab_create(create_properties: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="query")]
    fn tab_query(query_obj: JsValue) -> Promise;
}

pub async fn current_tab_cookie_store_id()
-> Result<CookieStoreId, CustomError> {
    let query_obj = HashMap::from([("active", true), ("currentWindow", true)]);
    let active_tabs = JsFuture::from(tab_query(query_obj
        .serialize(MAP_SERIALIZER).expect("inline construction"))).await;
    if let Ok(active_tabs) = active_tabs.as_ref().map(Array::from) {
        let prop = super::get_or_standard_mismatch(
            &active_tabs.pop(), "cookieStoreId")?;
        Ok(CookieStoreId::new(super::cast_or_standard_mismatch(prop)?))
    } else { Err(CustomError::FailedFetchActiveTab) }
}

pub fn create_tab(_: Box<[JsValue]>) -> Promise {
    let mut args = HashMap::new();
    args.insert("url", "https://wylieyyyy.gitlab.io");
    match args.serialize(MAP_SERIALIZER) {
        Ok(serialized) => tab_create(serialized),
        Err(err) => Promise::reject(&JsValue::from(err))
    }
}
