mod container;
mod context;
mod domain;
mod interop;
mod message;
mod preferences;
mod tab;
mod util;

use std::collections::HashMap;
use std::panic;
use std::sync::Arc;

use async_std::sync::Mutex;
use js_sys::JsString;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use crate::container::ContainerVariant;
use crate::context::GlobalContext;
use crate::domain::EncodedDomain;
use crate::interop::tabs::{TabId, TabProperties};
use crate::message::Message;
use crate::tab::TabDeterminant;
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
static MANAGED_TABS: Lazy<Mutex<HashMap<TabId, TabDeterminant>>> = Lazy::new(||
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
        let container_handle = global_context.preferences.assign_strategy
            .clone().match_container(&mut global_context,
            new_domain.clone()).await?;
        drop(global_context);

        let tab_det = TabDeterminant { container_handle, domain: new_domain };
        assign_tab(tab_id, tab_properties, tab_det).await
    }.map_err(|error: CustomError| JsError::new(&error.to_string()))
}

#[wasm_bindgen(js_name="onTabRemoved")]
pub async fn on_tab_removed(tab_id: isize) -> Result<(), JsError> {
    let tab_id = TabId::new(tab_id);
    let Some(tab_det) = MANAGED_TABS.lock().await.remove(&tab_id) else {
        return Ok(());
    };
    let cookie_store_id = (*tab_det.container_handle).clone();
    drop(tab_det);
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    let Some(container) = global_context.containers
        .get_mut(&cookie_store_id) else { return Ok(()); };

    if container.variant == ContainerVariant::Temporary {
        let deleted = container.delete_if_empty().await
            .map_err(|error: CustomError| JsError::new(&error.to_string()))?;
        if deleted { global_context.containers.remove(&cookie_store_id); }
    }

    Ok(())
}

async fn tab_new_domain(tab_id: TabId, tab_properties: &TabProperties)
-> Option<EncodedDomain> {
    let new_domain = tab_properties.domain().ok()??;
    let mut managed_tabs = MANAGED_TABS.lock().await;
    let mut same_domain = false;

    managed_tabs.entry(tab_id).and_modify(|old_det| {
        if old_det.domain == new_domain { same_domain = true; }
        else { old_det.domain = new_domain.clone(); }
    }).or_insert(TabDeterminant {
        container_handle: Arc::new(tab_properties.cookie_store_id.clone()),
        domain: new_domain.clone()
    });

    if same_domain {
        None
    } else { Some(new_domain) }
}

async fn assign_tab(tab_id: TabId, mut tab_properties: TabProperties,
    tab_det: TabDeterminant)
-> Result<(), CustomError> {
    if *tab_det.container_handle == tab_properties.cookie_store_id {
        MANAGED_TABS.lock().await.insert(tab_id.clone(), tab_det);
        tab_id.reload_tab().await
    } else {
        tab_properties.cookie_store_id = (*tab_det.container_handle).clone();
        let new_tab_id = tab_properties.new_tab().await?;
        MANAGED_TABS.lock().await.insert(new_tab_id, tab_det);
        tab_id.close_tab().await
    }
}
