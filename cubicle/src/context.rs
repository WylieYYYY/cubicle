//! Data that are persisted to the storage with version control.

use js_sys::{JsString, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::container::{ContainerOwner, ContainerVariant};
use crate::domain::psl::Psl;
use crate::interop::contextual_identities::CookieStoreId;
use crate::interop::{self, storage};
use crate::message::Message;
use crate::migrate::{self, Version};
use crate::preferences::Preferences;
use crate::util::errors::CustomError;

/// Persisting data for determining which container to switch to.
#[derive(Default, Deserialize, Serialize)]
pub struct GlobalContext {
    #[serde(flatten)]
    pub containers: ContainerOwner,
    #[serde(default)]
    pub psl: Psl,
    #[serde(default)]
    pub preferences: Preferences,
}

impl GlobalContext {
    /// Populates a context after checking the version for compatibility.
    /// Fails with [CustomError::UnsupportedVersion]
    /// or if the browser indicates so.
    pub async fn from_storage() -> Result<Self, CustomError> {
        let mut stored_version = Version::default();
        storage::get_with_keys(&mut stored_version).await?;
        let mut context = GlobalContext::default();
        if stored_version == Version::default() {
            storage::set_with_serde_keys(&context).await?;
            storage::set_with_serde_keys(&migrate::CURRENT_VERSION).await?;
            Message::PslUpdate { url: None }
                .act(&mut &mut context)
                .await?;
            Ok(context)
        } else if stored_version != migrate::CURRENT_VERSION {
            Err(CustomError::UnsupportedVersion)
        } else {
            let all_stored = storage::get_all().await?;
            Reflect::delete_property(&all_stored, &JsString::from("version"))
                .expect("constructed object from get all function");
            context = interop::cast_or_standard_mismatch(JsValue::from(all_stored))?;

            if context.psl.is_empty() {
                Message::PslUpdate { url: None }
                    .act(&mut &mut context)
                    .await?;
            }

            context.purge_temporary_containers().await?;
            Ok(context)
        }
    }

    /// Deletes and remove temporary containers from the [ContainerOwner].
    /// Fails if the browser indicates so.
    /// May be changed in the future to accommodate session restore.
    async fn purge_temporary_containers(&mut self) -> Result<(), CustomError> {
        let temp_handles = self
            .containers
            .iter()
            .filter(|container| container.variant == ContainerVariant::Temporary)
            .map(|container| container.handle().cookie_store_id().clone())
            .collect::<Vec<CookieStoreId>>();
        for cookie_store_id in &temp_handles {
            if let Some(container) = self.containers.remove(cookie_store_id) {
                container.delete().await?;
            }
        }
        storage::remove_entries(&temp_handles).await
    }
}
