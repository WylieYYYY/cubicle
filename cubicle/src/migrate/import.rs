//! Import functions for migrating from vanilla containers,
//! or from other container providers.

use crate::container::{Container, ContainerOwner, ContainerVariant};
use crate::interop::contextual_identities::{ContextualIdentity, IdentityDetailsProvider};
use crate::util::errors::CustomError;

/// Fetches all [ContextualIdentity] and treat them as [Container],
/// detects temporary containers by name if needed.
/// Fails if the browser indicates so.
pub async fn fetch_all_containers(detect_temp: bool) -> Result<ContainerOwner, CustomError> {
    Ok(ContainerOwner::from_iter(
        ContextualIdentity::fetch_all()
            .await?
            .into_iter()
            .map(|identity| {
                let name = identity.identity_details().name;
                let mut container = Container::from(identity);
                if detect_temp && name.starts_with("Temporary Container ") {
                    container.variant = ContainerVariant::Temporary;
                }
                container
            }),
    ))
}
