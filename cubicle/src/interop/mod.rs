//! Components that interact with the browser and Javascript,
//! all types are only minimally wrapped and type-casted.
//! Operations can fail with [StandardMismatch](CustomError::StandardMismatch)
//! if it uses an external API and the API returned an unexpected value.

mod bits;
pub mod contextual_identities;
pub mod fetch;
pub mod storage;
pub mod tabs;

use std::any;

use js_sys::{JsString, Promise, Reflect};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Url;

use crate::domain::EncodedDomain;
use crate::util::errors::CustomError;

#[wasm_bindgen(raw_module = "./background.js")]
extern "C" {
    /// Add a runtime listener.
    /// The closure should be leaked using [Closure::forget] later.
    #[wasm_bindgen(js_name = "addRuntimeListener")]
    pub fn add_runtime_listener(event: &str, handler: &Closure<dyn Fn(Box<[JsValue]>) -> Promise>);
}

#[wasm_bindgen]
extern "C" {
    /// Prepends a relative path with extension's domain.
    #[wasm_bindgen(js_namespace=["browser", "runtime"], js_name="getURL")]
    pub fn prepend_extension_base_url(path: &str) -> String;
}

/// Fetches a file owned by the extension as a UTF-8 encoded string.
/// Large file should be fetched using [Fetch](crate::interop::fetch::Fetch)
/// instead.
pub async fn fetch_extension_file(path: &str) -> String {
    JsFuture::from(
        fetch::get(&prepend_extension_base_url(path))
            .await
            .expect("valid and stable connection")
            .text()
            .expect("standard does not define synchronous errors"),
    )
    .await
    .expect("assume consume body successful")
    .as_string()
    .expect("body must be a valid string")
}

/// Converts a URL to [EncodedDomain] using Javascript's [Url] API.
/// Fails if the URL is not valid.
pub fn url_to_domain(url: &str) -> Result<EncodedDomain, CustomError> {
    let hostname = Url::new(url)
        .or(Err(CustomError::StandardMismatch {
            message: String::from("url should be validated"),
        }))?
        .hostname();
    EncodedDomain::try_from(&*hostname).or(Err(CustomError::StandardMismatch {
        message: String::from("domain should be validated"),
    }))
}

/// Serializes a [Serialize] type to a [JsValue]
/// using a JSON compatible serializer.
pub fn to_jsvalue<T>(value: &T) -> JsValue
where
    T: Serialize + ?Sized,
{
    value
        .serialize(&Serializer::json_compatible())
        .expect("serialization fail unlikely")
}

/// Gets a value within a [JsValue] using a string key via reflection.
/// Fails if no such key was found.
pub fn get_or_standard_mismatch(target: &JsValue, key: &str) -> Result<JsValue, CustomError> {
    Reflect::get(target, &JsString::from(key)).or(Err(CustomError::StandardMismatch {
        message: format!("key `{}` is missing", key),
    }))
}

/// Casts a [JsValue] into a [Deserialize] type.
/// Fails if they are not compatible.
pub fn cast_or_standard_mismatch<T>(target: JsValue) -> Result<T, CustomError>
where
    T: for<'de> Deserialize<'de>,
{
    Ok(
        serde_wasm_bindgen::from_value(target).or(Err(CustomError::StandardMismatch {
            message: format!("`{}` expected", any::type_name::<T>()),
        }))?,
    )
}
