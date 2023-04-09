use strum::IntoEnumIterator;
use tera::{Context, Tera};

use crate::interop::contextual_identities::{IdentityIcon, IdentityColor};

pub fn new_container() -> String {
    let mut tera = Tera::default();
    let mut context = Context::new();
    context.insert("colors", &IdentityColor::iter()
        .collect::<Vec<IdentityColor>>());
    context.insert("icons", &IdentityIcon::iter()
        .map(|icon| (icon.clone(), icon.url()))
        .collect::<Vec<(IdentityIcon, String)>>());
    tera.render_str(include_str!("../../res/components/new-container.html"),
        &context).expect("controlled enum template rendering")
}