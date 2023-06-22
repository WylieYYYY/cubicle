use std::cmp::Ordering;
use std::{convert, iter, mem};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::EncodedDomain;
use crate::util::{errors::CustomError, KeyRangeExt};

pub fn match_suffix<'a, T>(set: &'a T, domain: EncodedDomain)
-> impl Iterator<Item = (EncodedDomain, Suffix)> + 'a
where T: KeyRangeExt<'a, Suffix> + 'a {
    let mut domain = Some(domain);
    let domain_iter = iter::repeat_with(move || {
        let parent = domain.as_ref().and_then(EncodedDomain::parent);
        mem::replace(&mut domain, parent)
    }).map_while(convert::identity);
    domain_iter.filter_map(|domain| {
        match_suffix_exact(set, &domain.parent()?)
            .map(|suffix| (domain, suffix))
    })
}

fn match_suffix_exact<'a, T>(set: &'a T, domain: &EncodedDomain)
-> Option<Suffix>
where T: KeyRangeExt<'a, Suffix> + 'a {
    let end = Suffix::new(SuffixType::Normal, domain.clone());
    let start = if let Some(parent) = domain.parent() {
        Suffix::new(SuffixType::Glob, parent)
    } else { end.clone() };
    let mut search_range = set.key_range(start..=end);
    search_range.rfind(|suffix| suffix.match_ordering(domain).is_eq()).cloned()
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct Suffix { suffix_type: SuffixType, domain: EncodedDomain }

impl Suffix {
    pub fn new(suffix_type: SuffixType, domain: EncodedDomain) -> Self {
        Self { suffix_type, domain }
    }

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

    pub fn encoded(&self) -> String {
        format!("{}{}", self.suffix_type.prefix(), self.domain.encoded())
    }
    pub fn raw(&self) -> String {
        format!("{}{}", self.suffix_type.prefix(), self.domain.raw())
    }

    pub fn suffix_type(&self) -> &SuffixType { &self.suffix_type }
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

#[derive(
    Clone, Deserialize, EnumIter, Eq,
    Ord, PartialEq, PartialOrd, Serialize
)]
pub enum SuffixType { Exclusion, Normal, Glob }

impl SuffixType {
    pub(self) const INDEX_AFTER_NORMAL: usize = 2;

    pub(self) fn prefix(&self) -> &str {
        match self {
            SuffixType::Glob => "*.",
            SuffixType::Exclusion => "!",
            SuffixType::Normal => ""
        }
    }
}
