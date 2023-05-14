use crate::domain::suffix::SuffixSet;
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityDetails, IdentityDetailsProvider
};
use crate::util::errors::CustomError;

pub struct Container {
    identity: ContextualIdentity, pub variant: ContainerVariant
}

impl Container {
    pub async fn create(details: IdentityDetails, variant: ContainerVariant)
    -> Result<Self, CustomError> {
        let identity = ContextualIdentity::create(details).await?;
        Ok(Self { identity, variant })
    }
    pub async fn update(&mut self, details: IdentityDetails)
    -> Result<(), CustomError> {
        self.identity.update(details).await.and(Ok(()))
    }
    pub async fn delete(self) -> Result<(), Self> {
        self.identity.cookie_store_id().delete_identity().await.or(Err(self))
    }
    pub fn cookie_store_id(&self) -> &CookieStoreId {
        &self.identity.cookie_store_id()
    }
}

impl IdentityDetailsProvider for Container {
    fn identity_details(&self) -> IdentityDetails {
        self.identity.identity_details()
    }
}

impl From<ContextualIdentity> for Container {
    fn from(identity: ContextualIdentity) -> Self {
        Self { identity, variant: ContainerVariant::default() }
    }
}

pub enum ContainerVariant { Permanent(SuffixSet) }

impl Default for ContainerVariant {
    fn default() -> Self { Self::Permanent(SuffixSet::default()) }
}
