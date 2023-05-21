pub mod psl;
pub mod suffix;

use std::cmp::Ordering;

use serde::Serialize;

#[derive(Clone, Eq, Serialize)]
pub struct EncodedDomain { encoded: String, raw: String }

impl EncodedDomain {
    pub fn encoded(&self) -> &str { &self.encoded }
    pub fn raw(&self) -> &str { &self.raw }
}

impl EncodedDomain {
    pub fn tld(&self) -> Self {
        Self::try_from(self.encoded.split('.').last()
            .expect("string split has at least one element"))
            .expect("validity checked from existing instance")
    }
    pub fn parent(&self) -> Option<Self> {
        self.encoded.split_once('.').map(|parent| Self::try_from(
            parent.1).expect("validity checked from existing instance"))
    }
    pub fn reverse(&self) -> impl Iterator<Item = &str> {
        self.encoded.split('.').rev()
    }
}

impl TryFrom<&str> for EncodedDomain {
    type Error = idna::Errors;

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
