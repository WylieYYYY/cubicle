//! Structures that allow checking if a tab may need to be relocated.

use std::collections::HashMap;
use std::mem;

use crate::container::ContainerHandle;
use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;
use crate::interop::tabs::{TabId, TabProperties};

/// Determinant that stores the current handle for bypassing context lock.
/// Contains all detail that are used to determine if the tab does not require
/// relocation for certain.
pub struct TabDeterminant {
    pub container_handle: ContainerHandle,
    pub domain: Option<EncodedDomain>,
}

/// Detail required for determining where the tab should be relocated to.
/// When wrapped in [Option], it indicates whether relocation should occur.
pub struct RelocationDetail {
    pub old_domain: Option<EncodedDomain>,
    pub new_domain: EncodedDomain,
    pub current_cookie_store_id: CookieStoreId,
    pub opener_is_managed: bool,
}

/// Structure that allows checking if a tab may need to be relocated.
/// This does not lock up the context.
/// Should be synchronous as this is used before tab interception.
/// Currently this just checks for a domain change.
#[derive(Default)]
pub struct ManagedTabs {
    determinant_map: HashMap<TabId, TabDeterminant>,
}

impl ManagedTabs {
    /// Checks quickly to see if the tab requires relocating.
    /// If the tab is to be relocated, returns a [RelocationDetail],
    /// [None] otherwise.
    pub fn check_relocation(
        &mut self,
        tab_id: TabId,
        tab_properties: &TabProperties,
    ) -> Option<RelocationDetail> {
        let new_domain = tab_properties.domain().ok()??;
        let mut same_domain = false;
        let mut old_domain = None;

        let (opener_is_managed, stolen_tab_det) =
            self.steal_opener_tab_det(tab_properties, &new_domain);
        let has_failed_to_steal = stolen_tab_det.is_none();

        let current_cookie_store_id = self
            .determinant_map
            .entry(tab_id)
            .and_modify(|old_det| {
                (same_domain, old_domain) = Self::replace_tab_det_domain(old_det, &new_domain)
            })
            .or_insert_with(|| {
                Self::use_stolen_tab_det_or_fake(
                    &tab_properties.cookie_store_id,
                    stolen_tab_det,
                    &new_domain,
                )
            })
            .container_handle
            .cookie_store_id()
            .clone();

        (!same_domain && has_failed_to_steal).then_some(RelocationDetail {
            old_domain,
            new_domain,
            current_cookie_store_id,
            opener_is_managed,
        })
    }

    /// Registers a tab for quick relocation lookup later.
    pub fn register(&mut self, tab_id: TabId, tab_det: TabDeterminant) -> Option<TabDeterminant> {
        self.determinant_map.insert(tab_id, tab_det)
    }

    /// Gets a mutable reference to [TabDeterminant] for modifying, [None] if it does not exist.
    pub fn get_mut(&mut self, tab_id: &TabId) -> Option<&mut TabDeterminant> {
        self.determinant_map.get_mut(tab_id)
    }

    /// Unregisters a tab to avoid possible collision.
    /// Returns a [TabDeterminant] if the tab was managed, [None] otherwise.
    pub fn unregister(&mut self, tab_id: &TabId) -> Option<TabDeterminant> {
        self.determinant_map.remove(tab_id)
    }

    /// Replaces the domain in the given [TabDeterminant] with the new domain
    /// if the new domain is not the same as the old one, including if the old one is [None].
    /// Returns a tuple of whether they are the same and the old domain or [None].
    fn replace_tab_det_domain(
        tab_det: &mut TabDeterminant,
        new_domain: &EncodedDomain,
    ) -> (bool, Option<EncodedDomain>) {
        let new_domain = Some(new_domain);
        let same_domain = tab_det.domain.as_ref() == new_domain;
        match same_domain {
            true => (true, None),
            false => (
                false,
                mem::replace(&mut tab_det.domain, new_domain.cloned()),
            ),
        }
    }

    /// Steal a [TabDeterminant] from the opener if they are still managed and have the same domain.
    /// Returns a tuple of whether the opener is managed and the stolen tab determinant if the criteria are satisfied.
    fn steal_opener_tab_det(
        &mut self,
        tab_properties: &TabProperties,
        new_domain: &EncodedDomain,
    ) -> (bool, Option<TabDeterminant>) {
        let opener_det = tab_properties
            .opener_tab_id()
            .and_then(|tab_id| self.determinant_map.get(tab_id));

        let opener_domain = opener_det.and_then(|tab_det| tab_det.domain.as_ref());

        let stolen_handle = opener_det
            .filter(|_| opener_domain == Some(new_domain))
            .map(|tab_det| TabDeterminant {
                container_handle: tab_det.container_handle.clone(),
                domain: Some(new_domain.clone()),
            });

        (opener_domain.is_some(), stolen_handle)
    }

    /// Returns the stolen [TabDeterminant] if it is given or a fake handle.
    /// The fake handle should be replaced with a real handle when the global context is available.
    fn use_stolen_tab_det_or_fake(
        current_cookie_store_id: &CookieStoreId,
        stolen_tab_det: Option<TabDeterminant>,
        new_domain: &EncodedDomain,
    ) -> TabDeterminant {
        // if opener handle exists, no relocation happens, handle must be kept
        stolen_tab_det.unwrap_or_else(|| {
            // otherwise use a fake handle, still ok if the container with the ID is deleted before reaching there
            // this handle will never be stored for a long time as no current tab related actions can be performed
            // but this handle will be used for pre-relocation so the ID still needs to be correct
            let handle = ContainerHandle::from(current_cookie_store_id.clone());
            handle.finish();
            TabDeterminant {
                container_handle: handle,
                domain: Some(new_domain.clone()),
            }
        })
    }
}
