//! Public suffix list, as described at
//! [publicsuffix.org](https://publicsuffix.org/).

use std::collections::BTreeSet;
use std::io::ErrorKind;

use async_std::io::prelude::*;
use chrono::naive::NaiveDate;
use serde::{Deserialize, Serialize};

use super::suffix::{self, MatchMode, Suffix, SuffixType};
use super::EncodedDomain;
use crate::util::errors::CustomError;

/// Public suffix list, used for checking if domains are controlled by
/// the same entity, and if containers should span across them.
#[derive(Default, Deserialize, Serialize)]
pub struct Psl {
    last_updated: NaiveDate,
    set: BTreeSet<Suffix>,
}

impl Psl {
    /// Reads and constructs a public suffix list from a stream.
    /// Comments and empty lines are ignored,
    /// comments must start from column 0.
    /// Fails with [CustomError::IoError] if the stream ends unexpectedly,
    /// or with [CustomError::InvalidSuffix].
    pub async fn from_stream<T>(
        stream: &mut T,
        last_updated: NaiveDate,
    ) -> Result<Self, CustomError>
    where
        T: BufRead + Unpin,
    {
        let mut set = BTreeSet::default();
        let mut buf = String::new();
        while let 1.. = stream
            .read_line(&mut buf)
            .await
            .map_err(|error| CustomError::IoError(error.kind()))?
        {
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
    pub fn match_suffix(&self, domain: EncodedDomain) -> Option<EncodedDomain> {
        suffix::match_suffix(&self.set, domain, MatchMode::Parent).find_map(|(domain, suffix)| {
            (*suffix.suffix_type() != SuffixType::Exclusion).then_some(domain)
        })
    }

    /// Returns `true` if the list contains no suffix.
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// The number of suffixes stored.
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Last updated date, time is not stored as
    /// this is only used for rate limiting.
    pub fn last_updated(&self) -> NaiveDate {
        self.last_updated
    }
}

#[cfg(test)]
mod test {
    use std::assert_eq;

    use async_std::io::Cursor;
    use chrono::Utc;
    use indoc::indoc;

    use super::*;
    use crate::util::test::TestFrom;

    #[async_std::test]
    async fn test_psl_from_stream() {
        let mut builtin_bytes =
            Cursor::new(std::include_bytes!("../../res/public_suffix_list.dat"));
        let last_updated = Utc::now().date_naive();
        let builtin_psl = Psl::from_stream(&mut builtin_bytes, last_updated)
            .await
            .expect("from_stream should read the builtin PSL with no error");
        assert_eq!(builtin_psl.len(), 9021);
        assert!(!builtin_psl.is_empty());
        assert_eq!(last_updated, builtin_psl.last_updated());
    }

    #[async_std::test]
    async fn test_psl_match_suffix() {
        let mut bytes = Cursor::new(
            indoc! {"
            com
            *.com
            !example.com
        "}
            .as_bytes(),
        );
        let psl = Psl::from_stream(&mut bytes, Utc::now().date_naive())
            .await
            .expect("controlled test");
        let table = [
            ("example.org", None),
            ("example.com", Some("example.com")),
            ("sub.example.com", Some("example.com")),
            ("testing.com", Some("testing.com")),
            ("sub.testing.com", Some("sub.testing.com")),
            ("com", None),
        ];
        for entry in table {
            let got = psl.match_suffix(EncodedDomain::tfrom(entry.0));
            assert_eq!(
                got.map(|got| String::from(got.raw())),
                entry.1.map(String::from)
            );
        }
    }
}
