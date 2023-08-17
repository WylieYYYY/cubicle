//! Data that are persisted to the storage with version control.

use serde::{Deserialize, Serialize};

use crate::container::{Container, ContainerOwner};
use crate::domain::psl::Psl;
use crate::interop::contextual_identities::ContextualIdentity;
use crate::interop::storage;
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
            storage::set_with_serde_keys(&CURRENT_VERSION).await?;
            Ok(context)
        } else if stored_version != CURRENT_VERSION {
            Err(CustomError::UnsupportedVersion)
        } else {
            storage::get_with_keys(&mut context).await?;
            Ok(context)
        }
    }

    /// Fetches all [ContextualIdentity] and treat them as [Container].
    /// Temporary function before importing is implemented,
    /// may be removed in the future.
    /// Fails if the browser indicates so.
    pub async fn fetch_all_containers(&mut self) -> Result<(), CustomError> {
        self.containers = ContainerOwner::from_iter(
            ContextualIdentity::fetch_all()
                .await?
                .into_iter()
                .map(Container::from),
        );
        Ok(())
    }
}

/// Versioning of [GlobalContext] for migrating and detecteing older version.
/// The versioning scheme is to be decided in the next release.
#[derive(Default, Deserialize, Eq, PartialEq, Serialize)]
struct Version {
    pub version: (i16, i16, i16),
}
const CURRENT_VERSION: Version = Version { version: (0, 1, 0) };
