use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::util::{self, errors::BrowserApiError};

pub fn buffer(this: &JsValue) -> Uint8Array {
    util::get_or_standard_mismatch(this, "value").unwrap().dyn_into()
        .or(Err(BrowserApiError::StandardMismatch {
            message: String::from("expected `value` to be an ArrayBuffer")
        })).unwrap()
}
