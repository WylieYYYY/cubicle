//! Helper function for manipulating objects returned by a
//! [ReadableStreamByobReader](web_sys::ReadableStreamByobReader).

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::interop;
use crate::util::errors::CustomError;

/// Gets and casts the buffer returned.
/// This is implemented with [Result::unwrap] for now,
/// as the caller's error handling has not been finalized.
pub fn buffer(this: &JsValue) -> Uint8Array {
    interop::get_or_standard_mismatch(this, "value").unwrap().dyn_into()
        .or(Err(CustomError::StandardMismatch {
            message: String::from("expected `value` to be an ArrayBuffer")
        })).unwrap()
}
