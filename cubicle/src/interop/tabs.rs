//! Wrappers around the `browser.tabs` API.
//! Most fails are represented by
//! [FailedTabOperation](CustomError::FailedTabOperation).

use std::collections::HashMap;

use js_sys::{Array, Promise};
use serde::{Deserialize, Serialize, Serializer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::contextual_identities::CookieStoreId;
use crate::domain::EncodedDomain;
use crate::interop;
use crate::util::errors::CustomError;

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

/// Retained properties that affect tab creation,
/// deserializes from a `Tab` instance and
/// serializes to a `create_properties` instance.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabProperties {
    active: bool,
    #[serde(
        deserialize_with = "CookieStoreId::deserialize_inner",
        serialize_with = "CookieStoreId::serialize_inner"
    )]
    pub cookie_store_id: CookieStoreId,
    discarded: Option<bool>,
    #[serde(skip_serializing)]
    id: isize,
    index: usize,
    #[serde(rename(serialize = "muted"))]
    muted_info: MutedInfo,
    opener_tab_id: Option<TabId>,
    #[serde(rename(deserialize = "isInReaderMode", serialize = "openInReaderMode"))]
    reader_mode: Option<bool>, // found to be optional
    pinned: bool,
    url: Option<String>,
    window_id: isize,
}

impl TabProperties {
    /// The domain, [None] if the tab does not have a URL.
    /// Fails if a domain cannot be extracted from the contained URL.
    pub fn domain(&self) -> Result<Option<EncodedDomain>, CustomError> {
        let Some(url) = &self.url else { return Ok(None); };
        interop::url_to_domain(url).map(Some)
    }

    /// Create a new tab using this instance,
    /// whether the resulting tab completely matches is unchecked.
    /// Fails if the browser indicates so.
    pub async fn new_tab(&self) -> Result<TabId, CustomError> {
        let new_properties = interop::cast_or_standard_mismatch::<Self>(
            JsFuture::from(tab_create(interop::to_jsvalue(self)))
                .await
                .or(Err(CustomError::FailedTabOperation {
                    verb: String::from("create"),
                }))?,
        )?;
        Ok(TabId::new(new_properties.id))
    }
}

/// Unique identifier that allow operations on specific tabs.
/// All operations may fail if the tab specified by the ID does not exist.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TabId {
    inner: isize,
}

impl TabId {
    /// Creates a new ID by trusting the given value.
    /// May be replaced by [FromWasmAbi](wasm_bindgen::convert::FromWasmAbi)
    /// later for brevity and clarity.
    pub fn new(tab_id: isize) -> Self {
        Self { inner: tab_id }
    }

    /// Stops the specified tab from loading.
    /// This function does not return errors,
    /// as failures are not recoverable and the tab will be loaded.
    pub async fn stop_loading(&self) {
        let details = interop::to_jsvalue(&HashMap::from([
            ("code", "window.stop();"),
            ("runAt", "document_start"),
        ]));
        drop(JsFuture::from(tab_execute_js(self.inner, details)).await);
    }

    /// Closes the specified tab, fails if the browser indicates so.
    pub async fn close_tab(&self) -> Result<(), CustomError> {
        interop::cast_or_standard_mismatch(JsFuture::from(tab_remove(self.inner)).await.or(Err(
            CustomError::FailedTabOperation {
                verb: String::from("remove"),
            },
        ))?)
    }

    /// Reloads the specified tab, fails if the browser indicates so.
    pub async fn reload_tab(&self) -> Result<(), CustomError> {
        interop::cast_or_standard_mismatch(JsFuture::from(tab_reload(self.inner)).await.or(Err(
            CustomError::FailedTabOperation {
                verb: String::from("reload"),
            },
        ))?)
    }
}

/// Structure contained in [TabProperties] that requires
/// asymmetric serialization.
/// No interfaces are exposed as this is only used for conversion.
#[derive(Deserialize)]
struct MutedInfo {
    muted: bool,
}

impl Serialize for MutedInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(self.muted)
    }
}

/// Gets the [CookieStoreId] of the current tab.
/// Fails with [FailedFetchActiveTab](CustomError::FailedFetchActiveTab)
/// if there is no active tab in the current window.
pub async fn current_tab_cookie_store_id() -> Result<CookieStoreId, CustomError> {
    let query_obj = HashMap::from([("active", true), ("currentWindow", true)]);
    let active_tabs = JsFuture::from(tab_query(interop::to_jsvalue(&query_obj))).await;
    if let Ok(active_tabs) = active_tabs.as_ref().map(Array::from) {
        let prop = super::get_or_standard_mismatch(&active_tabs.pop(), "cookieStoreId")?;
        Ok(CookieStoreId::new(super::cast_or_standard_mismatch(prop)?))
    } else {
        Err(CustomError::FailedFetchActiveTab)
    }
}
