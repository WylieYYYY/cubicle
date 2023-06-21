use std::collections::BTreeSet;
use std::io::ErrorKind;

use async_std::io::prelude::*;
use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};

use super::EncodedDomain;
use super::suffix::{self, Suffix, SuffixType};
use crate::util::errors::CustomError;

#[derive(Default, Deserialize, Serialize)]
pub struct Psl { last_updated: NaiveDate, set: BTreeSet<Suffix> }

impl Psl {
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

    pub fn match_suffix(&self, domain: EncodedDomain)
    -> Option<EncodedDomain> {
        suffix::match_suffix(&self.set, domain).find_map(|(domain, suffix)| {
            let is_exclusion = *suffix.suffix_type() == SuffixType::Exclusion;
            if is_exclusion { None } else { Some(domain) }
        })
    }

    pub fn len(&self) -> usize { self.set.len() }
    pub fn last_updated(&self) -> NaiveDate { self.last_updated }
}
