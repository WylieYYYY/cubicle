mod interop;
mod util;
mod view;

use std::panic;

use async_std::io::prelude::*;
use interop::{fetch::FetchReader, contextual_identities::ContextualIdentity};
use js_sys::{ArrayBuffer, JsString, Uint8Array};
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::interop::{contextual_identities::*, fetch, tabs};
use crate::util::message::Message;

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let tab_creator = Closure::new(tabs::create_tab);
    interop::add_runtime_listener("onInstalled", &tab_creator);
    tab_creator.forget();
    let mut buffer = [0; 100];
    FetchReader::try_from(fetch::get("https://wylieyyyy.gitlab.io")
        .await?.body().unwrap())?.read(&mut buffer).await.map_err(|e| JsString::from(e.to_string()))?;
    let a_buffer = Uint8Array::new(&ArrayBuffer::new(100));
    a_buffer.copy_from(&buffer);
    console::log_1(&a_buffer);
    let mut container = ContextualIdentity::create(IdentityDetails::default())
        .await.unwrap();
    let mut new_details = IdentityDetails::default();
    new_details.color = IdentityColor::Yellow;
    container.update(new_details).await.unwrap();
    container.delete().await.unwrap();
    Ok(())
}

#[wasm_bindgen(js_name="onMessage")]
pub async fn on_message(message: JsValue) -> Result<JsString, JsError> {
    let message = serde_wasm_bindgen::from_value::<Message>(message)
        .expect("unexpected message format");
    message.act().await.map(|html| JsString::from(html))
        .map_err(|error| JsError::new(&error.to_string()))
}
