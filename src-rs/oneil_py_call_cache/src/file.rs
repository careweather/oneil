//! On-disk cache file: [`FileCache`] and JSON load/save.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, fs::File};

use oneil_shared::{paths::PythonPath, symbols::PyFunctionName};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{FunctionCall, ReadCacheError, WriteCacheError};

const ONEIL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// On-disk cache for one python module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileCache {
    /// The version of Oneil that created the cache.
    pub oneil_version: String,
    /// The path of the python module that was cached.
    pub module_path: PythonPath,
    /// The hash of the python module and its dependencies.
    pub hash: ImportHash,
    /// The local dependencies included in the combined hash.
    pub dependencies: BTreeSet<PathBuf>,
    /// The function calls that are cached for this module.
    pub function_calls: BTreeMap<PyFunctionName, Vec<FunctionCall>>,
}

impl FileCache {
    /// Creates a new empty file cache.
    #[must_use]
    pub fn new(module_path: PythonPath, hash: ImportHash, dependencies: BTreeSet<PathBuf>) -> Self {
        Self {
            oneil_version: ONEIL_VERSION.to_owned(),
            module_path,
            hash,
            dependencies,
            function_calls: BTreeMap::new(),
        }
    }

    /// Writes this cache as pretty-printed JSON to `path`, creating or truncating the file.
    ///
    /// # Errors
    ///
    /// Returns [`WriteCacheError`] if the file cannot be created or JSON serialization fails.
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), WriteCacheError> {
        if let Some(dir) = path.as_ref().parent()
            && !dir.exists()
        {
            std::fs::create_dir_all(dir).map_err(WriteCacheError::Io)?;
        }

        let file = File::create(path.as_ref()).map_err(WriteCacheError::Io)?;
        serde_json::to_writer_pretty(file, self).map_err(WriteCacheError::Serde)?;
        Ok(())
    }

    /// Reads a [`FileCache`] from JSON at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadCacheError`] if the file cannot be opened or JSON deserialization fails.
    pub fn read_from_path(path: impl AsRef<Path>) -> Result<Self, ReadCacheError> {
        let file = File::open(path.as_ref()).map_err(ReadCacheError::Io)?;
        serde_json::from_reader(file).map_err(ReadCacheError::Serde)
    }
}

/// Fingerprint for a python module's sources (stored as raw `u64`, serialized as hex).
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
