//! On-disk cache file: [`FileCache`] and JSON load/save.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use oneil_shared::paths::PythonPath;
use oneil_shared::symbols::{ParameterName, TestIndex};
use serde::{Deserialize, Serialize};

use crate::FunctionCall;
use crate::error::{ReadCacheError, WriteCacheError};
use crate::imports::ImportEntry;

const ONEIL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// On-disk cache for one source file: imported modules, parameter calls, and test calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileCache {
    /// The version of Oneil that created the cache.
    pub oneil_version: String,
    /// Imported Python modules and their invalidation metadata.
    pub imports: BTreeMap<PythonPath, ImportEntry>,
    /// Function calls originating from named parameters.
    pub parameters: BTreeMap<ParameterName, Vec<FunctionCall>>,
    /// Function calls originating from tests, keyed by test index.
    pub tests: BTreeMap<TestIndex, Vec<FunctionCall>>,
}

impl FileCache {
    /// Creates a new empty file cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            oneil_version: ONEIL_VERSION.to_owned(),
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

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}
