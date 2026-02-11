//! Cache for Python import load results (set of callable names or error).

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use indexmap::IndexSet;
use oneil_shared::error::OneilError;

/// Result of loading Python import for a path: set of callable names or error.
pub type PythonImportLoadResult = Result<IndexSet<String>, OneilError>;

/// Cache of Python import load results keyed by path.
#[derive(Debug, Default)]
pub struct PythonImportCache {
    entries: IndexMap<PathBuf, PythonImportLoadResult>,
}

impl PythonImportCache {
    /// Creates an empty Python import cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached result for `path`, if present.
    pub fn get_entry(&self, path: &Path) -> Option<&PythonImportLoadResult> {
        self.entries.get(path)
    }

    /// Returns the cached error for `path`, if present.
    pub fn get_error(&self, path: &Path) -> Option<&OneilError> {
        self.entries.get(path)?.as_ref().err()
    }

    /// Stores a successful load result (set of callable names) for `path`.
    pub fn insert_ok(&mut self, path: PathBuf, names: IndexSet<String>) {
        self.entries.insert(path, Ok(names));
    }

    /// Stores a load error for `path`.
    pub fn insert_err(&mut self, path: PathBuf, error: OneilError) {
        self.entries.insert(path, Err(error));
    }
}
