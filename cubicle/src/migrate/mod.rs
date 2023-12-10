//! Migration module for importing from existing configurations,
//! or from an older version.

pub mod import;

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Versioning of [GlobalContext](crate::context::GlobalContext)
/// for migrating and detecteing older version.
/// The versioning scheme is to be decided in the next release.
#[derive(Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Version {
    pub version: (i16, i16, i16),
}
pub const CURRENT_VERSION: Version = Version { version: (0, 1, 0) };
pub static BUILTIN_PSL_VERSION: Lazy<NaiveDate> = Lazy::new(|| {
    NaiveDate::from_ymd_opt(2023, 5, 8).expect("date checked to be valid at compile time")
});

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn test_psl_version_no_panic() {
        let _ = BUILTIN_PSL_VERSION.clone();
    }
}
