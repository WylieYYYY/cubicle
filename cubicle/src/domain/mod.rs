pub mod suffix;

use std::cmp::Ordering;

#[derive(Clone, Eq)]
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
    pub fn reverse(&self) -> impl Iterator<Item = &str> {
        self.encoded.split('.').rev()
    }
}

impl TryFrom<&str> for EncodedDomain {
    type Error = idna::Errors;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            encoded: idna::domain_to_ascii_strict(value)?,
            raw: String::from(value)
        })
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
