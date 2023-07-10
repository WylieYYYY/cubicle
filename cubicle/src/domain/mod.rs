//! Domain representation and matching,
//! core components of initial container designation.

pub mod psl;
pub mod suffix;

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

/// Domain that can be encoded as an international domain name.
#[derive(Clone, Deserialize, Eq, Serialize)]
pub struct EncodedDomain { encoded: String, raw: String }

impl EncodedDomain {
    /// Encoded version of the domain,
    /// safe to use for checking for domain duplication.
    pub fn encoded(&self) -> &str { &self.encoded }

    /// Unencoded version of the domain.
    pub fn raw(&self) -> &str { &self.raw }
}

impl EncodedDomain {
    /// The top level domain.
    /// Since segments are non-empty and the top level is a valid domain,
    /// it can be returned as an [EncodedDomain].
    pub fn tld(&self) -> Self {
        Self::try_from(self.encoded.split('.').last()
            .expect("string split has at least one element"))
            .expect("validity checked from existing instance")
    }

    /// Parent of this domain, [None] if this is a top level domain.
    pub fn parent(&self) -> Option<Self> {
        self.encoded.split_once('.').map(|parent| Self::try_from(
            parent.1).expect("validity checked from existing instance"))
    }

    /// This domain in reverse domain name notation,
    /// for ordering and searching.
    pub fn reverse(&self) -> impl Iterator<Item = &str> {
        self.encoded.split('.').rev()
    }
}

impl TryFrom<&str> for EncodedDomain {
    type Error = idna::Errors;

    /// Constructs a domain from a string,
    /// bare TLDs are accepted as domain for allowing all suffixes.
    /// Fails with [idna::Errors] if the string cannot be encoded as an
    /// international domain name.
    /// May be changed to [CustomError::InvalidDomain](crate::util::errors::CustomError::InvalidDomain)
    /// later.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let compat_value = idna::domain_to_ascii_strict(
            &format!("{}.example", value))?;
        let encoded = String::from(compat_value.strip_suffix(".example")
            .expect("suffix preserved from encoded domain"));
        Ok(Self { encoded, raw: String::from(value) })
    }
}

impl PartialEq for EncodedDomain {
    fn eq(&self, other: &Self) -> bool { self.encoded == other.encoded }
}

impl PartialOrd for EncodedDomain {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.reverse().cmp(other.reverse()))
    }
}
impl Ord for EncodedDomain {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("controlled PartialOrd implementation")
    }
}
