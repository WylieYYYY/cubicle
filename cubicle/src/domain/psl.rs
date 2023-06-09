//! Public suffix list, as described at
//! [publicsuffix.org](https://publicsuffix.org/).

use std::collections::BTreeSet;
use std::io::ErrorKind;

use async_std::io::prelude::*;
use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};

use super::EncodedDomain;
use super::suffix::{self, Suffix, SuffixType};
use crate::util::errors::CustomError;

/// Public suffix list, used for checking if domains are controlled by
/// the same entity, and if containers should span across them.
#[derive(Default, Deserialize, Serialize)]
pub struct Psl { last_updated: NaiveDate, set: BTreeSet<Suffix> }

impl Psl {
    /// Reads and constructs a public suffix list from a stream.
    /// Comments and empty lines are ignored,
    /// comments must start from column 0.
    /// Fails with [CustomError::IoError] if the stream ends unexpectedly,
    /// or with [CustomError::InvalidSuffix].
    pub async fn from_stream<T>(stream: &mut T, last_updated: NaiveDate)
    -> Result<Self, CustomError>
    where T: BufRead + Unpin {
        let mut set = BTreeSet::default();
        let mut buf = String::new();
        while let 1.. = stream.read_line(&mut buf).await
            .map_err(|error| CustomError::IoError(error.kind()))? {
            let Some(strip) = buf.strip_suffix('\n').map(String::from) else {
               return Err(CustomError::IoError(ErrorKind::OutOfMemory));
            };
            if !(strip.starts_with("//") || strip.is_empty()) {
                set.insert(Suffix::try_from(&*strip)?);
            }
            buf.clear();
        }
        Ok(Self { last_updated, set })
    }

    /// Matches the given domain with the stored suffixes.
    /// Returns a domain which is equal to the input, or is an ancestor of it.
    /// [None] if the list does not specify the condition for the domain.
    /// Domains that share the same can share cookies safely.
    pub fn match_suffix(&self, domain: EncodedDomain)
    -> Option<EncodedDomain> {
        suffix::match_suffix(&self.set, domain).find_map(|(domain, suffix)| {
            let is_exclusion = *suffix.suffix_type() == SuffixType::Exclusion;
            if is_exclusion { None } else { Some(domain) }
        })
    }

    /// The number of suffixes stored.
    pub fn len(&self) -> usize { self.set.len() }

    /// Last updated date, time is not stored as
    /// this is only used for rate limiting.
    pub fn last_updated(&self) -> NaiveDate { self.last_updated }
}
