use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;
use tera::{Context, Tera};

use crate::interop;
use crate::interop::contextual_identities::{IdentityIcon, IdentityColor};

#[derive(Deserialize, Display, Serialize)]
#[serde(rename_all="snake_case")]
#[strum(serialize_all="kebab-case")]
pub enum View { NewContainer, Welcome }

impl View {
    pub async fn render(&self) -> String {
        use View::*;
        match self {
            NewContainer => render_with(new_container().await, self).await,
            Welcome => render_with(Context::default(), self).await
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

async fn render_with(context: Context, view: &View) -> String {
    Tera::default().render_str(&interop::fetch_extension_file(&format!(
        "components/{filename}.html", filename=view.to_string()))
        .await, &context).expect("controlled enum template rendering")
}
