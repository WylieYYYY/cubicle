use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{
    Display, EnumCount as EnumCountMacro, EnumIter, EnumString, FromRepr
};
use tera::{Context, Tera};

#[derive(Deserialize, Serialize)]
pub struct IdentityDetails {
    pub color: IdentityColor, pub icon: IdentityIcon, pub name: String
}

impl Default for IdentityDetails {
    fn default() -> Self {
        Self {
            color: IdentityColor::Cycle, icon: IdentityIcon::Circle,
            name: String::from("Cubicle")
        }
    }
}

pub trait IdentityDetailsProvider {
    fn identity_details(&self) -> IdentityDetails;
}

#[derive(
    Clone, Deserialize, Display, EnumCountMacro, EnumIter, EnumString, Eq,
    FromRepr, PartialEq, Serialize
)]
#[serde(rename_all="lowercase")]
#[strum(serialize_all="lowercase")]
pub enum IdentityColor {
    Blue, Turquoise, Green, Yellow, Orange,
    Red, Pink, Purple, Toolbar,
    #[strum(disabled)] Cycle,
    #[strum(disabled, default)] Unknown(String)
}

impl IdentityColor {
    pub fn new_rolling_color() -> Self {
        static COLOR_INDEX: AtomicUsize = AtomicUsize::new(0);
        let new_index = COLOR_INDEX.fetch_add(1,
            Ordering::Relaxed) % (Self::COUNT - 2);
        Self::from_repr(new_index)
            .expect("controlled representation input range")
    }
}

const ICON_URL_TEMPLATE: &str = "resource://usercontext-content/{{name}}.svg";

#[derive(Clone, Deserialize, Display, EnumIter, EnumString, Serialize)]
#[serde(rename_all="lowercase")]
#[strum(serialize_all="lowercase")]
pub enum IdentityIcon {
    Fingerprint, Briefcase, Dollar, Cart, Circle, Gift, Vacation,
    Food, Fruit, Pet, Tree, Chill, Fence,
    #[strum(disabled, default)] Unknown(String)
}

impl IdentityIcon {
    pub fn url(&self) -> String {
        let mut context = Context::new();
        context.insert("name", &self.to_string());
        Tera::one_off(&ICON_URL_TEMPLATE, &context, false)
            .expect("controlled enum template rendering")
    }
}
