//! Additional functionalities for the builtin [ContextualIdentity].

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::suffix::{self, MatchMode, Suffix, SuffixType};
use crate::domain::EncodedDomain;
#[mockall_double::double]
use crate::interop::contextual_identities::ContextualIdentity;
use crate::interop::contextual_identities::{
    CookieStoreId, IdentityDetails, IdentityDetailsProvider,
};
use crate::interop::storage;
use crate::interop::tabs::TabId;
use crate::tab::RelocationDetail;
use crate::util::errors::CustomError;

/// A glorified lookup table for [Container],
/// either from a [CookieStoreId] or an [EncodedDomain].
#[derive(Default, Deserialize, Serialize)]
pub struct ContainerOwner {
    #[serde(skip)]
    suffix_id_map: BTreeMap<Suffix, CookieStoreId>,
    #[serde(flatten)]
    id_container_map: HashMap<CookieStoreId, Container>,
}

impl ContainerOwner {
    /// Fetches all [ContextualIdentity] and treat them as [Container],
    /// detects temporary containers by name if needed.
    /// Returns a new [ContainerOwner] with all containers detected.
    /// Fails if the browser indicates so.
    pub async fn fetch_all(detect_temp: bool) -> Result<Self, CustomError> {
        let containers = ContextualIdentity::fetch_all()
            .await?
            .into_iter()
            .map(|identity| {
                let name = identity.identity_details().name;
                let mut container = Container::from(identity);
                if detect_temp && name.starts_with("Temporary Container ") {
                    container.variant = ContainerVariant::Temporary;
                }
                container
            });
        let mut owner = Self::default();
        for container in containers {
            owner.insert(container);
        }
        Ok(owner)
    }

    /// Inserts a container, this will also add suffix mappings for lookup.
    pub fn insert(&mut self, container: Container) {
        if container.variant.allows_suffix_match() {
            for suffix in container.suffixes.iter() {
                self.suffix_id_map
                    .insert(suffix.clone(), (**container.handle()).clone());
            }
        }
        self.id_container_map
            .insert((**container.handle()).clone(), container);
    }

    /// Gets an owned container immutably,
    /// [None] if the container specified does not exist.
    pub fn get(&self, cookie_store_id: &CookieStoreId) -> Option<&Container> {
        self.id_container_map.get(cookie_store_id)
    }

    /// Gets an owned container mutably, wrapped with an [OwnerHandle].
    /// [None] if the container specified does not exist.
    pub fn get_mut(&mut self, cookie_store_id: CookieStoreId) -> Option<OwnerHandle> {
        if self.id_container_map.get_mut(&cookie_store_id).is_some() {
            Some(OwnerHandle {
                owner: self,
                cookie_store_id,
            })
        } else {
            None
        }
    }

    /// Remove an owned container.
    /// Returns the popped container, or [None] if not found.
    pub fn remove(&mut self, cookie_store_id: &CookieStoreId) -> Option<Container> {
        let container = self.id_container_map.remove(cookie_store_id);
        self.suffix_id_map
            .retain(|_suffix, id| *id != *cookie_store_id);
        container
    }

    /// Matches a container to the given domain by the stored suffixes,
    /// skipping over the removed containers.
    /// Returns a [ContainerMatch], [None] if there is no match.
    /// Glob suffix may not match if the container with the corresponding
    /// normal suffix is removed, this may be fixed in the future.
    pub fn match_container(&mut self, domain: EncodedDomain) -> Option<ContainerMatch> {
        let matches = suffix::match_suffix(&self.suffix_id_map, domain, MatchMode::Full);
        for (matched_domain, suffix) in matches {
            let cookie_store_id = self.suffix_id_map.get(&suffix).expect("suffix matched");
            if let Some(container) = self.id_container_map.remove(cookie_store_id) {
                let container = self
                    .id_container_map
                    .entry(cookie_store_id.clone())
                    .or_insert(container);
                return Some(ContainerMatch {
                    container,
                    matched_domain,
                    suffix,
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

/// Handle of a [Container] that is owned by a [ContainerOwner].
/// Dereferences into a container.
/// When dropped, the owner's suffix lookup table is updated.
pub struct OwnerHandle<'a> {
    owner: &'a mut ContainerOwner,
    cookie_store_id: CookieStoreId,
}

impl Deref for OwnerHandle<'_> {
    type Target = Container;

    fn deref(&self) -> &Self::Target {
        self.owner
            .id_container_map
            .get(&self.cookie_store_id)
            .expect("held mutable reference to owner")
    }
}

impl DerefMut for OwnerHandle<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.owner
            .id_container_map
            .get_mut(&self.cookie_store_id)
            .expect("held mutable reference to owner")
    }
}

impl Drop for OwnerHandle<'_> {
    fn drop(&mut self) {
        if !self.variant.allows_suffix_match() {
            return;
        }
        self.owner
            .suffix_id_map
            .retain(|_suffix, cookie_store_id| *cookie_store_id != self.cookie_store_id);
        let suffixes = self.suffixes.clone().into_iter();
        self.owner
            .suffix_id_map
            .extend(suffixes.map(|suffix| (suffix, self.cookie_store_id.clone())));
    }
}

/// Structure for storing a match from [ContainerOwner::match_container].
/// This is used to reduce repetitive container lookup and domain matching.
pub struct ContainerMatch<'a> {
    pub container: &'a mut Container,
    pub matched_domain: EncodedDomain,
    pub suffix: Suffix,
}

