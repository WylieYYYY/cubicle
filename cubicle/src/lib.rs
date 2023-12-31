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

use std::panic;

use async_std::sync::Mutex;
use js_sys::JsString;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;

use crate::container::ContainerVariant;
use crate::context::GlobalContext;
use crate::interop::tabs::{TabId, TabProperties};
use crate::message::Message;
use crate::tab::{ManagedTabs, TabDeterminant};
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

/// Managed tabs lookup for quick interception.
static MANAGED_TABS: Lazy<Mutex<ManagedTabs>> = Lazy::new(|| Mutex::new(ManagedTabs::default()));

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

        let Some(relocation_detail) = MANAGED_TABS
            .lock()
            .await
            .check_relocation(tab_id.clone(), &tab_properties)
        else {
            return Ok(());
        };
        drop(tab_id.stop_loading().await);

        let mut global_context = GLOBAL_CONTEXT.lock().await;

        let Some(relocation_detail) = ContainerVariant::on_pre_relocation(
            &mut global_context.containers,
            &tab_id,
            relocation_detail,
        )
        .await?
        else {
            return Ok(());
        };

        let eject_strategy = global_context.preferences.eject_strategy.clone();
        let assign_strategy = global_context.preferences.assign_strategy.clone();
        let should_revert_old_tab = global_context.preferences.should_revert_old_tab;

        let container_handle = if relocation_detail.opener_is_managed {
            eject_strategy
                .match_container(
                    &mut global_context,
                    relocation_detail.new_domain.clone(),
                    &relocation_detail.current_cookie_store_id,
                    assign_strategy,
                )
                .await?
        } else {
            assign_strategy
                .match_container(&mut global_context, relocation_detail.new_domain.clone())
                .await?
        };
        drop(global_context);

        let tab_det = TabDeterminant {
            container_handle,
            domain: Some(relocation_detail.new_domain),
        };
        assign_tab(tab_id, tab_properties, tab_det, should_revert_old_tab).await
    }
    .map_err(|error: CustomError| JsError::new(&error.to_string()))
}

/// Cleans up end of life containers when a tab is closed.
/// Best effort with no error as it is optional,
/// as cleanup is not possible when the browser is closed anyway.
#[wasm_bindgen(js_name = "onTabRemoved")]
pub async fn on_tab_removed(tab_id: isize) {
    let tab_id = TabId::new(tab_id);
    let Some(tab_det) = MANAGED_TABS.lock().await.unregister(&tab_id) else {
        return;
    };
    let cookie_store_id = (*tab_det.container_handle).clone();
    drop(tab_det);
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    drop(ContainerVariant::on_handle_drop(&mut global_context.containers, cookie_store_id).await);
}

/// Switchs the tab to a [Container](crate::container::Container)
/// specified by the [TabDeterminant].
/// Fails if any tab operation failed.
async fn assign_tab(
    tab_id: TabId,
    mut tab_properties: TabProperties,
    tab_det: TabDeterminant,
    should_revert_old_tab: bool,
) -> Result<(), CustomError> {
    if *tab_det.container_handle == tab_properties.cookie_store_id {
        MANAGED_TABS.lock().await.register(tab_id.clone(), tab_det);
        tab_id.reload_tab().await
    } else {
        if should_revert_old_tab {
            MANAGED_TABS.lock().await.invalidate_domain(&tab_id);
            tab_id.back_or_close().await?;
        } else {
            tab_id.close_tab().await?;
        }

        tab_properties.cookie_store_id = (*tab_det.container_handle).clone();
        let new_tab_id = tab_properties.new_tab().await?;
        MANAGED_TABS.lock().await.register(new_tab_id, tab_det);
        Ok(())
    }
}
