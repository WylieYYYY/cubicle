use std::collections::BTreeSet;

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::container::{Container, ContainerVariant};
use crate::context::GlobalContext;
use crate::domain::EncodedDomain;
use crate::domain::suffix::{Suffix, SuffixType};
use crate::interop::contextual_identities::{CookieStoreId, IdentityDetails};
use crate::interop::storage;
use crate::util::errors::CustomError;

#[derive(Default, Deserialize, Serialize)]
pub struct Preferences {
    pub assign_strategy: ContainerAssignStrategy,
    pub eject_strategy: ContainerEjectStrategy
}

#[derive(Clone, Derivative, Deserialize, Eq, PartialEq, Serialize)]
#[derivative(Default)]
pub enum ContainerAssignStrategy {
    #[derivative(Default)]
    SuffixedTemporary,
    IsolatedTemporary
}

impl ContainerAssignStrategy {
    pub async fn match_container(&self, global_context: &mut GlobalContext,
        domain: EncodedDomain)
    -> Result<CookieStoreId, CustomError> {
        if let Some(container_match) = global_context
            .containers.match_container(domain.clone()) {
            return Ok(container_match.container.cookie_store_id().clone());
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
            ContainerVariant::Temporary { tab_count: 1 }, suffixes).await?;
        let cookie_store_id = container.cookie_store_id().clone();
        storage::store_single_entry(&cookie_store_id, &container).await?;
        global_context.containers.insert(container);
        Ok(cookie_store_id)
    }
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Default)]
pub enum ContainerEjectStrategy {
    #[derivative(Default)]
    IsolatedTemporary,
    RemainInPlace,
    Reassignment
}
