use strum::IntoEnumIterator;
use tera::{Context, Tera};

use crate::interop::contextual_identities::IdentityIcon;

pub fn new_container() -> String {
    let mut tera = Tera::default();
    let mut context = Context::new();
    context.insert("icons", &IdentityIcon::iter()
        .map(|icon| (icon.clone(), icon.url()))
        .collect::<Vec<(IdentityIcon, String)>>());
    tera.render_str(include_str!("../../res/components/icon-grid.html"),
        &context).expect("controlled enum template rendering")
}