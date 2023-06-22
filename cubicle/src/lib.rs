mod container;
mod context;
mod domain;
mod interop;
mod message;
mod preferences;
mod util;

use std::collections::HashMap;
use std::panic;

use async_std::sync::Mutex;
use js_sys::JsString;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use crate::context::GlobalContext;
use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;
use crate::interop::tabs::{TabId, TabProperties};
use crate::message::Message;
use crate::util::errors::CustomError;

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    *global_context = GlobalContext::from_storage().await.unwrap();
    if global_context.psl.len() == 0 {
        Message::PslUpdate { url: None }.act(&mut global_context).await.unwrap();
    }
    drop(global_context.fetch_all_containers().await);
    Ok(())
}

static GLOBAL_CONTEXT: Lazy<Mutex<GlobalContext>> = Lazy::new(||
    Mutex::new(GlobalContext::default()));
static MANAGED_TABS: Lazy<Mutex<HashMap<TabId, EncodedDomain>>> = Lazy::new(||
    Mutex::new(HashMap::default()));

#[wasm_bindgen(js_name="onMessage")]
pub async fn on_message(message: JsValue) -> Result<JsString, JsError> {
    let message = serde_wasm_bindgen::from_value::<Message>(message)
        .expect("unexpected message format");
    message.act(&mut GLOBAL_CONTEXT.lock().await).await
        .map(|html| JsString::from(html))
        .map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen(js_name="onTabUpdated")]
pub async fn on_tab_updated(tab_id: isize, tab_properties: JsValue)
-> Result<(), JsError> {
    {
        let tab_id = TabId::new(tab_id);
        let tab_properties = interop::cast_or_standard_mismatch
            ::<TabProperties>(tab_properties)?;

        let Some(new_domain) = tab_new_domain(tab_id.clone(),
            &tab_properties).await else { return Ok(()); };
        tab_id.stop_loading().await;

        let mut global_context = GLOBAL_CONTEXT.lock().await;
        let cookie_store_id = global_context.preferences.assign_strategy
            .clone().match_container(&mut global_context, new_domain.clone()).await?;
        drop(global_context);

        assign_tab(tab_id, tab_properties, cookie_store_id, new_domain).await
    }.map_err(|error: CustomError| JsError::new(&error.to_string()))
}

async fn tab_new_domain(tab_id: TabId, tab_properties: &TabProperties)
-> Option<EncodedDomain> {
    let new_url = tab_properties.url().as_ref()?;
    let new_domain = interop::url_to_domain(&new_url).ok()?;
    let old_domain = MANAGED_TABS.lock().await.insert(tab_id,
        new_domain.clone());
    if old_domain.map_or(true, |old_domain| old_domain != new_domain) {
        Some(new_domain)
    } else { None }
}

async fn assign_tab(tab_id: TabId, mut tab_properties: TabProperties,
    cookie_store_id: CookieStoreId, new_domain: EncodedDomain)
-> Result<(), CustomError> {
    if cookie_store_id == tab_properties.cookie_store_id {
        MANAGED_TABS.lock().await.insert(tab_id.clone(), new_domain);
        tab_id.reload_tab().await
    } else {
        tab_properties.cookie_store_id = cookie_store_id;
        let new_tab_id = tab_properties.new_tab().await?;
        MANAGED_TABS.lock().await.insert(new_tab_id, new_domain);
        tab_id.close_tab().await
    }
}
