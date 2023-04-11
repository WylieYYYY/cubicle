use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub use super::bits::identity_details::{
    IdentityColor, IdentityDetails, IdentityDetailsProvider, IdentityIcon
};

use js_sys::Promise;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::util::errors::BrowserApiError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"])]
    fn create(details: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"])]
    fn update(cookie_store_id: &str, detail: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"])]
    fn remove(cookie_store_id: &str) -> Promise;
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ContextualIdentity {
    cookie_store_id: String, color: IdentityColor, _color_code: String,
    icon: IdentityIcon, _icon_url: String, name: String
}

impl ContextualIdentity {
    pub async fn create(mut details: IdentityDetails)
    -> Result<Self, BrowserApiError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let identity = JsFuture::from(create(serde_wasm_bindgen::to_value(
            &details).expect("serialization fail unlikely"))).await
            .or(Err(BrowserApiError::FailedContainerCreation {
                name: details.name
            }))?;
        let error = BrowserApiError::StandardMismatch {
            message: String::from("contextual identity expected")
        };
        Ok(serde_wasm_bindgen::from_value(identity).or(Err(error))?)
    }
    pub async fn update(&mut self, mut details: IdentityDetails)
    -> Result<(), BrowserApiError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let details = serde_wasm_bindgen::to_value(&details)
            .expect("serialization fail unlikely");
        let error = BrowserApiError::FailedContainerUpdate {
            name: self.name.clone()
        };
        let identity = JsFuture::from(update(&self.cookie_store_id,
            details)).await.or(Err(error))?;
        let error = BrowserApiError::StandardMismatch {
            message: String::from("contextual identity expected")
        };
        *self = serde_wasm_bindgen::from_value(identity).or(Err(error))?;
        Ok(())
    }
    pub async fn delete(self) -> Result<(), BrowserApiError> {
        JsFuture::from(remove(&self.cookie_store_id)).await.or(Err(
            BrowserApiError::FailedContainerDeletion { container: self }))?;
        Ok(())
    }
}

impl IdentityDetailsProvider for ContextualIdentity {
    fn identity_details(&self) -> IdentityDetails {
        IdentityDetails {
            color: self.color.clone(), icon: self.icon.clone(),
            name: self.name.clone()
        }
    }
}

impl Debug for ContextualIdentity{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_fmt(format_args!(
            "contextual identity `{}`", self.name))
    }
}
impl Display for ContextualIdentity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        (self as &dyn Debug).fmt(formatter)
    }
}
