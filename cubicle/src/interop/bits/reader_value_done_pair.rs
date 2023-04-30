use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::interop;
use crate::util::errors::CustomError;

pub fn buffer(this: &JsValue) -> Uint8Array {
    interop::get_or_standard_mismatch(this, "value").unwrap().dyn_into()
        .or(Err(CustomError::StandardMismatch {
            message: String::from("expected `value` to be an ArrayBuffer")
        })).unwrap()
}
