//! On-disk cache file: [`FileCache`] and JSON load/save.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, fs::File};

use oneil_shared::{paths::PythonPath, symbols::PyFunctionName};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FunctionCall, ReadCacheError, WriteCacheError};

/// On-disk cache for one python module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum FileCache {
    /// Version 1 of the cache format.
    #[serde(rename = "v1")]
    V1 {
        /// The path of the python module that was cached.
        module_path: PythonPath,
        /// The hash of the python module and its dependencies.
        hash: ImportHash,
        /// The local dependencies included in the combined hash.
        dependencies: BTreeSet<PathBuf>,
        /// The function calls that are cached for this module.
        function_calls: BTreeMap<PyFunctionName, Vec<FunctionCall>>,
    },
}

impl FileCache {
    /// Creates a new empty file cache.
    #[must_use]
    pub const fn new(
        module_path: PythonPath,
        hash: ImportHash,
        dependencies: BTreeSet<PathBuf>,
    ) -> Self {
        Self::V1 {
            module_path,
            hash,
            dependencies,
            function_calls: BTreeMap::new(),
        }
    }

    /// Returns the module path.
    #[must_use]
    pub const fn module_path(&self) -> &PythonPath {
        match self {
            Self::V1 { module_path, .. } => module_path,
        }
    }

    /// Returns the hash.
    #[must_use]
    pub const fn hash(&self) -> ImportHash {
        match self {
            Self::V1 { hash, .. } => *hash,
        }
    }

    /// Returns the dependencies set.
    #[must_use]
    pub const fn dependencies(&self) -> &BTreeSet<PathBuf> {
        match self {
            Self::V1 { dependencies, .. } => dependencies,
        }
    }

    /// Returns the function calls map.
    #[must_use]
    pub const fn function_calls(&self) -> &BTreeMap<PyFunctionName, Vec<FunctionCall>> {
        match self {
            Self::V1 { function_calls, .. } => function_calls,
        }
    }

