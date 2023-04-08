use serde::{Deserialize, Serialize};

use crate::util::errors::BrowserApiError;
use crate::view;

#[derive(Deserialize, Serialize)]
#[serde(rename_all="snake_case", tag="message_type")]
pub enum Message {
    RequestNewContainer,
    DeleteContainer { cookie_store_id: String }
}

impl Message {
    pub async fn act(self) -> Result<String, BrowserApiError> {
        use Message::*;
        match self {
            RequestNewContainer => Ok(view::new_container()),
            DeleteContainer { cookie_store_id: _ } => todo!()
        }
    }
}