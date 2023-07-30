//! Structure that allows checking if a tab may need to be relocated.

use std::sync::Arc;

use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;

/// Structure that allows checking if a tab may need to be relocated.
/// This does not lock up the context, and should have no false negative,
/// false positive should be kept at a minimum level without speed penalty.
/// Currently this just checks for a domain change.
pub struct TabDeterminant {
    pub container_handle: Arc<CookieStoreId>,
    pub domain: EncodedDomain
}
