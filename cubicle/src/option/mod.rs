use serde::{Deserialize, Serialize};

use crate::interop::contextual_identities::{IdentityDetails, Container};
use crate::util::errors::BrowserApiError;
use crate::view;

#[derive(Deserialize, Serialize)]
#[serde(rename_all="snake_case", tag="message_type")]
pub enum Message {
    RequestNewContainer,
    SubmitIdentityDetails {
        cookie_store_id: Option<String>,
        details: IdentityDetails
    },
    DeleteContainer { cookie_store_id: String }
}

impl Message {
    pub async fn act(self) -> Result<String, BrowserApiError> {
        use Message::*;
        match self {
            RequestNewContainer => Ok(view::new_container()),
            SubmitIdentityDetails { cookie_store_id: _, details } =>
                Container::create(details).await.and(Ok(String::new())),
            DeleteContainer { cookie_store_id: _ } => todo!()
        }
    }
}