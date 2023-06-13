mod container;
mod domain;
mod interop;
mod message;
mod util;

use std::panic;

use async_std::sync::Mutex;
use js_sys::JsString;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::domain::EncodedDomain;
use crate::interop::tabs::{TabId, TabProperties};
use crate::message::Message;
use crate::util::{errors::CustomError, options::GlobalContext};

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    *global_context = GlobalContext::from_storage().await.unwrap();
    if global_context.psl.len() == 0 {
        Message::PslUpdate { url: None }.act(&mut global_context).await.unwrap();
    }
    drop(global_context.fetch_all_containers().await);
    let exmaple_com = EncodedDomain::try_from("example.com").unwrap();
    console::log_1(&JsValue::from_f64(global_context.psl.match_suffix(
        exmaple_com).count() as f64));
    Ok(())
}

static GLOBAL_CONTEXT: Lazy<Mutex<GlobalContext>> = Lazy::new(||
    Mutex::new(GlobalContext::default()));

#[wasm_bindgen(js_name="onMessage")]
pub async fn on_message(message: JsValue) -> Result<JsString, JsError> {
    let message = serde_wasm_bindgen::from_value::<Message>(message)
        .expect("unexpected message format");
    message.act(&mut GLOBAL_CONTEXT.lock().await).await
        .map(|html| JsString::from(html))
        .map_err(|error| JsError::new(&error.to_string()))
}

#[wasm_bindgen(js_name="onTabUpdated")]
pub async fn on_tab_updated(tab_id: isize, tab_properties: JsValue)
-> Result<(), JsError> {
    {
        let tab_id = TabId::new(tab_id);

        tab_id.stop_loading().await;
        let tab_properties = interop::cast_or_standard_mismatch
            ::<TabProperties>(tab_properties)?;
        if true {
            tab_id.reload_tab().await
        } else {
            tab_properties.new_tab().await?;
            tab_id.close_tab().await
        }
    }.map_err(|error: CustomError| JsError::new(&error.to_string()))
}
