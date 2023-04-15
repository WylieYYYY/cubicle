use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::iter;
use std::ops::{DerefMut, Deref};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::EncodedDomain;
use crate::interop::contextual_identities::ContextualIdentity;
use crate::util::errors::CustomError;

pub struct SuffixMap { tree: BTreeMap<Suffix, ContextualIdentity> }

impl SuffixMap {
    pub fn match_contextual_identity(&self, domain: &EncodedDomain)
    -> Option<&ContextualIdentity> {
        let start = Suffix::new(SuffixType::Exclusion, domain.tld());
        let end = Suffix::new(SuffixType::Normal, domain.clone());
        let search_range = self.tree.range(start..=end);
        search_range.fold(None, |acc, element| {
            if element.0.match_ordering(domain).is_eq() {
                Some(element.1)
            } else { acc }
        })
    }
}

impl Default for SuffixMap {
    fn default() -> Self {
        Self { tree: BTreeMap::default() }
    }
}

impl Deref for SuffixMap {
    type Target = BTreeMap<Suffix, ContextualIdentity>;

    fn deref(&self) -> &Self::Target { &self.tree }
}
impl DerefMut for SuffixMap {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.tree }
}

#[derive(Eq, PartialEq)]
pub struct Suffix { suffix_type: SuffixType, domain: EncodedDomain }

impl Suffix {
    pub fn match_ordering(&self, domain: &EncodedDomain) -> Ordering {
        let self_reversed = self.domain.reverse();
        let globbed: Box<dyn Iterator<Item = &str>> = {
            if self.suffix_type == SuffixType::Glob {
                Box::new(iter::once(domain.reverse().last()
                    .expect("string split has at least one element")))
            } else { Box::new(iter::empty::<&str>()) }
        };
        domain.reverse().cmp(self_reversed.chain(globbed))
    }
    pub(self) fn new(suffix_type: SuffixType, domain: EncodedDomain) -> Self {
        Self { suffix_type, domain }
    }
}

impl TryFrom<&str> for Suffix {
    type Error = CustomError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for suffix_type in SuffixType::iter().cycle()
            .skip(SuffixType::INDEX_AFTER_NORMAL) {
            if let Some(domain) = value.strip_prefix(suffix_type.prefix()) {
                return if domain.is_empty() || domain.split('.')
                    .find(|&segment| segment.is_empty()).is_some() {
                    Err(CustomError::InvalidSuffix {
                        suffix: String::from(domain)
                    })
                } else {
                    Ok(Self {
                        suffix_type, domain: EncodedDomain::try_from(domain)?
                    })
                };
            }
        }
        unreachable!("empty prefix fallback for normal type");
    }
}

impl PartialOrd for Suffix {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let tld_ordering = self.domain.tld().cmp(&other.domain.tld());
        let level_ordering = self.domain.reverse().count()
            .cmp(&other.domain.reverse().count());
        let type_ordering = self.suffix_type.cmp(&other.suffix_type);
        let alpha_ordering = self.domain.reverse().cmp(other.domain.reverse());
        Some(tld_ordering
            .then(level_ordering)
            .then(type_ordering)
            .then(alpha_ordering))
    }
}
impl Ord for Suffix {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("controlled PartialOrd implementation")
    }
}

#[derive(EnumIter, Eq, Ord, PartialEq, PartialOrd)]
pub enum SuffixType { Exclusion, Normal, Glob }

impl SuffixType {
    pub const INDEX_AFTER_NORMAL: usize = 2;

    pub fn prefix(&self) -> &str {
        match self {
            SuffixType::Glob => "*.",
            SuffixType::Exclusion => "!",
            SuffixType::Normal => ""
        }
    }
}
