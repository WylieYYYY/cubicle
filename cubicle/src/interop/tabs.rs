use std::collections::HashMap;

use js_sys::{Array, Promise};
use serde::{Deserialize, Serialize, Serializer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::contextual_identities::CookieStoreId;
use crate::interop;
use crate::util::{self, errors::CustomError};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="create")]
    fn tab_create(create_properties: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="query")]
    fn tab_query(query_obj: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="remove")]
    fn tab_remove(tab_id: isize) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="executeScript")]
    fn tab_execute_js(tab_id: isize, details: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "tabs"], js_name="reload")]
    fn tab_reload(tab_id: isize) -> Promise;
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct TabProperties {
    active: bool,
    #[serde(deserialize_with="CookieStoreId::deserialize_inner",
        serialize_with="CookieStoreId::serialize_inner")]
    pub cookie_store_id: CookieStoreId,
    discarded: Option<bool>,
    #[serde(skip_serializing)]
    id: isize,
    index: usize,
    #[serde(rename(serialize="muted"))]
    muted_info: MutedInfo,
    opener_tab_id: Option<TabId>,
    #[serde(rename(deserialize="isInReaderMode",
        serialize="openInReaderMode"))]
    reader_mode: Option<bool>, // found to be optional
    pinned: bool,
    url: Option<String>, window_id: isize
}

impl TabProperties {
    pub fn url(&self) -> &Option<String> { &self.url }

    pub async fn new_tab(&self) -> Result<TabId, CustomError> {
        let new_properties = interop::cast_or_standard_mismatch::<Self>(
            JsFuture::from(tab_create(util::to_jsvalue(self))).await.or(Err(
            CustomError::FailedTabOperation {
                verb: String::from("create")
            }))?)?;
        Ok(TabId::new(new_properties.id))
    }
}

#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TabId { inner: isize }

impl TabId {
    pub fn new(tab_id: isize) -> Self {
        Self { inner: tab_id }
    }
    pub async fn stop_loading(&self) {
        let details = util::to_jsvalue(&HashMap::from([
            ("code", "window.stop();"), ("runAt", "document_start")
        ]));
        drop(JsFuture::from(tab_execute_js(self.inner, details)).await);
    }
    pub async fn close_tab(&self) -> Result<(), CustomError> {
        interop::cast_or_standard_mismatch(JsFuture::from(
            tab_remove(self.inner)).await.or(Err(
            CustomError::FailedTabOperation {
                verb: String::from("remove")
            }))?)
    }
    pub async fn reload_tab(&self) -> Result<(), CustomError> {
        interop::cast_or_standard_mismatch(JsFuture::from(
            tab_reload(self.inner)).await.or(Err(
            CustomError::FailedTabOperation {
                verb: String::from("reload")
            }))?)
    }
}

#[derive(Deserialize)]
pub struct MutedInfo { muted: bool }

impl Serialize for MutedInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_bool(self.muted)
    }
}

pub async fn current_tab_cookie_store_id()
-> Result<CookieStoreId, CustomError> {
    let query_obj = HashMap::from([("active", true), ("currentWindow", true)]);
    let active_tabs = JsFuture::from(tab_query(
        util::to_jsvalue(&query_obj))).await;
    if let Ok(active_tabs) = active_tabs.as_ref().map(Array::from) {
        let prop = super::get_or_standard_mismatch(
            &active_tabs.pop(), "cookieStoreId")?;
        Ok(CookieStoreId::new(super::cast_or_standard_mismatch(prop)?))
    } else { Err(CustomError::FailedFetchActiveTab) }
}
