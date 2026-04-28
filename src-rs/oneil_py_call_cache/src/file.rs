//! On-disk cache file: [`FileCache`] and JSON load/save.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::FunctionCall;
use crate::error::{ReadCacheError, WriteCacheError};
use crate::identifiers::{CachedParameterName, CachedPythonPath, CachedTestIndex};
use crate::imports::ImportEntry;

/// On-disk cache for one source file: imported modules, parameter calls, and test calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileCache {
    /// Imported Python modules and their invalidation metadata.
    pub imports: BTreeMap<CachedPythonPath, ImportEntry>,
    /// Function calls originating from named parameters.
    pub parameters: BTreeMap<CachedParameterName, Vec<FunctionCall>>,
    /// Function calls originating from tests, keyed by test index.
    pub tests: BTreeMap<CachedTestIndex, Vec<FunctionCall>>,
}

impl FileCache {
    /// Creates a new empty file cache.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            imports: BTreeMap::new(),
            parameters: BTreeMap::new(),
            tests: BTreeMap::new(),
        }
    }

    /// Writes this cache as pretty-printed JSON to `path`, creating or truncating the file.
    ///
    /// # Errors
    ///
    /// Returns [`WriteCacheError`] if the file cannot be created or JSON serialization fails.
    ///
    /// ```
    /// use oneil_py_call_cache::FileCache;
    ///
    /// let dir = std::env::temp_dir();
    /// let path = dir.join(format!("oneil_cache_doc_{}.json", std::process::id()));
    /// let cache = FileCache {
    ///     imports: Default::default(),
    ///     parameters: Default::default(),
    ///     tests: Default::default(),
    /// };
    /// cache.write_to_path(&path).expect("write cache file");
    /// assert!(path.exists());
    /// # std::fs::remove_file(&path).expect("remove temp cache file");
    /// ```
    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), WriteCacheError> {
        let file = File::create(path.as_ref()).map_err(WriteCacheError::Io)?;
        serde_json::to_writer_pretty(file, self).map_err(WriteCacheError::Serde)?;
        Ok(())
    }

    /// Reads a [`FileCache`] from JSON at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`ReadCacheError`] if the file cannot be opened/read or JSON deserialization fails.
    ///
    /// ```
    /// use oneil_py_call_cache::FileCache;
    ///
    /// let dir = std::env::temp_dir();
    /// let path = dir.join(format!("oneil_cache_read_doc_{}.json", std::process::id()));
    /// let cache = FileCache {
    ///     imports: Default::default(),
    ///     parameters: Default::default(),
    ///     tests: Default::default(),
    /// };
    /// cache.write_to_path(&path).expect("write cache file");
    /// let loaded = FileCache::read_from_path(&path).expect("read cache file");
    /// assert_eq!(loaded, cache);
    /// # std::fs::remove_file(&path).expect("remove temp cache file");
    /// ```
    pub fn read_from_path(path: impl AsRef<Path>) -> Result<Self, ReadCacheError> {
        let file = File::open(path.as_ref()).map_err(ReadCacheError::Io)?;
        serde_json::from_reader(file).map_err(ReadCacheError::Serde)
    }
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}
