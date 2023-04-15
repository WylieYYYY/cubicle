use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ops::Deref;
use std::sync::{Mutex, Arc};

pub use super::bits::identity_details::*;

use js_sys::Promise;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::util::errors::CustomError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="create")]
    fn identity_create(details: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="update")]
    fn identity_update(cookie_store_id: &str, detail: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="remove")]
    fn identity_remove(cookie_store_id: &str) -> Promise;
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ContextualIdentity {
    cookie_store_id: CookieStoreId, color: IdentityColor, _color_code: String,
    icon: IdentityIcon, _icon_url: String, name: String
}

impl ContextualIdentity {
    pub async fn create(mut details: IdentityDetails)
    -> Result<Self, CustomError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let identity = JsFuture::from(identity_create(
            serde_wasm_bindgen::to_value( &details)
            .expect("serialization fail unlikely"))).await
            .or(Err(CustomError::FailedContainerCreation {
                name: details.name
            }))?;
        let error = CustomError::StandardMismatch {
            message: String::from("contextual identity expected")
        };
        Ok(serde_wasm_bindgen::from_value(identity).or(Err(error))?)
    }
    pub async fn update(&mut self, details: IdentityDetails)
    -> Result<(), CustomError> {
        *self = self.cookie_store_id.update_identity(details).await?;
        Ok(())
    }

    pub fn cookie_store_id(&self) -> CookieStoreId {
        self.cookie_store_id.clone()
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

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct CookieStoreId { inner: Arc<Mutex<String>> }

impl CookieStoreId {
    pub async fn update_identity(&mut self, mut details: IdentityDetails)
    -> Result<ContextualIdentity, CustomError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let error = CustomError::FailedContainerUpdate {
            name: details.name.clone()
        };
        let details = serde_wasm_bindgen::to_value(&details)
            .expect("serialization fail unlikely");
        let identity = JsFuture::from(identity_update(&self
            .try_lock("update")?, details)).await.or(Err(error))?;
        let error = CustomError::StandardMismatch {
            message: String::from("contextual identity expected")
        };
        Ok(serde_wasm_bindgen::from_value(identity).or(Err(error))?)
    }
    pub async fn delete_identity(self) -> Result<(), CustomError> {
        let removal_result = JsFuture::from(identity_remove(
            &self.try_lock("delete")?)).await;
        if removal_result.is_err() {
            Err(CustomError::FailedContainerDeletion)
        } else { Ok(()) }
    }

    pub(self) fn try_lock(&self, locker: &str)
    -> Result<impl Deref<Target = String> + '_, CustomError> {
        self.inner.try_lock().or(Err(CustomError::ContainerLocked {
            locker: String::from(locker)
        }))
    }
}
