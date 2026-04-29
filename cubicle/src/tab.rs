//! Structures that allow checking if a tab may need to be relocated.

use std::collections::HashMap;
use std::mem;

use crate::container::ContainerHandle;
use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;
use crate::interop::tabs::TabId;
#[mockall_double::double]
use crate::interop::tabs::TabProperties;

/// Determinant that stores the current handle for bypassing context lock.
/// Contains all detail that are used to determine if the tab does not require
/// relocation for certain.
pub struct TabDeterminant {
    pub container_handle: ContainerHandle,
    pub domain: Option<EncodedDomain>,
}

/// Detail required for determining where the tab should be relocated to.
/// When wrapped in [Option], it indicates whether relocation should occur.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
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
/// May be made stricter for certain edge cases.
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

        let (opener_is_managed, mut stolen_tab_det) =
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
                    tab_properties.cookie_store_id(),
                    &mut stolen_tab_det,
                    &new_domain,
                )
            })
            .container_handle
            .cookie_store_id()
            .clone();

        // it is possible to steal tab determinant from a managed opener with the current tab already managed
        // then we just drop the stolen tab determinant
        if let Some(stolen_tab_det) = stolen_tab_det {
            // safe to finish since the opener is not being removed
            // we could not have stolen this handle otherwise
            stolen_tab_det.container_handle.finish();
        }

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
        stolen_tab_det: &mut Option<TabDeterminant>,
        new_domain: &EncodedDomain,
    ) -> TabDeterminant {
        // if opener handle exists, no relocation happens, handle must be kept
        stolen_tab_det.take().unwrap_or_else(|| {
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

#[cfg(test)]
pub mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::interop::tabs::MockTabProperties;
    use crate::util::test::TestFrom;

    const OPENER_TAB_ID: TabId = TabId::new(1isize);
    const TAB_ID: TabId = TabId::new(2isize);

    #[wasm_bindgen_test]
    fn test_unmanaged_tab_with_managed_opener_same_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(OPENER_TAB_ID.clone(), make_tab_det("example.com"));

        assert_eq!(
            None,
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_unmanaged_tab_with_managed_opener_different_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(OPENER_TAB_ID.clone(), make_tab_det("example.org"));

        assert_eq!(
            Some(RelocationDetail {
                old_domain: None,
                new_domain: EncodedDomain::tfrom("example.com"),
                current_cookie_store_id: CookieStoreId::new(String::from("mock_id")),
                opener_is_managed: true,
            }),
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_managed_tab_with_managed_opener_same_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(OPENER_TAB_ID.clone(), make_tab_det("example.com"));
        managed_tabs.register(TAB_ID.clone(), make_tab_det("example.com"));

        assert_eq!(
            None,
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_managed_tab_with_managed_opener_different_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(OPENER_TAB_ID.clone(), make_tab_det("example.org"));
        managed_tabs.register(TAB_ID.clone(), make_tab_det("example.org"));

        assert_eq!(
            Some(RelocationDetail {
                old_domain: Some(EncodedDomain::tfrom("example.org")),
                new_domain: EncodedDomain::tfrom("example.com"),
                current_cookie_store_id: CookieStoreId::new(String::from("mock_managed_id")),
                opener_is_managed: true,
            }),
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_unmanaged_tab_without_opener() {
        let mut managed_tabs = ManagedTabs::default();

        assert_eq!(
            Some(RelocationDetail {
                old_domain: None,
                new_domain: EncodedDomain::tfrom("example.com"),
                current_cookie_store_id: CookieStoreId::new(String::from("mock_id")),
                opener_is_managed: false,
            }),
            check_relocation_with_example_com(&mut managed_tabs, None)
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_managed_tab_without_opener_same_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(TAB_ID.clone(), make_tab_det("example.com"));

        assert_eq!(
            None,
            check_relocation_with_example_com(&mut managed_tabs, None)
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_managed_tab_without_opener_different_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(TAB_ID.clone(), make_tab_det("example.org"));

        assert_eq!(
            Some(RelocationDetail {
                old_domain: Some(EncodedDomain::tfrom("example.org")),
                new_domain: EncodedDomain::tfrom("example.com"),
                current_cookie_store_id: CookieStoreId::new(String::from("mock_managed_id")),
                opener_is_managed: false,
            }),
            check_relocation_with_example_com(&mut managed_tabs, None)
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_unmanaged_tab_with_unmanaged_opener() {
        let mut managed_tabs = ManagedTabs::default();

        assert_eq!(
            Some(RelocationDetail {
                old_domain: None,
                new_domain: EncodedDomain::tfrom("example.com"),
                current_cookie_store_id: CookieStoreId::new(String::from("mock_id")),
                opener_is_managed: false,
            }),
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_managed_tab_with_unmanaged_opener_same_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(TAB_ID.clone(), make_tab_det("example.com"));

        assert_eq!(
            None,
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    #[wasm_bindgen_test]
    fn test_managed_tab_with_unmanaged_opener_different_domain() {
        let mut managed_tabs = ManagedTabs::default();
        managed_tabs.register(TAB_ID.clone(), make_tab_det("example.org"));

        assert_eq!(
            Some(RelocationDetail {
                old_domain: Some(EncodedDomain::tfrom("example.org")),
                new_domain: EncodedDomain::tfrom("example.com"),
                current_cookie_store_id: CookieStoreId::new(String::from("mock_managed_id")),
                opener_is_managed: false,
            }),
            check_relocation_with_example_com(&mut managed_tabs, Some(&OPENER_TAB_ID))
        );

        finish_all_handles(managed_tabs);
    }

    fn make_tab_det(domain: &str) -> TabDeterminant {
        let handle = ContainerHandle::from(CookieStoreId::new(String::from("mock_managed_id")));
        TabDeterminant {
            container_handle: handle,
            domain: Some(EncodedDomain::tfrom(domain)),
        }
    }

    fn check_relocation_with_example_com(
        managed_tabs: &mut ManagedTabs,
        opener_tab_id: Option<&'static TabId>,
    ) -> Option<RelocationDetail> {
        let mut mock_tab_properties = MockTabProperties::new();
        mock_tab_properties
            .expect_cookie_store_id()
            .return_const(CookieStoreId::new(String::from("mock_id")));
        mock_tab_properties
            .expect_domain()
            .returning(|| Ok(Some(EncodedDomain::tfrom("example.com"))));
        mock_tab_properties
            .expect_opener_tab_id()
            .return_const(opener_tab_id);

        managed_tabs.check_relocation(TAB_ID, &mock_tab_properties)
    }

    fn finish_all_handles(mut managed_tabs: ManagedTabs) {
        managed_tabs
            .get_mut(&OPENER_TAB_ID)
            .inspect(|tab_det| tab_det.container_handle.finish());
        managed_tabs
            .get_mut(&TAB_ID)
            .inspect(|tab_det| tab_det.container_handle.finish());
    }
}
