//! Message for content that can be rendered to a string.
use std::{iter, ops::DerefMut};

use chrono::offset::Utc;
use chrono::Duration;
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::Display;
use tera::{Context, Tera};

use crate::container::{Container, ContainerVariant};
use crate::context::GlobalContext;
use crate::interop::contextual_identities::{
    CookieStoreId, IdentityColor, IdentityDetails, IdentityDetailsProvider, IdentityIcon,
};
use crate::interop::{self, tabs};
use crate::util::errors::CustomError;

/// Message for content that can be rendered to a string,
/// kebab-case name of the view should be the start
/// of a template file name in `res/components`.
#[derive(Deserialize, Display)]
#[serde(rename_all = "snake_case", tag = "view")]
#[strum(serialize_all = "kebab-case")]
pub enum View {
    NewContainer,
    Welcome,
    Import,
    FetchAllContainers { selected: Option<CookieStoreId> },
    DeletePrompt { cookie_store_id: CookieStoreId },
    UpdateContainer { cookie_store_id: CookieStoreId },
    ContainerDetail { cookie_store_id: CookieStoreId },

    OptionsBody,
}

impl View {
    /// Renders the view, can return any string within the predefined format.
    /// Currently fails if the browser indicates so.
    /// Failure may be changed once the [fetch_all_containers]
    /// function is replaced.
    pub async fn render(
        &self,
        global_context: &mut impl DerefMut<Target = GlobalContext>,
    ) -> Result<String, CustomError> {
        use View::*;
        match self {
            NewContainer => Ok(render_with(new_container(None), self).await),
            Welcome | Import => Ok(render_with(Context::default(), self).await),
            FetchAllContainers { selected } => {
                let selected = selected
                    .clone()
                    .unwrap_or(tabs::current_tab_cookie_store_id().await?);
                fetch_all_containers(global_context, &selected).await
            }
            DeletePrompt { cookie_store_id } => {
                let container = global_context
                    .containers
                    .get(cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(delete_prompt(container), self).await)
            }
            UpdateContainer { cookie_store_id } => {
                let container = global_context
                    .containers
                    .get(cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(new_container(Some(container)), &NewContainer).await)
            }
            ContainerDetail { cookie_store_id } => {
                let container = global_context
                    .containers
                    .get(cookie_store_id)
                    .expect("valid ID passed from message");
                Ok(render_with(container_detail(container), self).await)
            }
            OptionsBody => Ok(render_with(options_body(global_context), self).await),
        }
    }
}

/// View for the customization of container styles when creating a new
/// container or updating an existing container.
/// This may be renamed later to be less misleading.
fn new_container(existing_container: Option<&Container>) -> Context {
    let mut context = Context::new();

    context.insert(
        "colors",
        &IdentityColor::iter().collect::<Vec<IdentityColor>>(),
    );
    context.insert(
        "icons",
        &IdentityIcon::iter()
            .map(|icon| (icon.clone(), icon.url()))
            .collect::<Vec<(IdentityIcon, String)>>(),
    );

    context.insert("update_existing", &existing_container.is_some());
    context.insert(
        "details",
        &match existing_container {
            Some(container) => container.identity_details(),
            None => IdentityDetails {
                name: String::new(),
                ..Default::default()
            },
        },
    );

    context
}

/// View for existing container list with additional action entries.
/// Returns a string of HTML fragment, which is an `option` element.
/// Fails if the browser indicates so.
async fn fetch_all_containers(
    global_context: &mut impl DerefMut<Target = GlobalContext>,
    selected: &CookieStoreId,
) -> Result<String, CustomError> {
    let mut context = Context::new();

    context.insert(
        "containers",
        &global_context
            .containers
            .iter()
            .filter_map(|container| {
                use ContainerVariant::*;
                match container.variant {
                    Permanent | Recording { .. } => Some((
                        container.handle().cookie_store_id().clone(),
                        container.identity_details(),
                    )),
                    Temporary => None,
                }
            })
            .collect::<Vec<(CookieStoreId, IdentityDetails)>>(),
    );

    context.insert("selected", selected);
    Ok(Tera::default()
        .render_str(
            r#"
        <option value="none">No Container</option>
        {% for container in containers %}
            <option value="{{container.0}}"
                {% if container.0 == selected %}selected=""{% endif %}>
                {{container.1.name}}
            </option>
        {% endfor %}
        <option value="new">+ Create New</option>
    "#,
            &context,
        )
        .expect("controlled enum template rendering"))
}

/// View for the deletion confirmation prompt.
fn delete_prompt(container: &Container) -> Context {
    let mut context = Context::new();
    context.insert("name", &container.identity_details().name);
    context
}

/// View for the body of the pop-up if a container is selected.
fn container_detail(container: &Container) -> Context {
    let mut context = Context::new();
    context.insert("icon_link", &container.identity_details().icon.url());
    context.insert("icon_color", &container.identity_details().color);
    context.insert(
        "is_recording",
        &matches!(container.variant, ContainerVariant::Recording { .. }),
    );
    context.insert(
        "suffixes",
        &container
            .suffixes
            .iter()
            .map(|suffix| (suffix.raw(), suffix.encoded()))
            .chain(iter::once((String::new(), String::new())))
            .collect::<Vec<(String, String)>>(),
    );
    context
}

/// View for the body of the preferences page.
/// May be rename to `preference_body` as the name has changed for that page.
fn options_body(global_context: &mut impl DerefMut<Target = GlobalContext>) -> Context {
    let mut context = Context::new();
    let last_updated = global_context.psl.last_updated();
    context.insert("psl_last_updated", &last_updated);
    let duration_since_update = Utc::now().date_naive().signed_duration_since(last_updated);
    context.insert(
        "psl_no_update",
        &(duration_since_update < Duration::weeks(1)),
    );
    context.insert(
        "assign_strategy",
        &global_context.preferences.assign_strategy,
    );
    context.insert("eject_strategy", &global_context.preferences.eject_strategy);
    context.insert(
        "should_revert_old_tab",
        &global_context.preferences.should_revert_old_tab,
    );
    context
}

/// Helper for rendering, since the templates are stored in the same directory,
/// and the fetching methods are the same.
/// Returns the rendered template as a string.
async fn render_with(context: Context, view: &View) -> String {
    Tera::default()
        .render_str(
            &interop::fetch_extension_file(&format!("components/{filename}.html", filename = view))
                .await,
            &context,
        )
        .expect("controlled enum template rendering")
}
