use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::EncodedDomain;
use crate::domain::suffix::{self, Suffix};
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityDetails, IdentityDetailsProvider
};
use crate::util::errors::CustomError;

#[derive(Default, Deserialize, Serialize)]
pub struct ContainerOwner {
    #[serde(skip)]
    suffix_id_map: BTreeMap<Suffix, CookieStoreId>,
    #[serde(flatten)]
    id_container_map: HashMap<CookieStoreId, Container>
}

impl ContainerOwner {
    pub fn insert(&mut self, container: Container) {
        for suffix in container.suffixes.iter() {
            self.suffix_id_map.insert(suffix.clone(),
                (**container.handle()).clone());
        }
        self.id_container_map.insert((**container.handle())
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

    pub fn match_container(&mut self, domain: EncodedDomain)
    -> Option<ContainerMatch> {
        let matches = suffix::match_suffix(&self.suffix_id_map, domain);
        for (matched_domain, suffix) in matches {
            let cookie_store_id = self.suffix_id_map.get(&suffix)
                .expect("suffix matched");
            if let Some(container) = self.id_container_map
                .remove(cookie_store_id) {
                let container = self.id_container_map
                    .entry(cookie_store_id.clone()).or_insert(container);
                return Some(ContainerMatch {
                    container, matched_domain, suffix
                });
            }
        }
        None
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

pub struct ContainerMatch<'a> {
    pub container: &'a mut Container,
    pub matched_domain: EncodedDomain,
    pub suffix: Suffix
}

#[derive(Deserialize, Serialize)]
pub struct Container {
    handle: Arc<CookieStoreId>,
    identity: ContextualIdentity,
    pub variant: ContainerVariant,
    pub suffixes: BTreeSet<Suffix>
}

impl Container {
    pub async fn create(details: IdentityDetails,
        variant: ContainerVariant, suffixes: BTreeSet<Suffix>)
    -> Result<Self, CustomError> {
        let identity = ContextualIdentity::create(details).await?;
        let handle = Arc::new(identity.cookie_store_id().clone());
        Ok(Self { handle, identity, variant, suffixes })
    }
    pub async fn update(&mut self, details: IdentityDetails)
    -> Result<(), CustomError> {
        self.identity.update(details).await.and(Ok(()))
    }
    pub async fn delete(&self) -> Result<(), CustomError> {
        self.identity.cookie_store_id().delete_identity().await
    }

    pub async fn delete_if_empty(&mut self) -> Result<bool, CustomError> {
        match Arc::get_mut(&mut self.handle) {
            Some(_cookie_store_id) => self.delete().await.and(Ok(true)),
            None => Ok(false)
        }
    }

    pub fn handle(&self) -> &Arc<CookieStoreId> {
        &self.handle
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
            handle: Arc::new(identity.cookie_store_id().clone()),
            identity, variant: ContainerVariant::Permanent,
            suffixes: BTreeSet::default()
        }
    }
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
pub enum ContainerVariant {
    Permanent, Temporary
}
