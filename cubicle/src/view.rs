use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;
use tera::{Context, Tera};

use crate::interop;
use crate::interop::contextual_identities::{
    IdentityIcon, IdentityColor, ContextualIdentity
};
use crate::util::errors::CustomError;

#[derive(Deserialize, Display, Serialize)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="kebab-case")]
pub enum View { NewContainer, Welcome, FetchAllContainers }

impl View {
    pub async fn render(&self) -> Result<String, CustomError> {
        use View::*;
        match self {
            NewContainer => Ok(render_with(new_container().await, self).await),
            Welcome => Ok(render_with(Context::default(), self).await),
            FetchAllContainers => fetch_all_containers().await
        }
    }
}

async fn new_container() -> Context {
    let mut context = Context::new();
    context.insert("colors", &IdentityColor::iter()
        .collect::<Vec<IdentityColor>>());
    context.insert("icons", &IdentityIcon::iter()
        .map(|icon| (icon.clone(), icon.url()))
        .collect::<Vec<(IdentityIcon, String)>>());
    context
}

async fn fetch_all_containers() -> Result<String, CustomError> {
    let mut context = Context::new();
    context.insert("containers", &ContextualIdentity::fetch_all().await?);
    Ok(Tera::default().render_str(r#"
        <option value="none">No Container</option>
        {% for container in containers %}
            <option value="{{loop.index}}">{{container.name}}</option>
        {% endfor %}
        <option value="new">+ Create New</option>
    "#, &context).expect("controlled enum template rendering"))
}

async fn render_with(context: Context, view: &View) -> String {
    Tera::default().render_str(&interop::fetch_extension_file(&format!(
        "components/{filename}.html", filename=view.to_string()))
        .await, &context).expect("controlled enum template rendering")
}
