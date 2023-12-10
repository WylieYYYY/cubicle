//! A flexible container manager.

#[cfg(test)]
use wasm_bindgen_test::wasm_bindgen_test_configure;
#[cfg(test)]
wasm_bindgen_test_configure!(run_in_worker);

pub mod container;
pub mod context;
pub mod domain;
pub mod interop;
pub mod message;
pub mod migrate;
pub mod preferences;
pub mod tab;
pub mod util;

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
use crate::interop::contextual_identities::CookieStoreId;
use crate::interop::tabs::{TabId, TabProperties};
use crate::message::Message;
use crate::tab::TabDeterminant;
use crate::util::errors::CustomError;

/// Entry point for loading this extension.
/// Mainly to load or populate a [GlobalContext].
#[wasm_bindgen(start)]
async fn start() -> Result<(), JsError> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    *global_context = GlobalContext::from_storage()
        .await
        .map_err(|error: CustomError| JsError::new(&error.to_string()))?;
    Ok(())
}

/// Persisting data for determining which container to switch to.
static GLOBAL_CONTEXT: Lazy<Mutex<GlobalContext>> =
    Lazy::new(|| Mutex::new(GlobalContext::default()));

/// Map to check if tab needs to be relocated using a [TabDeterminant],
/// there should be no false negative.
/// Locking should only be done by short synchronous tasks,
/// as a check is done before stop loading to prevent infinite reload.
static MANAGED_TABS: Lazy<Mutex<HashMap<TabId, TabDeterminant>>> =
    Lazy::new(|| Mutex::new(HashMap::default()));

/// Message passing function for user actions other than tab changes.
/// See [Message] for all possible message types.
/// Returns and failures are specific to the message types.
#[wasm_bindgen(js_name = "onMessage")]
pub async fn on_message(message: JsValue) -> Result<JsString, JsError> {
    let message =
        serde_wasm_bindgen::from_value::<Message>(message).expect("unexpected message format");
    message
        .act(&mut GLOBAL_CONTEXT.lock().await)
        .await
        .map(JsString::from)
        .map_err(|error| JsError::new(&error.to_string()))
}

/// Intercepts the tabs for container operations.
/// First stop the tab loading, and recreate the tab if a container switch
/// is required, reload the tab otherwise.
#[wasm_bindgen(js_name = "onTabUpdated")]
pub async fn on_tab_updated(tab_id: isize, tab_properties: JsValue) -> Result<(), JsError> {
    {
        let tab_id = TabId::new(tab_id);
        let tab_properties = interop::cast_or_standard_mismatch::<TabProperties>(tab_properties)?;

        let Some((new_domain, cookie_store_id, opener_is_managed)) =
            tab_new_domain(tab_id.clone(), &tab_properties).await
        else {
            return Ok(());
        };
        tab_id.stop_loading().await;

        let mut global_context = GLOBAL_CONTEXT.lock().await;
        let assign_strategy = global_context.preferences.assign_strategy.clone();
        let container_handle = if opener_is_managed {
            global_context
                .preferences
                .eject_strategy
                .clone()
                .match_container(
                    &mut global_context,
                    new_domain.clone(),
                    &cookie_store_id,
                    assign_strategy,
                )
                .await?
        } else {
            assign_strategy
                .match_container(&mut global_context, new_domain.clone())
                .await?
        };
        drop(global_context);

        let tab_det = TabDeterminant {
            container_handle,
            domain: new_domain,
        };
        assign_tab(tab_id, tab_properties, tab_det).await
    }
    .map_err(|error: CustomError| JsError::new(&error.to_string()))
}

/// Cleans up end of life containers when a tab is closed.
/// Best effort with no error as it is optional,
/// as cleanup is not possible when the browser is closed anyway.
#[wasm_bindgen(js_name = "onTabRemoved")]
pub async fn on_tab_removed(tab_id: isize) {
    let tab_id = TabId::new(tab_id);
    let Some(tab_det) = MANAGED_TABS.lock().await.remove(&tab_id) else {
        return;
    };
    let cookie_store_id = (*tab_det.container_handle).clone();
    drop(tab_det);
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    let Some(mut container) = global_context.containers.get_mut(cookie_store_id.clone()) else {
        return;
    };

    if container.variant == ContainerVariant::Temporary {
        let deleted = container.delete_if_empty().await.unwrap_or(false);
        drop(container);
        if deleted {
            global_context.containers.remove(&cookie_store_id);
        }
    }
}

/// Checks [MANAGED_TABS] quickly to see if the tab requires switching.
/// If the tab is to be switched, returns a tuple of the new domain,
/// current [CookieStoreId], and a boolean value indicating
/// if the opener tab was managed, [None] otherwise.
async fn tab_new_domain(
    tab_id: TabId,
    tab_properties: &TabProperties,
) -> Option<(EncodedDomain, CookieStoreId, bool)> {
    let new_domain = tab_properties.domain().ok()??;
    let mut managed_tabs = MANAGED_TABS.lock().await;
    let mut same_domain = false;

    let opener_domain = tab_properties
        .opener_tab_id()
        .and_then(|tab_id| managed_tabs.get(tab_id))
        .map(|tab_det| tab_det.domain.clone());
    let cookie_store_id = (*managed_tabs
        .entry(tab_id)
        .and_modify(|old_det| {
            if old_det.domain == new_domain {
                same_domain = true;
            } else {
                old_det.domain = new_domain.clone();
            }
        })
        .or_insert(TabDeterminant {
            container_handle: Arc::new(tab_properties.cookie_store_id.clone()),
            domain: new_domain.clone(),
        })
        .container_handle)
        .clone();

    (!same_domain && opener_domain.as_ref() != Some(&new_domain)).then_some((
        new_domain,
        cookie_store_id,
        opener_domain.is_some(),
    ))
}

/// Switchs the tab to a [Container](crate::container::Container)
/// specified by the [TabDeterminant].
/// Fails if any tab operation failed.
async fn assign_tab(
    tab_id: TabId,
    mut tab_properties: TabProperties,
    tab_det: TabDeterminant,
) -> Result<(), CustomError> {
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
