mod interop;
mod util;

extern crate console_error_panic_hook;

use std::panic;

use async_std::io::prelude::*;
use interop::{fetch::FetchReader, contextual_identities::Container};
use js_sys::{ArrayBuffer, Uint8Array, JsString};
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::interop::{contextual_identities::*, fetch, tabs};

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let tab_creator = Closure::new(tabs::create_tab);
    interop::addRuntimeListener("onInstalled", &tab_creator);
    tab_creator.forget();
    let mut buffer = [0; 100];
    FetchReader::try_from(fetch::get("https://wylieyyyy.gitlab.io")
        .await?.body().unwrap())?.read(&mut buffer).await.map_err(|e| JsString::from(e.to_string()))?;
    let a_buffer = Uint8Array::new(&ArrayBuffer::new(100));
    a_buffer.copy_from(&buffer);
    console::log_1(&a_buffer);
    let mut container = Container::create(IdentityDetails::default())
        .await.unwrap();
    let mut new_details = IdentityDetails::default();
    new_details.color = IdentityColor::Yellow;
    container.update(new_details).await.unwrap();
    container.delete().await.unwrap();
    Ok(())
}
