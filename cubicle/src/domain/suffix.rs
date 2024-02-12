//! Suffix that can appear in the public suffix list,
//! also used for allocating containers to domains.

use std::cmp::Ordering;
use std::{convert, iter, mem};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::EncodedDomain;
use crate::util::{errors::CustomError, KeyRangeExt};

/// Modes for matching suffixes from different sources,
/// as they have different expectations on the procedure.
/// - [Full](MatchMode::Full) means that the suffix should match the entirety
///   of the domain, useful for [Container](crate::container::Container).
/// - [Parent](MatchMode::Parent) means that the suffix should match the parent
///   of the domain, useful for [Psl](super::psl::Psl).
pub enum MatchMode {
    Full,
    Parent,
}

/// Looks through a binary tree based data structure of suffixes
/// to search for ones that match the domain or its ancestors.
/// Returns an iterator of tuples of the matched domains and suffixes.
pub fn match_suffix<'a, T>(
    set: &'a T,
    domain: EncodedDomain,
    mode: MatchMode,
) -> impl Iterator<Item = (EncodedDomain, Suffix)> + 'a
where
    T: KeyRangeExt<'a, Suffix> + 'a,
{
    let mut domain = Some(domain);
    let domain_iter = iter::repeat_with(move || {
        let parent = domain.as_ref().and_then(EncodedDomain::parent);
        mem::replace(&mut domain, parent)
    })
    .map_while(convert::identity);
    domain_iter.filter_map(move |domain| {
        let domain_or_parent = match mode {
            MatchMode::Full => domain.clone(),
            MatchMode::Parent => domain.parent()?,
        };
        match_suffix_exact(set, &domain_or_parent).map(|suffix| (domain, suffix))
    })
}

/// Looks through a binary tree based data structure of suffixes
/// to search for one that exactly matches the domain.
fn match_suffix_exact<'a, T>(set: &'a T, domain: &EncodedDomain) -> Option<Suffix>
where
    T: KeyRangeExt<'a, Suffix> + 'a,
{
    let end = Suffix::new(SuffixType::Normal, domain.clone());
    let start = if let Some(parent) = domain.parent() {
        Suffix::new(SuffixType::Glob, parent)
    } else {
        end.clone()
    };
    let mut search_range = set.key_range(start..=end);
    search_range
        .rfind(|suffix| suffix.match_ordering(domain).is_eq())
        .cloned()
}

/// Valid suffix that consists of a [SuffixType] and an [EncodedDomain].
/// This is okay as the bare glob `*` is handled separately.
/// The ordering is organized similarly as the
/// published suffix list for quick searching.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(expecting = "a suffix", into = "String", try_from = "String")]
pub struct Suffix {
    suffix_type: SuffixType,
    domain: EncodedDomain,
}

impl Suffix {
    /// Creates a suffix from its individual components.
    /// The instance is guarenteed to be well-formed.
    pub fn new(suffix_type: SuffixType, domain: EncodedDomain) -> Self {
        Self {
            suffix_type,
            domain,
        }
    }

    /// Check if this suffix matches the given domain.
    /// Returns an [Ordering] as it was used for hinting search direction,
    /// may be changed to return a boolean value later.
    pub fn match_ordering(&self, domain: &EncodedDomain) -> Ordering {
        let self_reversed = self.domain.reverse();
        let globbed: Box<dyn Iterator<Item = &str>> = {
            if self.suffix_type == SuffixType::Glob {
                Box::new(iter::once(
                    domain
                        .reverse()
                        .last()
                        .expect("string split has at least one element"),
                ))
            } else {
                Box::new(iter::empty::<&str>())
            }
        };
        domain.reverse().cmp(self_reversed.chain(globbed))
    }

    /// Encoded version of the suffix,
    /// safe to use for checking for suffix duplication.
    pub fn encoded(&self) -> String {
        format!("{}{}", self.suffix_type.prefix(), self.domain.encoded())
    }

    /// Unencoded version of the suffix.
    pub fn raw(&self) -> String {
        format!("{}{}", self.suffix_type.prefix(), self.domain.raw())
    }

    /// The type of the suffix, primarily to check if it is an
    /// [Exclusion](SuffixType::Exclusion).
    /// May be replaced by an `is_exclusion` function.
    pub fn suffix_type(&self) -> &SuffixType {
        &self.suffix_type
    }
}

impl From<Suffix> for String {
    fn from(value: Suffix) -> Self {
        String::from(value.suffix_type.prefix()) + value.domain.raw()
    }
}

impl TryFrom<String> for Suffix {
    type Error = CustomError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(&*value)
    }
}

impl TryFrom<&str> for Suffix {
    type Error = CustomError;

