mod container;
mod view;

use std::ops::DerefMut;

use async_std::io::BufReader;
use chrono::Utc;
use serde::Deserialize;

use self::container::ContainerAction;
use self::view::View;
use crate::context::GlobalContext;
use crate::domain::psl::Psl;
use crate::interop::{self, fetch::Fetch, storage};
use crate::util::errors::CustomError;

#[derive(Deserialize)]
#[serde(rename_all="snake_case", tag="message_type")]
pub enum Message {
    RequestPage { view: View },
    ContainerAction { action: ContainerAction },
    PslUpdate { url: Option<String> }
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
                storage::store_single_entry(&cookie_store_id,
                    &existing_container).await?;
                Ok(View::FetchAllContainers {
                    selected: existing_container.and(Some(cookie_store_id))
                }.render(global_context).await?)
            },
            PslUpdate { url } => {
                let local_path = interop::prepend_extension_base_url(
                    "public_suffix_list.dat");
                let mut reader = BufReader::new(
                    Fetch::get_stream(&url.unwrap_or(local_path)).await?);
                let new_date = Utc::now().date_naive();
                global_context.psl = Psl::from_stream(
                    &mut reader, new_date).await.unwrap();
                storage::store_single_entry("psl",
                    &global_context.psl).await?;
                Ok(new_date.to_string())
            }
        }
    }
}
