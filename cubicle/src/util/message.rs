use std::ops::DerefMut;

use serde::{Deserialize, Serialize};

use crate::GlobalContext;
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityDetails
};
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
                match cookie_store_id {
                    Some(cookie_store_id) => {
                        cookie_store_id.update_identity(details).await
                    },
                    None => ContextualIdentity::create(details).await
                }.and(Ok(String::new()))
            },
            DeleteContainer { cookie_store_id } => {
                cookie_store_id.delete_identity().await.and(Ok(String::new()))
            }
        }
    }
}
