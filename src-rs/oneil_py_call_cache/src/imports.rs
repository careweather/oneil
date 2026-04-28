//! Cache records for imported Python modules ([`ImportEntry`], [`ImportHash`]).

use std::collections::BTreeSet;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::CachedFunctionName;

/// One imported module record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportEntry {
    /// Logical module name.
    pub name: String,
    /// Local dependency paths included in the combined hash.
    pub dependencies: BTreeSet<String>,
    /// Hash of this module's source plus dependency sources.
    pub hash: ImportHash,
    /// Functions from this module that were invoked.
    pub functions_used: BTreeSet<CachedFunctionName>,
}

impl ImportEntry {
    /// Creates a new import entry.
    #[must_use]
    pub const fn new(name: String, dependencies: BTreeSet<String>, hash: ImportHash) -> Self {
        Self {
            name,
            dependencies,
            hash,
            functions_used: BTreeSet::new(),
        }
    }
}

/// Fingerprint for an imported module's sources (stored as raw `u64`, serialized as hex).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportHash(u64);

impl Serialize for ImportHash {
    /// Writes this hash as a 16-digit lowercase hexadecimal string (JSON string).
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:016x}", self.0))
    }
}

impl<'de> Deserialize<'de> for ImportHash {
    /// Parses a base-16 string into a hash (no `0x` prefix).
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.is_empty() {
            return Err(de::Error::custom("empty hexadecimal string"));
        }

        u64::from_str_radix(&s, 16)
            .map(ImportHash)
            .map_err(de::Error::custom)
    }
}

impl PartialEq<u64> for ImportHash {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<ImportHash> for u64 {
    fn eq(&self, other: &ImportHash) -> bool {
        *self == other.0
    }
}

impl From<u64> for ImportHash {
    fn from(hash: u64) -> Self {
        Self(hash)
    }
}

impl From<ImportHash> for u64 {
    fn from(hash: ImportHash) -> Self {
        hash.0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ImportEntry, ImportHash};
    use serde_json::json;

    #[test]
    fn serde_roundtrip_hex_string() {
        let h = ImportHash(0xdead_beef_cafe_babe);
        let j = serde_json::to_string(&h).expect("serialize");
        assert_eq!(j, "\"deadbeefcafebabe\"");
        let back: ImportHash = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, h);
    }

    #[test]
    fn deserializes_zero_padded_and_unpadded_hex() {
        assert_eq!(
            serde_json::from_str::<ImportHash>("\"0000000000000001\"")
                .expect("deserialize padded hex"),
            ImportHash(1)
        );
        assert_eq!(
            serde_json::from_str::<ImportHash>("\"42\"").expect("deserialize short hex"),
            ImportHash(0x42)
        );
    }

    #[test]
    fn partial_eq_u64_both_ways() {
        let h = ImportHash(7);
        assert_eq!(h, 7_u64);
        assert_eq!(7_u64, h);
        assert_ne!(h, 8_u64);
    }

    #[test]
    fn import_entry_json_contains_hex_hash() {
        let entry = ImportEntry {
            name: "m".into(),
            dependencies: BTreeSet::default(),
            hash: ImportHash(0x01),
            functions_used: BTreeSet::default(),
        };
        let v = serde_json::to_value(&entry).expect("serialize ImportEntry");
        assert_eq!(v["hash"], json!("0000000000000001"));
    }
}
