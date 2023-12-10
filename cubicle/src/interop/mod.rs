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

use js_sys::{JsString, Object, Promise, Reflect};
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
pub fn get_or_standard_mismatch(target: &Object, key: &str) -> Result<JsValue, CustomError> {
    let value = Reflect::get(target, &JsString::from(key)).expect("type checked to be object");
    if value.is_undefined() {
        Err(CustomError::StandardMismatch {
            message: format!("key `{}` is missing", key),
        })
    } else {
        Ok(value)
    }
}

/// Casts a [JsValue] into a [Deserialize] type.
/// Fails if they are not compatible.
pub fn cast_or_standard_mismatch<T>(target: JsValue) -> Result<T, CustomError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_wasm_bindgen::from_value(target).or(Err(CustomError::StandardMismatch {
        message: format!("`{}` expected", any::type_name::<T>()),
    }))
}

#[cfg(test)]
pub mod test {
    use std::collections::HashMap;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::util::test::TestFrom;

    use super::*;

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct TestStruct {
        attribute: bool,
    }

    #[wasm_bindgen_test]
    fn test_url_to_domain() {
        let example_com_domain = url_to_domain("https://example.com/index.html");
        assert!(example_com_domain.is_ok());
        assert_eq!(
            EncodedDomain::tfrom("example.com"),
            example_com_domain.expect("checked ok")
        );
        assert!(url_to_domain("gibberish").is_err());
    }

    #[wasm_bindgen_test]
    fn test_to_jsvalue() {
        let mut test_map = HashMap::new();
        test_map.insert("key", "value");

        let json_jsvalue = to_jsvalue(&test_map);
        assert_eq!(
            Ok(JsValue::UNDEFINED),
            Reflect::get(&json_jsvalue, &JsString::from("values"))
        );
        let map_jsvalue =
            serde_wasm_bindgen::to_value(&test_map).expect("serialization fail unlikely");
        let map_jsvalue_values = Reflect::get(&map_jsvalue, &JsString::from("values"));

        assert!(map_jsvalue_values.is_ok());
        assert!(map_jsvalue_values.expect("checked ok").is_function());
    }

    #[wasm_bindgen_test]
    fn test_get_or_standard_mismatch() {
        let known_object = Object::from(
            serde_wasm_bindgen::to_value(&TestStruct { attribute: true })
                .expect("known value serialization"),
        );
        let existing_attr = get_or_standard_mismatch(&known_object, "attribute");
        assert!(existing_attr.is_ok());
        assert_eq!(JsValue::TRUE, existing_attr.expect("checked ok"));
        assert!(get_or_standard_mismatch(&known_object, "dne").is_err());
    }

    #[wasm_bindgen_test]
    fn test_cast_or_standard_mismatch() {
        let empty_object = JsValue::from(Object::new());
        assert!(cast_or_standard_mismatch::<TestStruct>(empty_object).is_err());
        let test_jsvalue = serde_wasm_bindgen::to_value(&TestStruct { attribute: true })
            .expect("known value serialization");
        let converted = cast_or_standard_mismatch::<TestStruct>(test_jsvalue);
        assert!(converted.is_ok());
        assert_eq!(
            TestStruct { attribute: true },
            converted.expect("checked ok")
        );
    }
}
