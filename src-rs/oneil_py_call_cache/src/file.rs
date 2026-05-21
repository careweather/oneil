//! On-disk cache file: [`FileCache`] and JSON load/save.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, fs::File};

use oneil_shared::{paths::PythonPath, symbols::PyFunctionName};
use semver::{BuildMetadata, Prerelease, Version};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FunctionCall, ReadCacheError, WriteCacheError};

const ONEIL_VERSION: &str = env!("CARGO_PKG_VERSION");

const MINIMUM_ACCEPTED_VERSION: Version = Version {
    major: 0,
    minor: 16,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::EMPTY,
};

/// On-disk cache for one python module.
#[derive(Debug, Clone, PartialEq)]
pub struct FileCache {
    /// The version of Oneil that created the cache.
    pub oneil_version: Version,
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
    ///
    /// # Panics
    ///
    /// Panics if `CARGO_PKG_VERSION` is not a valid semver string.
    #[must_use]
    pub fn new(module_path: PythonPath, hash: ImportHash, dependencies: BTreeSet<PathBuf>) -> Self {
        Self {
            oneil_version: Version::parse(ONEIL_VERSION).expect("version should be valid"),
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

impl Serialize for FileCache {
    /// Serializes `oneil_version` first as a string, matching the on-disk field order.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        const FIELD_COUNT: usize = 5;
        let mut state = serializer.serialize_struct("FileCache", FIELD_COUNT)?;
        state.serialize_field("oneil_version", &self.oneil_version.to_string())?;
        state.serialize_field("module_path", &self.module_path)?;
        state.serialize_field("hash", &self.hash)?;
        state.serialize_field("dependencies", &self.dependencies)?;
        state.serialize_field("function_calls", &self.function_calls)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for FileCache {
    /// Deserializes `oneil_version` first and rejects unsupported versions before other fields.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "oneil_version",
            "module_path",
            "hash",
            "dependencies",
            "function_calls",
        ];

        deserializer.deserialize_struct("FileCache", FIELDS, FileCacheVisitor)
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum FileCacheField {
    OneilVersion,
    ModulePath,
    Hash,
    Dependencies,
    FunctionCalls,
}

struct FileCacheVisitor;

impl<'de> Visitor<'de> for FileCacheVisitor {
    type Value = FileCache;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("struct FileCache")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let oneil_version = match map.next_key::<FileCacheField>()? {
            Some(FileCacheField::OneilVersion) => {
                let version = map.next_value::<String>()?;
                let version = Version::parse(&version).map_err(de::Error::custom)?;

                if version < MINIMUM_ACCEPTED_VERSION {
                    return Err(de::Error::custom(format!(
                        "unsupported oneil_version: {version}"
                    )));
                }
                version
            }
            Some(_) => {
                return Err(de::Error::custom(
                    "oneil_version must be the first field in a cache file",
                ));
            }
            None => return Err(de::Error::missing_field("oneil_version")),
        };

        let mut module_path = None;
        let mut hash = None;
        let mut dependencies = None;
        let mut function_calls = None;

        while let Some(key) = map.next_key::<FileCacheField>()? {
            match key {
                FileCacheField::OneilVersion => {
                    return Err(de::Error::duplicate_field("oneil_version"));
                }
                FileCacheField::ModulePath => {
                    if module_path.is_some() {
                        return Err(de::Error::duplicate_field("module_path"));
                    }
                    module_path = Some(map.next_value()?);
                }
                FileCacheField::Hash => {
                    if hash.is_some() {
                        return Err(de::Error::duplicate_field("hash"));
                    }
                    hash = Some(map.next_value()?);
                }
                FileCacheField::Dependencies => {
                    if dependencies.is_some() {
                        return Err(de::Error::duplicate_field("dependencies"));
                    }
                    dependencies = Some(map.next_value()?);
                }
                FileCacheField::FunctionCalls => {
                    if function_calls.is_some() {
                        return Err(de::Error::duplicate_field("function_calls"));
                    }
                    function_calls = Some(map.next_value()?);
                }
            }
        }

        Ok(FileCache {
            oneil_version,
            module_path: module_path.ok_or_else(|| de::Error::missing_field("module_path"))?,
            hash: hash.ok_or_else(|| de::Error::missing_field("hash"))?,
            dependencies: dependencies.ok_or_else(|| de::Error::missing_field("dependencies"))?,
            function_calls: function_calls
                .ok_or_else(|| de::Error::missing_field("function_calls"))?,
        })
    }
}

/// Fingerprint for a python module's sources (stored as raw `u64`, serialized as hex).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ImportHash(u64);

impl fmt::Display for ImportHash {
    /// Formats an [`ImportHash`] as a 16-digit lowercase hex string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

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
