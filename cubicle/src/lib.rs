mod container;
mod domain;
mod interop;
mod util;
mod view;

use std::ops::Deref;
use std::panic;

use async_std::io::BufReader;
use async_std::sync::Mutex;
use chrono::NaiveDate;
use js_sys::JsString;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::domain::EncodedDomain;
use crate::domain::psl::Psl;
use crate::interop::{storage, tabs};
use crate::interop::fetch::{self, Fetch};
use crate::util::{message::Message, options::GlobalContext};

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let tab_creator = Closure::new(tabs::create_tab);
    interop::add_runtime_listener("onInstalled", &tab_creator);
    tab_creator.forget();
    let path = interop::prepend_extension_base_url("public_suffix_list.dat");
    let mut reader = BufReader::new(Fetch::try_from(
        fetch::get(&path).await.unwrap().body().unwrap()).unwrap());
    let mut global_context = GLOBAL_CONTEXT.lock().await;
    drop(global_context.fetch_all_containers().await);
    global_context.psl = Psl::from_stream(
        &mut reader, NaiveDate::MIN).await.unwrap();
    drop(storage::set_with_keys(global_context.deref()).await);
    GlobalContext::from_storage().await.unwrap();
    console::log_1(&serde_wasm_bindgen::to_value(
        &global_context.psl.last_updated()).unwrap());
    let exmaple_com = EncodedDomain::try_from("example.com").unwrap();
    console::log_1(&JsString::from(exmaple_com.encoded()));
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
