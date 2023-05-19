use std::collections::{HashMap, BTreeMap};

use crate::domain::suffix::{Suffix, SuffixSet};
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityDetails, IdentityDetailsProvider
};
use crate::util::errors::CustomError;

#[derive(Default)]
pub struct ContainerOwner {
    suffix_id_map: BTreeMap<Suffix, CookieStoreId>,
    id_container_map: HashMap<CookieStoreId, Container>
}

impl ContainerOwner {
    pub fn insert(&mut self, container: Container) {
        for suffix in container.suffixes.iter() {
            self.suffix_id_map.insert(suffix.clone(),
                container.cookie_store_id().clone());
        }
        self.id_container_map.insert(container.cookie_store_id()
            .clone(), container);
    }
    pub fn get(&self, cookie_store_id: &CookieStoreId) -> Option<&Container> {
        self.id_container_map.get(cookie_store_id)
    }
    pub fn get_mut(&mut self, cookie_store_id: &CookieStoreId)
    -> Option<&mut Container> {
        self.id_container_map.get_mut(cookie_store_id)
    }
    pub fn remove(&mut self, cookie_store_id: &CookieStoreId)
    -> Option<Container> {
        self.id_container_map.remove(cookie_store_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Container> {
        self.id_container_map.values()
    }
}

impl FromIterator<Container> for ContainerOwner {
    fn from_iter<T>(iter: T) -> Self
    where T: IntoIterator<Item = Container> {
        let mut instance = Self::default();
        for container in iter { instance.insert(container); }
        instance
    }
}

pub struct Container {
    identity: ContextualIdentity, pub variant: ContainerVariant,
    pub suffixes: SuffixSet
}

impl Container {
    pub async fn create(details: IdentityDetails, variant: ContainerVariant)
    -> Result<Self, CustomError> {
        let identity = ContextualIdentity::create(details).await?;
        Ok(Self { identity, variant, suffixes: SuffixSet::default() })
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
        Self {
            identity, variant: ContainerVariant::Permanent,
            suffixes: SuffixSet::default()
        }
    }
}

pub enum ContainerVariant { Permanent }