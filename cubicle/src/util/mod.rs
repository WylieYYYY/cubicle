use js_sys::{Reflect, JsString};
use wasm_bindgen::JsValue;

use self::errors::BrowserApiError;

pub mod errors;

pub fn usize_to_u32(value: usize) -> u32 {
    let maybe_truncated = value as u32;
    if value > maybe_truncated as usize { u32::MAX }
    else { maybe_truncated }
}

pub fn get_or_standard_mismatch(target: &JsValue, key: &str)
    -> Result<JsValue, BrowserApiError> {
    Reflect::get(target, &JsString::from(key))
        .or(Err(BrowserApiError::StandardMismatch {
        message: format!("key `{}` is missing", key)
    }))
}
