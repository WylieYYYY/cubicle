use serde::{Deserialize, Serialize};

use crate::interop::contextual_identities::*;
use crate::util::errors::BrowserApiError;

#[derive(Deserialize, Serialize)]
pub struct Message { details: IdentityDetails }

impl Message {
    pub async fn act(self) -> Result<(), BrowserApiError> {
        Container::create(self.details).await?;
        Ok(())
    }
}