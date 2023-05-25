use serde::{Deserialize, Serialize};

use crate::container::{Container, ContainerOwner};
use crate::domain::psl::Psl;
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityDetails, IdentityDetailsProvider
};
use crate::interop::storage;
use crate::util::errors::CustomError;

#[derive(Default, Deserialize, Serialize)]
pub struct GlobalContext {
    #[serde(flatten)]
    pub containers: ContainerOwner,
    pub psl: Psl
}

impl GlobalContext {
    pub async fn from_storage() -> Result<Self, CustomError> {
        let mut stored_version = Version::default();
        storage::get_with_keys(&mut stored_version).await?;
        if stored_version == Version::default() {
            Ok(GlobalContext::default())
        } else if stored_version != CURRENT_VERSION {
            Err(CustomError::UnsupportedVersion)
        } else {
            let mut context = GlobalContext::default();
            storage::get_with_keys(&mut context).await?;
            Ok(context)
        }
    }

    pub async fn fetch_all_containers(&mut self)
    -> Result<Vec<(&CookieStoreId, IdentityDetails)>, CustomError> {
        self.containers = ContainerOwner::from_iter(
            ContextualIdentity::fetch_all()
            .await?.into_iter().map(Container::from));
        Ok(self.containers.iter().map(|container| {
            (container.cookie_store_id(), container.identity_details())
        }).collect())
    }
}

#[derive(Default, Deserialize, Eq, PartialEq, Serialize)]
struct Version { pub version: (i16, i16, i16) }
const CURRENT_VERSION: Version = Version { version: (0, 1, 0) };
