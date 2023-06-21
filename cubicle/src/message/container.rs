use std::ops::DerefMut;

use serde::Deserialize;

use crate::container::{Container, ContainerVariant};
use crate::context::GlobalContext;
use crate::interop::contextual_identities::{CookieStoreId, IdentityDetails};
use crate::util::errors::CustomError;

#[derive(Deserialize)]
#[serde(rename_all="snake_case", tag="action")]
pub enum ContainerAction {
    SubmitIdentityDetails {
        cookie_store_id: Option<CookieStoreId>,
        details: IdentityDetails
    },
    DeleteContainer { cookie_store_id: CookieStoreId }
}

impl ContainerAction {
    pub async fn act(
        self, global_context: &mut impl DerefMut<Target = GlobalContext>
    ) -> Result<CookieStoreId, CustomError> {
        use ContainerAction::*;
        match self {
            SubmitIdentityDetails { cookie_store_id, details } => {
                let cookie_store_id = match cookie_store_id {
                    Some(cookie_store_id) => {
                        let container = global_context.containers
                            .get_mut(&cookie_store_id)
                            .expect("valid ID passed from message");
                        container.update(details).await?;
                        container.cookie_store_id().clone()
                    },
                    None => {
                        let container = Container::create(details,
                            ContainerVariant::Permanent).await?;
                        let cookie_store_id = container
                            .cookie_store_id().clone();
                        global_context.containers.insert(container);
                        cookie_store_id
                    }
                };
                Ok(cookie_store_id)
            },

            DeleteContainer { cookie_store_id } => {
                let container = global_context.containers
                    .remove(&cookie_store_id)
                    .expect("valid ID passed from message");
                if let Err(container) = container.delete().await {
                    global_context.containers.insert(container);
                }
                Ok(cookie_store_id)
            }
        }
    }
}
