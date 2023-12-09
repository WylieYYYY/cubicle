//! Information and structures used for
//! specifying the style of a contextual identity.

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use strum::EnumCount;
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString, FromRepr};
use tera::{Context, Tera};

/// Main styling structure for contextual identity,
/// check that [color](IdentityDetails::color) is not
/// [Cycle](IdentityColor::Cycle) before deserialization.
#[derive(Deserialize, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct IdentityDetails {
    pub color: IdentityColor,
    pub icon: IdentityIcon,
    pub name: String,
}

impl Default for IdentityDetails {
    /// Default styling for temporary containers.
    fn default() -> Self {
        Self {
            color: IdentityColor::Cycle,
            icon: IdentityIcon::Circle,
            name: String::from("Cubicle"),
        }
    }
}

/// Trait for getting an [IdentityDetails].
/// Currently used for getting styles as the fields of identities are private.
pub trait IdentityDetailsProvider {
    fn identity_details(&self) -> IdentityDetails;
}

/// Known supported color names, [Unknown](IdentityColor::Unknown) is for
/// potentially new colors in the future.
/// [Cycle](IdentityColor::Cycle) may be separated into its own enum in the
/// future to avoid incorrect deserialization.
#[derive(
    Clone,
    Deserialize,
    Display,
    EnumCountMacro,
    EnumIter,
    EnumString,
    Eq,
    FromRepr,
    PartialEq,
    Serialize,
)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum IdentityColor {
    Blue,
    Turquoise,
    Green,
    Yellow,
    Orange,
    Red,
    Pink,
    Purple,
    Toolbar,
    #[strum(disabled)]
    Cycle,
    #[strum(disabled, default)]
    Unknown(String),
}

impl IdentityColor {
    /// Gets a new color by rolling forward in the color cycle,
    /// the cycle is shared globally.
    pub fn new_rolling_color() -> Self {
        static COLOR_INDEX: AtomicUsize = AtomicUsize::new(0);
        let new_index = COLOR_INDEX.fetch_add(1, Ordering::Relaxed) % (Self::COUNT - 2);
        Self::from_repr(new_index).expect("controlled representation input range")
    }
}

/// Template for predicting where the icon images are,
/// necessary as the URL will only be provided once an identity is created.
const ICON_URL_TEMPLATE: &str = "resource://usercontext-content/{{name}}.svg";

/// Known supported icon names, [Unknown](IdentityIcon::Unknown) is for
/// potentially new icons in the future.
#[derive(Clone, Deserialize, Display, EnumIter, EnumString, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum IdentityIcon {
    Fingerprint,
    Briefcase,
    Dollar,
    Cart,
    Circle,
    Gift,
    Vacation,
    Food,
    Fruit,
    Pet,
    Tree,
    Chill,
    Fence,
    #[strum(disabled, default)]
    Unknown(String),
}

impl IdentityIcon {
    /// Gets the predicted URL of the icon.
    pub fn url(&self) -> String {
        let mut context = Context::new();
        context.insert("name", &self.to_string());
        Tera::one_off(ICON_URL_TEMPLATE, &context, false)
            .expect("controlled enum template rendering")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rolling_color() {
        let initial_color = IdentityColor::new_rolling_color();
        for _ in 1..(IdentityColor::COUNT - 2) {
            assert_ne!(initial_color, IdentityColor::new_rolling_color());
        }
        assert_eq!(initial_color, IdentityColor::new_rolling_color());
    }

    #[test]
    fn test_icon_url() {
        assert_eq!(
            "resource://usercontext-content/circle.svg",
            IdentityIcon::Circle.url()
        );
    }
}
