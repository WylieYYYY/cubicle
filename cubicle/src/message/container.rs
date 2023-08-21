//! Message type for container operations that are not tab related.

use std::collections::BTreeSet;
use std::ops::DerefMut;

use serde::Deserialize;

use crate::container::{Container, ContainerVariant};
use crate::context::GlobalContext;
use crate::domain::suffix::Suffix;
use crate::interop::contextual_identities::{CookieStoreId, IdentityDetails};
use crate::util::errors::CustomError;

/// Message type for container operations that are not tab related.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum ContainerAction {
    SubmitIdentityDetails {
        cookie_store_id: Option<CookieStoreId>,
        details: IdentityDetails,
    },
    UpdateSuffix {
        cookie_store_id: CookieStoreId,
        old_suffix: String,
        new_suffix: String,
    },
    DeleteContainer {
        cookie_store_id: CookieStoreId,
    },
}

impl ContainerAction {
    /// Performs the container operation,
    /// returns the [CookieStoreId] of the newly focused container.
    /// Fails if the browser indicates so.
    pub async fn act(
        self,
        global_context: &mut impl DerefMut<Target = GlobalContext>,
    ) -> Result<CookieStoreId, CustomError> {
        use ContainerAction::*;
        match self {
            SubmitIdentityDetails {
                cookie_store_id,
                details,
            } => {
                let cookie_store_id = match cookie_store_id {
                    Some(cookie_store_id) => {
                        let mut container = global_context
                            .containers
                            .get_mut(cookie_store_id.clone())
                            .expect("valid ID passed from message");
                        container.update(details).await?;
                        (**container.handle()).clone()
                    }
                    None => {
                        let container = Container::create(
                            details,
                            ContainerVariant::Permanent,
                            BTreeSet::default(),
                        )
                        .await?;
                        let cookie_store_id = (**container.handle()).clone();
                        global_context.containers.insert(container);
                        cookie_store_id
                    }
                };
                Ok(cookie_store_id)
            }

            UpdateSuffix {
                cookie_store_id,
                old_suffix,
                new_suffix,
            } => {
                let old_suffix = if old_suffix.is_empty() {
                    None
                } else {
                    Some(Suffix::try_from(&*old_suffix).expect("valid suffix passed from message"))
                };
                let new_suffix = if new_suffix.is_empty() {
                    None
                } else {
                    Some(Suffix::try_from(&*new_suffix)?)
                };
                let mut container = global_context
                    .containers
                    .get_mut(cookie_store_id.clone())
                    .expect("valid ID passed from message");
                if let Some(suffix) = old_suffix {
                    container.suffixes.remove(&suffix);
                }
                if let Some(suffix) = new_suffix {
                    container.suffixes.insert(suffix);
                }
                Ok(cookie_store_id)
            }

            DeleteContainer { cookie_store_id } => {
                let container = global_context
                    .containers
                    .get_mut(cookie_store_id.clone())
                    .expect("valid ID passed from message");
                container.delete().await?;
                drop(container);
                global_context.containers.remove(&cookie_store_id);
                Ok(cookie_store_id)
            }
        }
    }
}
