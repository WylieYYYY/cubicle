use std::fmt::{Formatter, Result as FmtResult};

use base64::prelude::*;
use js_sys::{Reflect, JsString};
use serde::de::Visitor;
use wasm_bindgen::JsValue;

use self::errors::CustomError;

pub mod errors;
pub mod message;

pub fn usize_to_u32(value: usize) -> u32 {
    let maybe_truncated = value as u32;
    if value > maybe_truncated as usize { u32::MAX }
    else { maybe_truncated }
}

pub fn get_or_standard_mismatch(target: &JsValue, key: &str)
    -> Result<JsValue, CustomError> {
    Reflect::get(target, &JsString::from(key))
        .or(Err(CustomError::StandardMismatch {
        message: format!("key `{}` is missing", key)
    }))
}

pub struct Base64Visitor;

impl Base64Visitor {
    pub const MARKER_PREFIX: &str = "b64_";
}

impl Visitor<'_> for Base64Visitor {
    type Value = String;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str(
            "a base-64 encoded UTF-8 string prefixed with `b64_`"
        )
    }

    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
    where E: serde::de::Error {
        use serde::de::{Error, Unexpected};
        use std::str::from_utf8;
        let decode_error = Error::invalid_value(
            Unexpected::Str(string), &self);
        let Some(string) = string.strip_prefix(Self::MARKER_PREFIX) else {
            return Err(decode_error);
        };
        if let Ok(b64) = BASE64_URL_SAFE_NO_PAD.decode(string) {
            Ok(String::from(from_utf8(&b64).or(Err(decode_error))?))
        } else { Err(decode_error) }
    }
}

pub struct SingleStringVisitor;

impl Visitor<'_> for SingleStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, string: String) -> Result<Self::Value, E>
    where E: serde::de::Error, {
        Ok(string)
    }
}
