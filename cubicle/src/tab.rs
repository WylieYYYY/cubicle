use std::sync::Arc;

use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;

pub struct TabDeterminant {
    pub container_handle: Arc<CookieStoreId>,
    pub domain: EncodedDomain
}
