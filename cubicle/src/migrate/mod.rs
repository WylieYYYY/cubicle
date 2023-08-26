//! Migration module for importing from existing configurations,
//! or from an older version.

pub mod import;

use serde::{Deserialize, Serialize};

/// Versioning of [GlobalContext](crate::context::GlobalContext)
/// for migrating and detecteing older version.
/// The versioning scheme is to be decided in the next release.
#[derive(Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Version {
    pub version: (i16, i16, i16),
}
pub const CURRENT_VERSION: Version = Version { version: (0, 1, 0) };
