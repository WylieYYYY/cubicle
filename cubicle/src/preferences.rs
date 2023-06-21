use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::container::ContainerOwner;
use crate::domain::EncodedDomain;
use crate::interop::contextual_identities::CookieStoreId;

#[derive(Default, Deserialize, Serialize)]
pub struct Preferences {
    pub assign_strategy: ContainerAssignStrategy,
    pub eject_strategy: ContainerEjectStrategy
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Default)]
pub enum ContainerAssignStrategy {
    #[derivative(Default)]
    SuffixedTemporary,
    IsolatedTemporary
}

impl ContainerAssignStrategy {
    pub fn match_container(&self, owner: &ContainerOwner,
        domain: EncodedDomain) -> CookieStoreId {
        owner.match_container(domain).map_or(CookieStoreId::default(),
            |container_match| {
                container_match.container.cookie_store_id().clone()
            })
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
