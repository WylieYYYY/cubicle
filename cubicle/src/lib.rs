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
use web_sys::console;

use crate::context::GlobalContext;
use crate::domain::EncodedDomain;
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
    let exmaple_com = EncodedDomain::try_from("example.com").unwrap();
    console::log_1(&JsValue::from_bool(global_context.psl.match_suffix(
        exmaple_com).is_some()));
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
        let mut managed_tabs = MANAGED_TABS.lock().await;

        let Some(new_url) = tab_properties.url() else { return Ok(()); };
        let new_domain = interop::url_to_domain(&new_url)?;
        let old_domain = managed_tabs.insert(tab_id.clone(),
            new_domain.clone());
        if old_domain.map_or(false, |old_domain| old_domain == new_domain) {
            return Ok(());
        }

        tab_id.stop_loading().await;
        let registered_new_domain = new_domain.clone();
        let register_tab = move |new_tab_id| {
            managed_tabs.insert(new_tab_id, registered_new_domain);
        };

        let global_context = GLOBAL_CONTEXT.lock().await;
        let cookie_store_id = global_context.preferences.assign_strategy
            .match_container(&global_context.containers, new_domain);
        drop(global_context);

        if cookie_store_id == tab_properties.cookie_store_id {
            register_tab(tab_id.clone());
            tab_id.reload_tab().await
        } else {
            tab_id.enter(cookie_store_id, tab_properties, register_tab).await
        }
    }.map_err(|error: CustomError| JsError::new(&error.to_string()))
}
