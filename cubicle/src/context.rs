//! Data that are persisted to the storage with version control.

use serde::{Deserialize, Serialize};

use crate::container::ContainerOwner;
use crate::domain::psl::Psl;
use crate::interop::storage;
use crate::message::Message;
use crate::migrate::{self, Version};
use crate::preferences::Preferences;
use crate::util::errors::CustomError;

/// Persisting data for determining which container to switch to.
#[derive(Default, Deserialize, Serialize)]
pub struct GlobalContext {
    #[serde(flatten)]
    pub containers: ContainerOwner,
    pub psl: Psl,
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
            storage::get_with_keys(&mut context).await?;
            Ok(context)
        }
    }
}
