use std::collections::{BTreeMap, BTreeSet, HashMap};

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

    pub fn match_container(&self, domain: EncodedDomain)
    -> Option<ContainerMatch> {
        suffix::match_suffix(&self.suffix_id_map, domain).find_map(
            |(matched_domain, suffix)| {
            let container_id = self.suffix_id_map.get(&suffix)
                .expect("suffix matched");
            self.id_container_map.get(&container_id).map(|container| {
                ContainerMatch { container, matched_domain, suffix }
            })
        })
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
    pub container: &'a Container,
    pub matched_domain: EncodedDomain,
    pub suffix: Suffix
}

#[derive(Deserialize, Serialize)]
pub struct Container {
    identity: ContextualIdentity, pub variant: ContainerVariant,
    pub suffixes: BTreeSet<Suffix>
}

impl Container {
    pub async fn create(details: IdentityDetails,
        variant: ContainerVariant, suffixes: BTreeSet<Suffix>)
    -> Result<Self, CustomError> {
        let identity = ContextualIdentity::create(details).await?;
        Ok(Self { identity, variant, suffixes })
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
            suffixes: BTreeSet::default()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum ContainerVariant {
    Permanent, Temporary { tab_count: usize }
}
