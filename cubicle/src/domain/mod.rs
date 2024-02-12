//! Domain representation and matching,
//! core components of initial container designation.

pub mod psl;
pub mod suffix;

use std::cmp::Ordering;

use serde::{Deserialize, Deserializer, Serialize};

use crate::util::SingleStringVisitor;

/// Domain that can be encoded as an international domain name.
#[derive(Clone, Debug, Eq, Serialize)]
#[serde(transparent)]
pub struct EncodedDomain {
    #[serde(skip_serializing)]
    encoded: String,
    raw: String,
}

impl EncodedDomain {
    /// Encoded version of the domain,
    /// safe to use for checking for domain duplication.
    pub fn encoded(&self) -> &str {
        &self.encoded
    }

    /// Unencoded version of the domain.
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

impl EncodedDomain {
    /// The top level domain.
    /// Since segments are non-empty and the top level is a valid domain,
    /// it can be returned as an [EncodedDomain].
    pub fn tld(&self) -> Self {
        Self::try_from(
            self.encoded
                .split('.')
                .last()
                .expect("string split has at least one element"),
        )
        .expect("validity checked from existing instance")
    }

    /// Parent of this domain, [None] if this is a top level domain.
    pub fn parent(&self) -> Option<Self> {
        self.encoded.split_once('.').map(|parent| {
            Self::try_from(parent.1).expect("validity checked from existing instance")
        })
    }

    /// This domain in reverse domain name notation,
    /// for ordering and searching.
    pub fn reverse(&self) -> impl Iterator<Item = &str> {
        self.encoded.split('.').rev()
    }
}

impl<'de> Deserialize<'de> for EncodedDomain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, Unexpected};
        let raw_domain = deserializer.deserialize_string(SingleStringVisitor)?;
        Self::try_from(&*raw_domain).or(Err(Error::invalid_value(
            Unexpected::Str(&raw_domain),
            &"an encodable domain",
        )))
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
        let compat_value = idna::domain_to_ascii_strict(&format!("{}.example", value))?;
        let encoded = String::from(
            compat_value
                .strip_suffix(".example")
                .expect("suffix preserved from encoded domain"),
        );
        Ok(Self {
            encoded,
            raw: String::from(value),
        })
    }
}

impl PartialEq for EncodedDomain {
    fn eq(&self, other: &Self) -> bool {
        self.encoded == other.encoded
    }
}

impl PartialOrd for EncodedDomain {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for EncodedDomain {
    fn cmp(&self, other: &Self) -> Ordering {
        self.reverse().cmp(other.reverse())
    }
}

#[cfg(test)]
pub mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::util::test::TestFrom;

    #[wasm_bindgen_test]
    fn test_domain_tld() {
        assert_eq!(EncodedDomain::tfrom("example.com").tld().raw(), "com");
        assert_eq!(EncodedDomain::tfrom("com").tld().raw(), "com");
    }

    #[wasm_bindgen_test]
    fn test_domain_parent() {
        assert_eq!(
            EncodedDomain::tfrom("example.com").parent(),
            Some(EncodedDomain::tfrom("com"))
        );
        assert_eq!(EncodedDomain::tfrom("com").parent(), None);
    }

    #[wasm_bindgen_test]
    fn test_domain_try_from() {
        assert!(EncodedDomain::try_from("a.com").is_ok());
        assert!(EncodedDomain::try_from("測試.net").is_ok());
        assert!(EncodedDomain::try_from("a..com").is_err());
        assert!(EncodedDomain::try_from(".com").is_err());
        assert!(EncodedDomain::try_from("com.").is_err());
    }

    #[wasm_bindgen_test]
    fn test_domain_reverse() {
        assert!(EncodedDomain::tfrom("sub.example.com")
            .reverse()
            .eq(["com", "example", "sub"]));
    }

    #[wasm_bindgen_test]
    fn test_domain_eq() {
        assert_eq!(
            EncodedDomain::tfrom("example.net"),
            EncodedDomain::tfrom("example.net")
        );
        assert_eq!(
            EncodedDomain::tfrom("試驗.net"),
            EncodedDomain::tfrom("xn--w22ay72a.net")
        );
    }

    #[wasm_bindgen_test]
    fn test_domain_order() {
        let table = [
            "example.com",
            "sub.example.com",
            "example.net",
            "測試.net",
            "xn--w22ay72a.net",
        ]
        .map(EncodedDomain::tfrom);
        assert!(table.windows(2).all(|window| window[0] <= window[1]));
    }
}
