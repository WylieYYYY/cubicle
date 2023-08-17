//! Generic reusable functions that do not rely on WebAssembly or project
//! specific resources.

pub mod errors;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Formatter, Result as FmtResult};
use std::iter::DoubleEndedIterator;
use std::ops::RangeBounds;

use base64::prelude::*;
use serde::de::Visitor;

/// Adapter for searching a key within a binary tree based data structures,
/// discarding values that are not keys.
pub trait KeyRangeExt<'a, K>
where
    K: Ord + 'a,
{
    /// Returns a [DoubleEndedIterator] of keys that are within the range.
    fn key_range<R>(&'a self, range: R) -> Box<dyn DoubleEndedIterator<Item = &'a K> + 'a>
    where
        R: RangeBounds<K>;
}

impl<'a, K> KeyRangeExt<'a, K> for BTreeSet<K>
where
    K: Ord + 'a,
{
    fn key_range<R>(&'a self, range: R) -> Box<dyn DoubleEndedIterator<Item = &'a K> + 'a>
    where
        R: RangeBounds<K>,
    {
        Box::new(self.range(range))
    }
}

impl<'a, K, V> KeyRangeExt<'a, K> for BTreeMap<K, V>
where
    K: Ord + 'a,
{
    fn key_range<R>(&'a self, range: R) -> Box<dyn DoubleEndedIterator<Item = &'a K> + 'a>
    where
        R: RangeBounds<K>,
    {
        Box::new(BTreeMap::range(self, range).map(|(k, _)| k))
    }
}

/// Deserialization visitor that decodes a string with no padding base 64,
/// and remove the prepending [MARKER_PREFIX](Base64Visitor::MARKER_PREFIX)
/// from the string.
pub struct Base64Visitor;

impl Base64Visitor {
    /// Marker that was prepended to the base 64 value,
    /// mainly for prompting external consumers.
    pub const MARKER_PREFIX: &str = "b64_";
}

impl Visitor<'_> for Base64Visitor {
    type Value = String;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        write!(
            formatter,
            "a base-64 encoded UTF-8 string prefixed with `{}`",
            Self::MARKER_PREFIX
        )
    }

    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use serde::de::{Error, Unexpected};
        use std::str::from_utf8;
        let decode_error = Error::invalid_value(Unexpected::Str(string), &self);
        let Some(string) = string.strip_prefix(Self::MARKER_PREFIX) else {
            return Err(decode_error);
        };
        if let Ok(b64) = BASE64_URL_SAFE_NO_PAD.decode(string) {
            Ok(String::from(from_utf8(&b64).or(Err(decode_error))?))
        } else {
            Err(decode_error)
        }
    }
}

/// Deserialization visitor that accepts a single string.
pub struct SingleStringVisitor;

impl Visitor<'_> for SingleStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("a string")
    }

    fn visit_string<E>(self, string: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(string)
    }
}
