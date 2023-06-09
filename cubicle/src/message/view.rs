use std::{iter, ops::DerefMut};

use chrono::offset::Utc;
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::Display;
use tera::{Context, Tera};

use crate::container::Container;
use crate::context::GlobalContext;
use crate::interop::{self, tabs};
use crate::interop::contextual_identities::{
    CookieStoreId, IdentityIcon, IdentityColor, IdentityDetailsProvider
};
use crate::util::errors::CustomError;

#[derive(Deserialize, Display)]
#[serde(rename_all="snake_case", tag="view")]
#[strum(serialize_all="kebab-case")]
pub enum View {
    NewContainer,
    Welcome,
    FetchAllContainers { selected: Option<CookieStoreId> },
    DeletePrompt { cookie_store_id: CookieStoreId },
    UpdateContainer { cookie_store_id: CookieStoreId },
    ContainerDetail { cookie_store_id: CookieStoreId },

    OptionsBody
}

impl View {
    pub async fn render(
        &self, global_context: &mut impl DerefMut<Target = GlobalContext>
    ) -> Result<String, CustomError> {
        use View::*;
        match self {
            NewContainer => {
                Ok(render_with(new_container(None), self).await)
            },
            Welcome => Ok(render_with(Context::default(), self).await),
            FetchAllContainers { selected } => {
                let selected = selected.clone()
                    .unwrap_or(tabs::current_tab_cookie_store_id().await?);
                fetch_all_containers(global_context, &selected).await
            },
            DeletePrompt { cookie_store_id } => {
                let container = global_context.containers.get(cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(delete_prompt(container), self).await)
            }
            UpdateContainer { cookie_store_id } => {
                let container = global_context.containers.get(cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(new_container(
                    Some(container)), &NewContainer).await)
            },
            ContainerDetail { cookie_store_id } => {
                let container = global_context.containers.get(cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(container_detail(container), self).await)
            },
            OptionsBody => {
                Ok(render_with(options_body(global_context), self).await)
            }
        }
    }
}

fn new_container(existing_container: Option<&Container>)
-> Context {
    let mut context = Context::new();
    context.insert("colors", &IdentityColor::iter()
        .collect::<Vec<IdentityColor>>());
    context.insert("icons", &IdentityIcon::iter()
        .map(|icon| (icon.clone(), icon.url()))
        .collect::<Vec<(IdentityIcon, String)>>());
    if let Some(container) = existing_container {
        context.insert("details", &container.identity_details());
    }
    context
}

async fn fetch_all_containers(
    global_context: &mut impl DerefMut<Target = GlobalContext>,
    selected: &CookieStoreId
) -> Result<String, CustomError> {
    let mut context = Context::new();
    context.insert("containers",
        &global_context.fetch_all_containers().await?);
    context.insert("selected", selected);
    Ok(Tera::default().render_str(r#"
        <option value="none">No Container</option>
        {% for container in containers %}
            <option value="{{container.0}}"
                {% if container.0 == selected %}selected=""{% endif %}>
                {{container.1.name}}
            </option>
        {% endfor %}
        <option value="new">+ Create New</option>
    "#, &context).expect("controlled enum template rendering"))
}

fn delete_prompt(container: &Container) -> Context {
    let mut context = Context::new();
    context.insert("name", &container.identity_details().name);
    context
}

fn container_detail(container: &Container) -> Context {
    let mut context = Context::new();
    context.insert("suffixes", &container.suffixes.iter().map(|suffix| {
            (suffix.raw(), suffix.encoded())
        }).chain(iter::once((String::new(), String::new())))
        .collect::<Vec<(String, String)>>());
    context
}

fn options_body(
    global_context: &mut impl DerefMut<Target = GlobalContext>
) -> Context {
    let mut context = Context::new();
    let last_updated = global_context.psl.last_updated();
    context.insert("psl_last_updated", &last_updated);
    let days_since_update = Utc::now().date_naive()
        .signed_duration_since(last_updated).num_days();
    context.insert("psl_no_update", &(days_since_update < 7));
    context
}

async fn render_with(context: Context, view: &View) -> String {
    Tera::default().render_str(&interop::fetch_extension_file(&format!(
        "components/{filename}.html", filename=view.to_string())).await,
        &context).expect("controlled enum template rendering")
}
