//! Import functions for migrating from vanilla containers,
//! or from other container providers.

use serde::Deserialize;

use crate::container::ContainerOwner;
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
        let containers = ContainerOwner::fetch_all(detect_temp).await?;
        use MigrateType::*;
        match *self {
            Native => Ok(containers),
        }
    }
}