/// Wrapper around [ContextualIdentity] with handle.
#[derive(Deserialize, Serialize)]
pub struct Container {
    handle: Arc<CookieStoreId>,
    identity: ContextualIdentity,
    pub variant: ContainerVariant,
    pub suffixes: BTreeSet<Suffix>,
}

impl Container {
    /// Creates a new container, fails if the browser indicates so.
    pub async fn create(
        details: IdentityDetails,
        variant: ContainerVariant,
        suffixes: BTreeSet<Suffix>,
    ) -> Result<Self, CustomError> {
        let identity = ContextualIdentity::create(details).await?;
        let handle = Arc::new(identity.cookie_store_id().clone());
        Ok(Self {
            handle,
            identity,
            variant,
            suffixes,
        })
    }

    /// Updates this container using the given [IdentityDetails].
    pub async fn update(&mut self, details: IdentityDetails) -> Result<(), CustomError> {
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
            None => Ok(false),
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
            identity,
            variant: ContainerVariant::Permanent,
            suffixes: BTreeSet::default(),
        }
    }
}

/// Variants of containers.
/// - [Permanent](ContainerVariant::Permanent) means that the container is
///   created by the user and all container operations are managed by the user.
/// - [Recording](ContainerVariant::Recording) means that the container should
///   be recreated with the new name after tabs movements are captured.
/// - [Temporary](ContainerVariant::Temporary) means that the container is
///   generated, and should be deleted once all tabs within it have closed.
#[derive(Deserialize, Eq, PartialEq, Serialize)]
pub enum ContainerVariant {
    Permanent,
    Recording { active: bool },
    Temporary,
}

impl ContainerVariant {
    /// Variant-specific actions to take before a tab is
    /// relocated to a new container.
    /// Returns the passed [RelocationDetail] if relocation should proceed,
    /// [None] otherwise.
    /// Fails if the browser indicates so.
    pub async fn on_pre_relocation(
        containers: &mut ContainerOwner,
        tab_id: &TabId,
        relocation_detail: RelocationDetail,
    ) -> Result<Option<RelocationDetail>, CustomError> {
        let Some(mut container) =
            containers.get_mut(relocation_detail.current_cookie_store_id.clone())
        else {
            return Ok(Some(relocation_detail));
        };
        match container.variant {
            Self::Recording { active: true } => {
                container.suffixes.insert(Suffix::new(
                    SuffixType::Normal,
                    relocation_detail.new_domain,
                ));
                tab_id.reload_tab().await.and(Ok(None))
            }
            Self::Permanent | Self::Recording { active: false } | Self::Temporary => {
                Ok(Some(relocation_detail))
            }
        }
    }

    /// Variant-specific actions to take when a container handle is dropped.
    /// [CookieStoreId] indicates which container's handle was dropped.
    /// Fails if the browser indicates so.
    pub async fn on_handle_drop(
        containers: &mut ContainerOwner,
        cookie_store_id: CookieStoreId,
    ) -> Result<(), CustomError> {
        let Some(mut container) = containers.get_mut(cookie_store_id.clone()) else {
            return Ok(());
        };
        match container.variant {
            Self::Temporary => {
                let deleted = container.delete_if_empty().await.unwrap_or(false);
                drop(container);
                if deleted {
                    containers.remove(&cookie_store_id);
                    storage::remove_entries(&[cookie_store_id]).await
                } else {
                    Ok(())
                }
            }
            Self::Permanent | Self::Recording { .. } => Ok(()),
        }
    }

    /// Checks if suffixes from a specific container should be matched.
    pub fn allows_suffix_match(&self) -> bool {
        match *self {
            Self::Permanent | Self::Temporary => true,
            Self::Recording { .. } => false,
        }
    }
}

#[cfg(test)]
pub mod test {
    use async_std::sync::Mutex;
    use once_cell::sync::Lazy;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::interop::contextual_identities::{CookieStoreId, MockContextualIdentity};

    static CONTEXTUAL_IDENTITY_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    async fn test_container(
        details: IdentityDetails,
        suffixes: BTreeSet<Suffix>,
        mock_identity_setup: impl FnOnce(&mut MockContextualIdentity),
    ) -> Container {
        let mut mock_identity = MockContextualIdentity::new();
        mock_identity
            .expect_cookie_store_id()
            .return_const(CookieStoreId::new(String::from("mock_id")));
        mock_identity_setup(&mut mock_identity);
        let ctx_mock_identity = MockContextualIdentity::create_context();
        ctx_mock_identity.expect().return_once(|details| {
            assert_eq!(IdentityDetails::default(), details);
            Ok(mock_identity)
        });

        Container::create(details, ContainerVariant::Temporary, suffixes)
            .await
            .expect("mocked contextual identity")
    }

    #[wasm_bindgen_test]
    async fn test_container_create_and_handle() -> Result<(), CustomError> {
        let _guard = CONTEXTUAL_IDENTITY_MUTEX.lock().await;
        let container =
            test_container(IdentityDetails::default(), BTreeSet::default(), |_| ()).await;

        assert_eq!(1usize, Arc::strong_count(container.handle()));
        assert_eq!(
            CookieStoreId::new(String::from("mock_id")),
            **container.handle()
        );
        Ok(())
    }
}
