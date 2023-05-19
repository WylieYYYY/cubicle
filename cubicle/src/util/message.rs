use std::ops::DerefMut;

use serde::{Deserialize, Serialize};

use crate::GlobalContext;
use crate::container::{Container, ContainerVariant};
use crate::interop::contextual_identities::{CookieStoreId, IdentityDetails};
use crate::util::errors::CustomError;
use crate::view::View;

#[derive(Deserialize, Serialize)]
#[serde(rename_all="snake_case", tag="message_type")]
pub enum Message {
    RequestPage { view: View },

    SubmitIdentityDetails {
        cookie_store_id: Option<CookieStoreId>,
        details: IdentityDetails
    },
    DeleteContainer { cookie_store_id: CookieStoreId }
}

impl Message {
    pub async fn act(
        self, global_context: &mut impl DerefMut<Target = GlobalContext>
    ) -> Result<String, CustomError> {
        use Message::*;
        match self {
            RequestPage { view } => view.render(global_context).await,

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
                        let cookie_store_id = container.cookie_store_id().clone();
                        global_context.containers.insert(container);
                        cookie_store_id
                    }
                };
                Ok(View::FetchAllContainers {
                    selected: Some(cookie_store_id)
                }.render(global_context).await?)
            },
            DeleteContainer { cookie_store_id } => {
                let container = global_context.containers
                    .remove(&cookie_store_id)
                    .expect("valid ID passed from message");
                if let Err(container) = container.delete().await {
                    global_context.containers.insert(container);
                }
                Ok(View::FetchAllContainers { selected: None }
                    .render(global_context).await?)
            }
        }
    }
}
