//! Wrappers around the `browser.contextualIdentities` API.
//! Most fails are represented by
//! [FailedContainerOperation](CustomError::FailedContainerOperation).

use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::hash::Hash;

pub use super::bits::identity_details::*;

use base64::prelude::*;
use js_sys::{Object, Promise};
#[cfg(test)]
use mockall::mock;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::interop;
use crate::util::{errors::CustomError, Base64Visitor, SingleStringVisitor};

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

/// Browser feature allowing the separation of sites' information
/// into different identities.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextualIdentity {
    #[serde(
        deserialize_with = "CookieStoreId::deserialize_inner",
        serialize_with = "CookieStoreId::serialize_inner"
    )]
    cookie_store_id: CookieStoreId,
    color: IdentityColor,
    _color_code: String,
    icon: IdentityIcon,
    _icon_url: String,
    name: String,
}

impl ContextualIdentity {
    /// Fetches all identities that are known to the browser,
    /// allows for resynchronization with the browser.
    /// Fails if the browser indicates so.
    pub async fn fetch_all() -> Result<Vec<Self>, CustomError> {
        let op_error = CustomError::FailedContainerOperation {
            verb: String::from("fetch all"),
        };
        super::cast_or_standard_mismatch(
            JsFuture::from(identity_query(JsValue::from(Object::default())))
                .await
                .or(Err(op_error))?,
        )
    }

    /// Creates an identity using the given details.
    /// Fails if the browser indicates so.
    pub async fn create(mut details: IdentityDetails) -> Result<Self, CustomError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let identity = JsFuture::from(identity_create(interop::to_jsvalue(&details)))
            .await
            .or(Err(CustomError::FailedContainerOperation {
                verb: String::from("create"),
            }))?;
        super::cast_or_standard_mismatch(identity)
    }

    /// Updates the identity and the details stored
    /// using the given [IdentityDetails].
    /// Fails if the browser indicates so.
    pub async fn update(&mut self, details: IdentityDetails) -> Result<(), CustomError> {
        *self = self.cookie_store_id.update_identity(details).await?;
        Ok(())
    }

    /// Gets the [CookieStoreId] of this identity.
    pub fn cookie_store_id(&self) -> &CookieStoreId {
        &self.cookie_store_id
    }
}

impl IdentityDetailsProvider for ContextualIdentity {
    fn identity_details(&self) -> IdentityDetails {
        IdentityDetails {
            color: self.color.clone(),
            icon: self.icon.clone(),
            name: self.name.clone(),
        }
    }
}

impl Debug for ContextualIdentity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_fmt(format_args!("contextual identity `{}`", self.name))
    }
}
impl Display for ContextualIdentity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        (self as &dyn Debug).fmt(formatter)
    }
}

/// Unique identifier that allow operations on specific identities.
/// By default, the serialzation is encoded. Otherwise, use
/// [CookieStoreId::deserialize_inner] or [CookieStoreId::serialize_inner].
/// All operations may fail if the identity specified by the ID does not exist.
#[derive(Clone, Eq, Hash, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct CookieStoreId {
    inner: String,
}

impl CookieStoreId {
    /// Creates a new ID by trusting the given value.
    /// May be removed as this is used for ad hoc tab request only
    /// in [super::tabs::current_tab_cookie_store_id].
    pub fn new(cookie_store_id: String) -> Self {
        Self {
            inner: cookie_store_id,
        }
    }

    /// Updates the [IdentityDetails] of the identity.
    /// Since this invalidates existing [ContextualIdentity],
    /// there is a helper [ContextualIdentity::update] for ensuring that
    /// the existing identity is updated.
    /// Fails if the browser indicates so.
    pub async fn update_identity(
        &self,
        mut details: IdentityDetails,
    ) -> Result<ContextualIdentity, CustomError> {
        if details.color == IdentityColor::Cycle {
            details.color = IdentityColor::new_rolling_color();
        }
        let error = CustomError::FailedContainerOperation {
            verb: String::from("update"),
        };
        let details = interop::to_jsvalue(&details);
        let identity = JsFuture::from(identity_update(&self.inner, details))
            .await
            .or(Err(error))?;
        super::cast_or_standard_mismatch(identity)
    }

    /// Deletes the identity.
    /// All [ContextualIdentity] will be invalidated,
    /// and the user is responsible for the cleanup.
    /// Fails if the browser indicates so.
    pub async fn delete_identity(&self) -> Result<(), CustomError> {
        let removal_result = JsFuture::from(identity_remove(&self.inner)).await;
        if removal_result.is_err() {
            Err(CustomError::FailedContainerOperation {
                verb: String::from("delete"),
            })
        } else {
            Ok(())
        }
    }

    /// Deserializes from a real unencoded value.
    pub fn deserialize_inner<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            inner: deserializer.deserialize_string(SingleStringVisitor)?,
        })
    }

    /// Serializes into the real unencoded value.
    pub fn serialize_inner<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

impl Default for CookieStoreId {
    /// The known [CookieStoreId] for the default [ContextualIdentity].
    /// Unused and may be removed as we don't care if the origin identity is
    /// the default, and we don't assign tabs to the default identity.
    fn default() -> Self {
        Self {
            inner: String::from("firefox-default"),
        }
    }
}

impl<'de> Deserialize<'de> for CookieStoreId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            inner: deserializer.deserialize_str(Base64Visitor)?,
        })
    }
}

impl Serialize for CookieStoreId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b64 = BASE64_URL_SAFE_NO_PAD.encode(&self.inner);
        serializer.serialize_str(&(String::from(Base64Visitor::MARKER_PREFIX) + &b64))
    }
}

#[cfg(test)]
mock! {
    pub ContextualIdentity {
        pub async fn fetch_all() -> Result<Vec<Self>, CustomError>;
        pub async fn create(mut details: IdentityDetails) -> Result<Self, CustomError>;
        pub async fn update(&mut self, details: IdentityDetails) -> Result<(), CustomError>;
        pub fn cookie_store_id(&self) -> &CookieStoreId;

        fn private_deserialize(deserializable: Result<ContextualIdentity, ()>) -> Self;
        fn private_serialize(&self) -> ContextualIdentity;
    }

    impl IdentityDetailsProvider for ContextualIdentity {
        fn identity_details(&self) -> IdentityDetails;
    }
}

#[cfg(test)]
impl<'de> Deserialize<'de> for MockContextualIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serializable = ContextualIdentity::deserialize(deserializer).map_err(|_| ());
        Ok(MockContextualIdentity::private_deserialize(serializable))
    }
}

#[cfg(test)]
impl Serialize for MockContextualIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.private_serialize().serialize(serializer)
    }
}