    /// Constructs a suffix from a string.
    /// Fails with [CustomError::InvalidSuffix] if it has a malformed prefix,
    /// or if the contained domain cannot be encoded as
    /// an international domain name.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for suffix_type in SuffixType::iter()
            .cycle()
            .skip(SuffixType::INDEX_AFTER_NORMAL)
        {
            if let Some(domain) = value.strip_prefix(suffix_type.prefix()) {
                return if domain.is_empty() || domain.split('.').any(|segment| segment.is_empty()) {
                    Err(CustomError::InvalidSuffix {
                        suffix: String::from(domain),
                    })
                } else {
                    Ok(Self {
                        suffix_type,
                        domain: EncodedDomain::try_from(domain)?,
                    })
                };
            }
        }
        unreachable!("empty prefix fallback for normal type");
    }
}

impl PartialOrd for Suffix {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Suffix {
    fn cmp(&self, other: &Self) -> Ordering {
        let tld_ordering = self.domain.tld().cmp(&other.domain.tld());
        let level_ordering = self
            .domain
            .reverse()
            .count()
            .cmp(&other.domain.reverse().count());
        let type_ordering = self.suffix_type.cmp(&other.suffix_type);
        let alpha_ordering = self.domain.reverse().cmp(other.domain.reverse());
        tld_ordering
            .then(level_ordering)
            .then(type_ordering)
            .then(alpha_ordering)
    }
}

/// Types for suffixes.
/// The ordering is the result of suffix not storing glob star
/// as a part of the domain.
#[derive(Clone, Deserialize, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum SuffixType {
    Exclusion,
    Normal,
    Glob,
}

impl SuffixType {
    /// Number of types to skip for better prefix matching.
    pub(self) const INDEX_AFTER_NORMAL: usize = 2;

    /// Textual representation of the type.
    /// To parse a suffix from a string, use [Suffix::try_from] instead.
    /// To create a suffix internally, use [Suffix::new] instead.
    pub(self) fn prefix(&self) -> &str {
        match self {
            SuffixType::Glob => "*.",
            SuffixType::Exclusion => "!",
            SuffixType::Normal => "",
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::collections::BTreeSet;

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;
    use crate::util::test::TestFrom;

    fn test_suffixes() -> [Suffix; 8] {
        [
            "*.com",
            "!example.com",
            // should not coexist with the exclusion rule,
            // but hypothetically should be ordered like this
            "example.com",
            "*.example.com",
            "!more.example.com",
            "*.net",
            "測試.net",
            "xn--w22ay72a.net",
        ]
        .map(Suffix::tfrom)
    }

    #[wasm_bindgen_test]
    fn test_encode_suffix() {
        assert_eq!("xn--g6w251d.net", Suffix::tfrom("測試.net").encoded());
    }

    #[wasm_bindgen_test]
    fn test_match_suffix() {
        let suffix_set = BTreeSet::from(test_suffixes());
        let table = [
            ("example.com", vec!["example.com"]),
            ("more.example.com", vec!["!more.example.com", "example.com"]),
            ("example.net", vec!["*.net"]),
            ("com", vec![]),
        ];
        for entry in table {
            assert!(
                match_suffix(&suffix_set, EncodedDomain::tfrom(entry.0), MatchMode::Full)
                    .map(|suffix_match| suffix_match.1.raw())
                    .eq(entry.1.clone())
            );
            let mut skipped_matches = entry.1.into_iter();
            skipped_matches.next();
            assert!(match_suffix(
                &suffix_set,
                EncodedDomain::tfrom(entry.0),
                MatchMode::Parent
            )
            .map(|suffix_match| suffix_match.1.raw())
            .eq(skipped_matches));
        }
    }

    #[wasm_bindgen_test]
    fn test_suffix_match_ordering() {
        let table = [
            (("*.com", "exmaple.com"), true),
            (("com", "exmaple.com"), false),
            (("!com", "example.com"), false),
            (("!example.com", "example.com"), true),
            (("*.example.com", "example.com"), false),
        ];
        for entry in table {
            assert!(
                Suffix::tfrom((entry.0).0)
                    .match_ordering(&EncodedDomain::tfrom((entry.0).1))
                    .is_eq()
                    == entry.1
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_suffix_try_from() {
        assert!(Suffix::try_from("*.com").is_ok());
        assert!(Suffix::try_from("*com").is_err());
        assert!(Suffix::try_from("com*").is_err());
        assert!(Suffix::try_from("!com").is_ok());
        assert!(Suffix::try_from("com!").is_err());
        assert!(Suffix::try_from("!.com").is_err());
        assert!(Suffix::try_from("a.com").is_ok());
        assert!(Suffix::try_from("a..com").is_err());
        assert!(Suffix::try_from(".com").is_err());
        assert!(Suffix::try_from("com.").is_err());
    }

    #[wasm_bindgen_test]
    fn suffix_sorting() {
        assert!(test_suffixes()
            .windows(2)
            .all(|window| window[0] <= window[1]));
    }
}
