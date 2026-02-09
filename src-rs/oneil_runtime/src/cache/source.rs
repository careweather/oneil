//! Cache for source file contents and associated load errors.

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_shared::error::OneilError;

/// Cache of loaded source file contents and per-file load errors.
#[derive(Debug, Default)]
pub struct SourceCache {
    /// Successfully loaded source keyed by path.
    contents: IndexMap<PathBuf, String>,
    /// Load error for a path, when reading the file failed.
    errors: IndexMap<PathBuf, OneilError>,
}

impl SourceCache {
    /// Creates an empty source cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached source for `path`, if present.
    pub fn get(&self, path: &Path) -> Option<&str> {
        self.contents.get(path).map(String::as_str)
    }

    /// Returns the cached load error for `path`, if present.
    pub fn get_error(&self, path: &Path) -> Option<&OneilError> {
        self.errors.get(path)
    }

    /// Returns the paths to the files that the source cache relies on.
    pub fn get_paths(&self) -> IndexSet<PathBuf> {
        self.contents.keys().cloned().collect()
    }

    /// Stores successfully loaded `source` for `path`.
    pub fn insert_ok(&mut self, path: PathBuf, source: String) {
        self.contents.insert(path, source);
    }

    /// Stores a load `error` for `path`.
    pub fn insert_err(&mut self, path: PathBuf, error: OneilError) {
        self.errors.insert(path, error);
    }
}
