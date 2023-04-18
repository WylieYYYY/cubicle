use serde::{Deserialize, Serialize};

use crate::interop::contextual_identities::{
    IdentityDetails, ContextualIdentity
};
use crate::util::errors::CustomError;
use crate::view::View;

#[derive(Deserialize, Serialize)]
#[serde(rename_all="snake_case", tag="message_type")]
pub enum Message {
    RequestPage { view: View },

    SubmitIdentityDetails {
        cookie_store_id: Option<String>,
        details: IdentityDetails
    },
    DeleteContainer { cookie_store_id: String }
}

impl Message {
    pub async fn act(self) -> Result<String, CustomError> {
        use Message::*;
        match self {
            RequestPage { view } => view.render().await,
            SubmitIdentityDetails { cookie_store_id: _, details } => {
                ContextualIdentity::create(details).await
                    .and(Ok(String::new()))
            },
            DeleteContainer { cookie_store_id: _ } => todo!()
        }
    }
}
