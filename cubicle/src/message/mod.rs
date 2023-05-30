mod container;
mod view;

use std::ops::DerefMut;

use js_sys::{Object, Reflect};
use serde::Deserialize;

use self::container::ContainerAction;
use self::view::View;
use crate::GlobalContext;
use crate::interop::storage;
use crate::util::{self, errors::CustomError};

#[derive(Deserialize)]
#[serde(rename_all="snake_case", tag="message_type")]
pub enum Message {
    RequestPage { view: View },
    ContainerAction { action: ContainerAction }
}

impl Message {
    pub async fn act(
        self, global_context: &mut impl DerefMut<Target = GlobalContext>
    ) -> Result<String, CustomError> {
        use Message::*;
        match self {
            RequestPage { view } => view.render(global_context).await,
            ContainerAction { action } => {
                let cookie_store_id = action.act(global_context).await?;
                let existing_container = global_context.containers
                    .get(&cookie_store_id);
                let keys = Object::new();
                Reflect::set(&keys, &util::to_jsvalue(&cookie_store_id),
                    &util::to_jsvalue(&existing_container))
                    .expect("inline construction");
                storage::set_with_value_keys(&keys).await?;
                Ok(View::FetchAllContainers {
                    selected: existing_container.and(Some(cookie_store_id))
                }.render(global_context).await?)
            }
        }
    }
}
