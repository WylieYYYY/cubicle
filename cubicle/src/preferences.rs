//! All preferences that are not container or storage item specific.

use std::collections::BTreeSet;
use std::sync::Arc;

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::container::{Container, ContainerVariant};
use crate::context::GlobalContext;
use crate::domain::EncodedDomain;
use crate::domain::suffix::{Suffix, SuffixType};
use crate::interop::contextual_identities::{CookieStoreId, IdentityDetails};
use crate::interop::storage;
use crate::util::errors::CustomError;

/// All preferences that are not container or storage item specific.
#[derive(Default, Deserialize, Serialize)]
pub struct Preferences {
    pub assign_strategy: ContainerAssignStrategy,
    pub eject_strategy: ContainerEjectStrategy
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
pub enum ContainerAssignStrategy {
    #[derivative(Default)]
    SuffixedTemporary,
    IsolatedTemporary
}

impl ContainerAssignStrategy {
    /// Matches a tab's domain to an accepting container, regardless of type.
    /// Returns a container handle that must be properly released.
    /// Fails if the browser indicates so.
    #[must_use = "clean up must be done before releasing the handle"]
    pub async fn match_container(&self, global_context: &mut GlobalContext,
        domain: EncodedDomain)
    -> Result<Arc<CookieStoreId>, CustomError> {
        if let Some(container_match) = global_context
            .containers.match_container(domain.clone()) {
            return Ok(Arc::clone(container_match.container.handle()));
        }

        let mut details = IdentityDetails::default();
        details.name = String::from("Temporary Container ");
        let mut suffixes = BTreeSet::default();
        if *self == ContainerAssignStrategy::SuffixedTemporary {
            let domain = global_context.psl.match_suffix(
                domain.clone()).unwrap_or(domain);
            details.name.push_str(domain.raw());
            suffixes.insert(Suffix::new(SuffixType::Normal, domain));
        }

        let container = Container::create(details,
            ContainerVariant::Temporary, suffixes).await?;
        let container_handle = Arc::clone(container.handle());
        storage::store_single_entry(&container_handle, &container).await?;
        global_context.containers.insert(container);
        Ok(container_handle)
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
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Default)]
pub enum ContainerEjectStrategy {
    #[derivative(Default)]
    IsolatedTemporary,
    RemainInPlace,
    Reassignment
}
