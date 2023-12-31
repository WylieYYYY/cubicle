//! Structures that allow checking if a tab may need to be relocated.

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;
use crate::interop::tabs::{TabId, TabProperties};

/// Determinant that stores the current handle for bypassing context lock.
/// Contains all detail that are used to determine if the tab does not require
/// relocation for certain.
pub struct TabDeterminant {
    pub container_handle: Arc<CookieStoreId>,
    pub domain: Option<EncodedDomain>,
}

/// Detail required for determining where the tab should be relocated to.
/// When wrapped in [Option], it indicates whether relocation should occur.
pub struct RelocationDetail {
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

        let opener_det = tab_properties
            .opener_tab_id()
            .and_then(|tab_id| self.determinant_map.get(tab_id));
        let opener_handle = opener_det.map(|tab_det| Arc::clone(&tab_det.container_handle));
        let opener_domain = opener_det.and_then(|tab_det| tab_det.domain.clone());

        let same_domain_as_opener = opener_domain.as_ref() == Some(&new_domain);

        let current_cookie_store_id = (*self
            .determinant_map
            .entry(tab_id)
            .and_modify(|old_det| {
                let new_domain = Some(new_domain.clone());
                if old_det.domain == new_domain {
                    same_domain = true;
                } else {
                    old_det.domain = new_domain;
                }
            })
            .or_insert(TabDeterminant {
                container_handle: opener_handle
                    .filter(|_| same_domain_as_opener)
                    .unwrap_or_else(|| Arc::new(tab_properties.cookie_store_id.clone())),
                domain: Some(new_domain.clone()),
            })
            .container_handle)
            .clone();

        (!same_domain && !same_domain_as_opener).then_some(RelocationDetail {
            new_domain,
            current_cookie_store_id,
            opener_is_managed: opener_domain.is_some(),
        })
    }

    /// Invalidates the domain stored and forces an extended check.
    /// Does nothing if the tab specified is not managed.
    pub fn invalidate_domain(&mut self, tab_id: &TabId) {
        if let Some(tab_det) = self.determinant_map.get_mut(tab_id) {
            tab_det.domain = None;
        }
    }

    /// Registers a tab for quick relocation lookup later.
    pub fn register(&mut self, tab_id: TabId, tab_det: TabDeterminant) {
        self.determinant_map.insert(tab_id, tab_det);
    }

    /// Unregisters a tab to avoid possible collision.
    /// Returns a [TabDeterminant] if the tab was managed, [None] otherwise.
    pub fn unregister(&mut self, tab_id: &TabId) -> Option<TabDeterminant> {
        self.determinant_map.remove(tab_id)
    }
}
