//! Additional functionalities for the builtin [ContextualIdentity].

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::EncodedDomain;
use crate::domain::suffix::{self, Suffix};
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityDetails, IdentityDetailsProvider
};
use crate::util::errors::CustomError;

/// A glorified lookup table for [Container],
/// either from a [CookieStoreId] or an [EncodedDomain].
#[derive(Default, Deserialize, Serialize)]
pub struct ContainerOwner {
    #[serde(skip)]
    suffix_id_map: BTreeMap<Suffix, CookieStoreId>,
    #[serde(flatten)]
    id_container_map: HashMap<CookieStoreId, Container>
}

impl ContainerOwner {
    /// Inserts a container, this will also add suffix mappings for lookup.
    pub fn insert(&mut self, container: Container) {
        for suffix in container.suffixes.iter() {
            self.suffix_id_map.insert(suffix.clone(),
                (**container.handle()).clone());
        }
        self.id_container_map.insert((**container.handle())
            .clone(), container);
    }

    /// Gets an owned container immutably,
    /// [None] if the container specified does not exist.
    pub fn get(&self, cookie_store_id: &CookieStoreId) -> Option<&Container> {
        self.id_container_map.get(cookie_store_id)
    }

    /// Gets an owned container mutably,
    /// [None] if the container specified does not exist.
    pub fn get_mut(&mut self, cookie_store_id: &CookieStoreId)
    -> Option<&mut Container> {
        self.id_container_map.get_mut(cookie_store_id)
    }

    /// Remove an owned container, this does not remove the suffix mappings.
    /// The suffixes may be cleaned up later.
    /// Returns the popped container, or [None] if not found.
    pub fn remove(&mut self, cookie_store_id: &CookieStoreId)
    -> Option<Container> {
        self.id_container_map.remove(cookie_store_id)
    }

    /// Matches a container to the given domain by the stored suffixes,
    /// skipping over the removed containers.
    /// Returns a [ContainerMatch], [None] if there is no match.
    /// Glob suffix may not match if the container with the corresponding
    /// normal suffix is removed, this may be fixed in the future.
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

    /// Iterator over owned containers.
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

/// Structure for storing a match from [ContainerOwner::match_container].
/// This is used to reduce repetitive container lookup and domain matching.
pub struct ContainerMatch<'a> {
    pub container: &'a mut Container,
    pub matched_domain: EncodedDomain,
    pub suffix: Suffix
}

/// Wrapper around [ContextualIdentity] with handle.
#[derive(Deserialize, Serialize)]
pub struct Container {
    handle: Arc<CookieStoreId>,
    identity: ContextualIdentity,
    pub variant: ContainerVariant,
    pub suffixes: BTreeSet<Suffix>
}

impl Container {
    /// Creates a new container, fails if the browser indicates so.
    pub async fn create(details: IdentityDetails,
        variant: ContainerVariant, suffixes: BTreeSet<Suffix>)
    -> Result<Self, CustomError> {
        let identity = ContextualIdentity::create(details).await?;
        let handle = Arc::new(identity.cookie_store_id().clone());
        Ok(Self { handle, identity, variant, suffixes })
    }

    /// Updates this container using the given [IdentityDetails].
    pub async fn update(&mut self, details: IdentityDetails)
    -> Result<(), CustomError> {
        self.identity.update(details).await.and(Ok(()))
    }

    /// Deletes this container, fails if the browser indicates so.
    pub async fn delete(&self) -> Result<(), CustomError> {
        self.identity.cookie_store_id().delete_identity().await
    }

    /// Deletes this container if there is no external handle holder.
    /// Any further operations on this instance will fail.
    /// Returns whether this instance is deleted.
    /// Fails if the browser indicates so.
    pub async fn delete_if_empty(&mut self) -> Result<bool, CustomError> {
        match Arc::get_mut(&mut self.handle) {
            Some(_cookie_store_id) => self.delete().await.and(Ok(true)),
            None => Ok(false)
        }
    }

    /// Handle to this container, the holder must clean up the container
    /// appropriately after releasing the handle.
    /// For example, by using [Container::delete_if_empty].
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
    /// Builds a container from a [ContextualIdentity].
    /// By default, it will be a [ContainerVariant::Permanent] container
    /// with no suffixes. This matches the builtin behaviour.
    fn from(identity: ContextualIdentity) -> Self {
        Self {
            handle: Arc::new(identity.cookie_store_id().clone()),
            identity, variant: ContainerVariant::Permanent,
            suffixes: BTreeSet::default()
        }
    }
}

/// Variants of containers.
/// - [Permanent](ContainerVariant::Permanent) means that the container is
///   created by the user and all container operations are managed by the user.
/// - [Temporary](ContainerVariant::Temporary) means that the container is
///   generated, and should be deleted once all tabs within it have closed.
#[derive(Deserialize, Eq, PartialEq, Serialize)]
pub enum ContainerVariant {
    Permanent, Temporary
}