    /// Returns the function calls map.
    #[must_use]
    pub const fn function_calls_mut(&mut self) -> &mut BTreeMap<PyFunctionName, Vec<FunctionCall>> {
        match self {
            Self::V1 { function_calls, .. } => function_calls,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use oneil_output::{Number, Value};
    use oneil_shared::{
        paths::{ModelPath, PythonPath},
        symbols::PyFunctionName,
    };
    use serde_json::json;

    use super::{FileCache, ImportHash};
    use crate::{FunctionCall, FunctionCallResult, ReadCacheError};

    static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    const SAMPLE_FILE_CACHE_V1_JSON: &str = r#"{
  "version": "v1",
  "module_path": "module.py",
  "hash": "0000000000000001",
  "dependencies": [
    "dep.py"
  ],
  "function_calls": {
    "f": [
      {
        "root_models": [
          "model.on"
        ],
        "inputs": [
          1.0
        ],
        "output": 2.0
      }
    ]
  }
}"#;

    /// Creates a unique temporary directory for cache file I/O tests.
    fn unique_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "oneil_py_call_cache_{}_{}",
            std::process::id(),
            TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Removes `dir` after a test, ignoring cleanup failures.
    fn remove_temp_dir(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Builds a cache with one successful call for serde and I/O tests.
    fn sample_file_cache() -> FileCache {
        let mut cache = FileCache::new(
            PythonPath::from_str_no_ext("module"),
            ImportHash::from(1),
            BTreeSet::from([PathBuf::from("dep.py")]),
        );
        cache.function_calls_mut().insert(
            PyFunctionName::from("f"),
            vec![FunctionCall {
                root_models: BTreeSet::from([ModelPath::from_str_no_ext("model")]),
                inputs: vec![Value::Number(Number::Scalar(1.0))],
                output: FunctionCallResult::Success(Value::Number(Number::Scalar(2.0))),
            }],
        );
        cache
    }

    #[test]
    fn import_hash_display_is_16_lowercase_hex_digits() {
        let hash = ImportHash::from(0xAB);

        let displayed = hash.to_string();

        assert_eq!(displayed, "00000000000000ab");
    }

    #[test]
    fn import_hash_serializes_as_16_lowercase_hex_digits() {
        let hash = ImportHash::from(1);

        let json = serde_json::to_value(hash).expect("serialize");

        assert_eq!(json, json!("0000000000000001"));
    }

    #[test]
    fn import_hash_deserializes_short_hex() {
        let json = json!("1");

        let hash: ImportHash = serde_json::from_value(json).expect("deserialize");

        assert_eq!(hash, 1_u64);
    }

    #[test]
    fn import_hash_deserializes_uppercase_hex() {
        let json = json!("AB");

        let hash: ImportHash = serde_json::from_value(json).expect("deserialize");

        assert_eq!(u64::from(hash), 0xAB);
    }

    #[test]
    fn import_hash_rejects_empty_hex_string() {
        let json = json!("");

        let error = serde_json::from_value::<ImportHash>(json).expect_err("empty hex");

        assert!(error.to_string().contains("empty hexadecimal string"));
    }

    #[test]
    fn import_hash_rejects_non_hex_string() {
        let json = json!("zz");

        serde_json::from_value::<ImportHash>(json).expect_err("non-hex");
    }

    #[test]
    fn import_hash_rejects_hex_that_overflows_u64() {
        let json = json!("10000000000000000");

        serde_json::from_value::<ImportHash>(json).expect_err("overflow");
    }

    #[test]
    fn import_hash_serializes_u64_max() {
        let hash = ImportHash::from(u64::MAX);

        let json = serde_json::to_value(hash).expect("serialize");

        assert_eq!(json, json!("ffffffffffffffff"));
    }

    #[test]
    fn import_hash_deserializes_u64_max() {
        let json = json!("ffffffffffffffff");

        let hash: ImportHash = serde_json::from_value(json).expect("deserialize");

        assert_eq!(hash, u64::MAX);
    }

    #[test]
    fn file_cache_serializes_as_versioned_v1_document() {
        let cache = sample_file_cache();

        let json = serde_json::to_value(&cache).expect("serialize");

        assert_eq!(
            json,
            json!({
                "version": "v1",
                "module_path": "module.py",
                "hash": "0000000000000001",
                "dependencies": ["dep.py"],
                "function_calls": {
                    "f": [{
                        "root_models": ["model.on"],
                        "inputs": [1.0],
                        "output": 2.0
                    }]
                }
            })
        );
    }

    #[test]
    fn file_cache_rejects_unknown_version() {
        let json = json!({
            "version": "v2",
            "module_path": "module.py",
            "hash": "0000000000000001",
            "dependencies": [],
            "function_calls": {}
        });

        serde_json::from_value::<FileCache>(json).expect_err("unknown version");
    }

    #[test]
    fn file_cache_module_path_returns_constructor_field() {
        let module_path = PythonPath::from_str_no_ext("module");
        let cache = FileCache::new(
            module_path.clone(),
            ImportHash::from(7),
            BTreeSet::new(),
        );

        assert_eq!(cache.module_path(), &module_path);
    }

    #[test]
    fn file_cache_hash_returns_constructor_field() {
        let hash = ImportHash::from(7);
        let cache = FileCache::new(
            PythonPath::from_str_no_ext("module"),
            hash,
            BTreeSet::new(),
        );

        assert_eq!(cache.hash(), hash);
    }

    #[test]
    fn file_cache_dependencies_return_constructor_field() {
        let dependencies = BTreeSet::from([PathBuf::from("dep.py")]);
        let cache = FileCache::new(
            PythonPath::from_str_no_ext("module"),
            ImportHash::from(7),
            dependencies.clone(),
        );

        assert_eq!(cache.dependencies(), &dependencies);
    }

    #[test]
    fn file_cache_function_calls_are_empty_after_construction() {
        let cache = FileCache::new(
            PythonPath::from_str_no_ext("module"),
            ImportHash::from(7),
            BTreeSet::new(),
        );

        assert!(cache.function_calls().is_empty());
    }

    #[test]
    fn write_to_path_writes_expected_v1_document() {
        let dir = unique_temp_dir();
        let path = dir.join("nested").join("module.json");
        let cache = sample_file_cache();

        cache.write_to_path(&path).expect("write");
        let document = std::fs::read_to_string(&path).expect("read cache file");

        assert_eq!(document, SAMPLE_FILE_CACHE_V1_JSON);
        remove_temp_dir(&dir);
    }

    #[test]
    fn read_from_path_reads_fixed_v1_document() {
        let dir = unique_temp_dir();
        let path = dir.join("module.json");
        std::fs::write(&path, SAMPLE_FILE_CACHE_V1_JSON).expect("write fixture");

        let loaded = FileCache::read_from_path(&path).expect("read");

        assert_eq!(loaded, sample_file_cache());
        remove_temp_dir(&dir);
    }

    #[test]
    fn read_from_path_returns_io_error_when_file_is_missing() {
        let dir = unique_temp_dir();
        let path = dir.join("missing.json");

        let error = FileCache::read_from_path(&path).expect_err("missing file");

        let ReadCacheError::Io(_) = error else {
            panic!("Expected Io, got {error:?}");
        };
        remove_temp_dir(&dir);
    }

    #[test]
    fn read_from_path_returns_serde_error_for_invalid_json() {
        let dir = unique_temp_dir();
        let path = dir.join("module.json");
        std::fs::write(&path, "{not valid json").expect("write invalid json");

        let error = FileCache::read_from_path(&path).expect_err("invalid json");

        let ReadCacheError::Serde(_) = error else {
            panic!("Expected Serde, got {error:?}");
        };
        remove_temp_dir(&dir);
    }
}
