//! Import functions for migrating from vanilla containers,
//! or from other container providers.

use serde::Deserialize;

use crate::container::{Container, ContainerOwner, ContainerVariant};
use crate::interop::contextual_identities::{ContextualIdentity, IdentityDetailsProvider};
use crate::util::errors::CustomError;

/// Provider of the containers to migrate from.
/// - [Native](MigrateType::Native) means that the provider is the browser itself,
///   and no additional container information is attached.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "migrate_type")]
pub enum MigrateType {
    Native,
}

impl MigrateType {
    /// Performs container migration.
    /// Fails if the browser indicates so.
    pub async fn act(&self, detect_temp: bool) -> Result<ContainerOwner, CustomError> {
        let containers = fetch_all_containers(detect_temp).await?;
        use MigrateType::*;
        match *self {
            Native => Ok(containers),
        }
    }
}

/// Fetches all [ContextualIdentity] and treat them as [Container],
/// detects temporary containers by name if needed.
/// Fails if the browser indicates so.
pub async fn fetch_all_containers(detect_temp: bool) -> Result<ContainerOwner, CustomError> {
    Ok(ContainerOwner::from_iter(
        ContextualIdentity::fetch_all()
            .await?
            .into_iter()
            .map(|identity| {
                let name = identity.identity_details().name;
                let mut container = Container::from(identity);
                if detect_temp && name.starts_with("Temporary Container ") {
                    container.variant = ContainerVariant::Temporary;
                }
                container
            }),
    ))
}
