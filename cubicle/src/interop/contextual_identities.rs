use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

pub use super::bits::identity_details::*;

use base64::prelude::*;
use js_sys::{Promise, Object};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::util::{
    self, Base64Visitor, errors::CustomError, SingleStringVisitor
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="query")]
    fn identity_query(details: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="create")]
    fn identity_create(details: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="update")]
    fn identity_update(cookie_store_id: &str, detail: JsValue) -> Promise;
    #[wasm_bindgen(js_namespace=["browser", "contextualIdentities"], js_name="remove")]
    fn identity_remove(cookie_store_id: &str) -> Promise;
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub struct ContextualIdentity {
    #[serde(deserialize_with="deserialize_inner_id",
        serialize_with="serialize_inner_id")]
    cookie_store_id: CookieStoreId,
    color: IdentityColor, _color_code: String, icon: IdentityIcon,
    _icon_url: String, name: String
}

impl ContextualIdentity {
    pub async fn fetch_all() -> Result<Vec<Self>, CustomError> {
        let op_error = CustomError::FailedContainerOperation {
            verb: String::from("fetch all")
        };
        super::cast_or_standard_mismatch(
            JsFuture::from(identity_query(JsValue::from(Object::default())))
            .await.or(Err(op_error))?)
    }
    pub async fn create(mut details: IdentityDetails)
    -> Result<Self, CustomError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let identity = JsFuture::from(identity_create(
            util::to_jsvalue(&details))).await
            .or(Err(CustomError::FailedContainerOperation {
                verb: String::from("create")
            }))?;
        super::cast_or_standard_mismatch(identity)
    }
    pub async fn update(&mut self, details: IdentityDetails)
    -> Result<(), CustomError> {
        *self = self.cookie_store_id.update_identity(details).await?;
        Ok(())
    }

    pub fn cookie_store_id(&self) -> &CookieStoreId {
        &self.cookie_store_id
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

fn deserialize_inner_id<'de, D>(deserializer: D)
-> Result<CookieStoreId, D::Error>
where D: Deserializer<'de> {
    Ok(CookieStoreId {
        inner: deserializer.deserialize_string(SingleStringVisitor)?
    })
}

fn serialize_inner_id<S>(cookie_store_id: &CookieStoreId, serializer: S)
-> Result<S::Ok, S::Error>
where S: Serializer {
    serializer.serialize_str(&cookie_store_id.inner)
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CookieStoreId { inner: String }

impl CookieStoreId {
    pub fn new(cookie_store_id: String) -> Self {
        Self { inner: cookie_store_id }
    }
    pub async fn update_identity(&self, mut details: IdentityDetails)
    -> Result<ContextualIdentity, CustomError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let error = CustomError::FailedContainerOperation {
            verb: String::from("update")
        };
        let details = util::to_jsvalue(&details);
        let identity = JsFuture::from(identity_update(&self.inner, details))
            .await.or(Err(error))?;
        super::cast_or_standard_mismatch(identity)
    }
    pub async fn delete_identity(&self) -> Result<(), CustomError> {
        let removal_result = JsFuture::from(identity_remove(
            &self.inner)).await;
        if removal_result.is_err() {
            Err(CustomError::FailedContainerOperation {
                verb: String::from("delete")
            })
        } else { Ok(()) }
    }
}

impl<'de> Deserialize<'de> for CookieStoreId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        Ok(Self { inner: deserializer.deserialize_str(Base64Visitor)? })
    }
}

impl Serialize for CookieStoreId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let b64 = BASE64_URL_SAFE_NO_PAD.encode(&self.inner);
        serializer.serialize_str(&(String::from(
            Base64Visitor::MARKER_PREFIX) + &b64))
    }
}
