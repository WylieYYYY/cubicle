use std::ops::DerefMut;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;
use tera::{Context, Tera};

use crate::{interop, GlobalContext};
use crate::interop::contextual_identities::{
    ContextualIdentity, CookieStoreId, IdentityIcon, IdentityColor
};
use crate::interop::tabs;
use crate::util::errors::CustomError;

#[derive(Deserialize, Display, Serialize)]
#[serde(rename_all="snake_case", tag="view")]
#[strum(serialize_all="kebab-case")]
pub enum View {
    NewContainer,
    Welcome,
    FetchAllContainers { selected: Option<CookieStoreId> },
    DeletePrompt { cookie_store_id: CookieStoreId },
    UpdateContainer { cookie_store_id: CookieStoreId }
}

impl View {
    pub async fn render(
        &self, global_context: &mut impl DerefMut<Target = GlobalContext>
    ) -> Result<String, CustomError> {
        use View::*;
        match self {
            NewContainer => {
                Ok(render_with(new_container(None).await, self).await)
            },
            Welcome => Ok(render_with(Context::default(), self).await),
            FetchAllContainers { selected } => {
                let selected = selected.clone()
                    .unwrap_or(tabs::current_tab_cookie_store_id().await?);
                fetch_all_containers(global_context, &selected).await
            },
            DeletePrompt { cookie_store_id } => {
                let identity = global_context.containers.get(&cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(delete_prompt(identity).await, self).await)
            }
            UpdateContainer { cookie_store_id } => {
                let identity = global_context.containers.get(&cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(new_container(
                    Some(identity)).await, &NewContainer).await)
            }
        }
    }
}

async fn new_container(existing_identity: Option<&ContextualIdentity>)
-> Context {
    let mut context = Context::new();
    context.insert("colors", &IdentityColor::iter()
        .collect::<Vec<IdentityColor>>());
    context.insert("icons", &IdentityIcon::iter()
        .map(|icon| (icon.clone(), icon.url()))
        .collect::<Vec<(IdentityIcon, String)>>());
    if let Some(identity) = existing_identity {
        context.insert("identity", identity);
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

async fn delete_prompt(identity: &ContextualIdentity) -> Context {
    let mut context = Context::new();
    context.insert("identity", identity);
    context
}

async fn render_with(context: Context, view: &View) -> String {
    Tera::default().render_str(&interop::fetch_extension_file(&format!(
        "components/{filename}.html", filename=view.to_string())).await,
        &context).expect("controlled enum template rendering")
}
