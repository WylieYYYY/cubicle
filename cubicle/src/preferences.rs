//! All preferences that are not container or storage item specific.

use std::collections::BTreeSet;
use std::sync::Arc;

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::container::{Container, ContainerVariant};
use crate::context::GlobalContext;
use crate::domain::suffix::{Suffix, SuffixType};
use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::{CookieStoreId, IdentityDetails};
use crate::interop::storage;
use crate::util::errors::CustomError;

/// All preferences that are not container or storage item specific.
#[derive(Default, Deserialize, Serialize)]
pub struct Preferences {
    pub assign_strategy: ContainerAssignStrategy,
    pub eject_strategy: ContainerEjectStrategy,
}

/// Assigning strategy for tabs that are previously not contained,
/// mainly addresses what happens if no permanent container accepts the tab.
/// - [SuffixedTemporary](ContainerAssignStrategy::SuffixedTemporary) means
///   that the tab will be assigned to a new or existing temporary container
///   that matches the public suffix of the domain.
/// - [IsolatedTemporary](ContainerAssignStrategy::IsolatedTemporary) means
///   that a new temporary container will always be created for the tab.
#[derive(Clone, Derivative, Deserialize, Eq, PartialEq, Serialize)]
#[derivative(Default)]
#[serde(rename_all = "snake_case")]
pub enum ContainerAssignStrategy {
    #[derivative(Default)]
    SuffixedTemporary,
    IsolatedTemporary,
}

impl ContainerAssignStrategy {
    /// Matches a tab's domain to an accepting container, regardless of type.
    /// Returns a container handle that must be properly released.
    /// Fails if the browser indicates so.
    #[must_use = "clean up must be done before releasing the handle"]
    pub async fn match_container(
        &self,
        global_context: &mut GlobalContext,
        domain: EncodedDomain,
    ) -> Result<Arc<CookieStoreId>, CustomError> {
        if let Some(container_match) = global_context.containers.match_container(domain.clone()) {
            return Ok(Arc::clone(container_match.container.handle()));
        }
        let domain = (*self == ContainerAssignStrategy::SuffixedTemporary).then_some(domain);
        new_temporary_container(global_context, domain).await
    }
}

/// Assigning strategy for tabs that are previously contained, including
/// a new tab that is a result of navigation from an existing tab.
/// - [IsolatedTemporary](ContainerEjectStrategy::IsolatedTemporary) means
///   that a new temporary container will be created for the tab specifically.
/// - [RemainInPlace](ContainerEjectStrategy::RemainInPlace) means that the tab
///   will remain in the container despite the incompatibility, useful for
///   referral links.
/// - [Reassignment](ContainerEjectStrategy::Reassignment) means that the tab
///   will be relocated as if it is a new uncontained tab, using a
///   [ContainerAssignStrategy].
#[derive(Clone, Derivative, Deserialize, Serialize)]
#[derivative(Default)]
#[serde(rename_all = "snake_case")]
pub enum ContainerEjectStrategy {
    #[derivative(Default)]
    IsolatedTemporary,
    RemainInPlace,
    Reassignment,
}

impl ContainerEjectStrategy {
    /// Matches a rejected tab's domain to a new container, regardless of type.
    /// Returns a container handle that must be properly released.
    /// Fails if the browser indicates so.
    #[must_use = "clean up must be done before releasing the handle"]
    pub async fn match_container(
        &self,
        global_context: &mut GlobalContext,
        domain: EncodedDomain,
        cookie_store_id: &CookieStoreId,
        assign_strategy: ContainerAssignStrategy,
    ) -> Result<Arc<CookieStoreId>, CustomError> {
        use ContainerEjectStrategy::*;
        match *self {
            IsolatedTemporary => new_temporary_container(global_context, None).await,
            RemainInPlace => Self::eject_remain_in_place(global_context, cookie_store_id).await,
            Reassignment => {
                assign_strategy
                    .match_container(global_context, domain)
                    .await
            }
        }
    }

    /// Remain in place eject strategy implementation.
    /// A new isolated temporary container is created
    /// if the container disappears before the handle is obtained.
    /// Fails if the browser indicates so.
    async fn eject_remain_in_place(
        global_context: &mut GlobalContext,
        cookie_store_id: &CookieStoreId,
    ) -> Result<Arc<CookieStoreId>, CustomError> {
        if let Some(container) = global_context.containers.get(cookie_store_id) {
            Ok(Arc::clone(container.handle()))
        } else {
            new_temporary_container(global_context, None).await
        }
    }
}

/// Creates a new temporary container,
/// does not check for an existing temporary container.
/// If a domain is supplied, its suffix will be appended.
/// the naming scheme may be changed in the future.
/// Fails if the browser indicates so.
async fn new_temporary_container(
    global_context: &mut GlobalContext,
    domain: Option<EncodedDomain>,
) -> Result<Arc<CookieStoreId>, CustomError> {
    let mut details = IdentityDetails {
        name: String::from("Temporary Container "),
        ..Default::default()
    };
    let mut suffixes = BTreeSet::default();
    if let Some(domain) = domain {
        let domain = global_context
            .psl
            .match_suffix(domain.clone())
            .unwrap_or(domain);
        details.name.push_str(domain.raw());
        suffixes.insert(Suffix::new(SuffixType::Normal, domain));
    }

    let container = Container::create(details, ContainerVariant::Temporary, suffixes).await?;
    let container_handle = Arc::clone(container.handle());
    storage::store_single_entry(&container_handle, &container).await?;
    global_context.containers.insert(container);
    Ok(container_handle)
}
