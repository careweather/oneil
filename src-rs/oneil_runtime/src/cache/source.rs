//! Cache for source file contents and associated load errors.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;

/// Cache of loaded source file contents and per-file load errors.
#[derive(Debug, Default)]
pub struct SourceCache {
    /// Successfully loaded source keyed by path.
    contents: IndexMap<PathBuf, String>,
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

    /// Stores successfully loaded `source` for `path`.
    pub fn insert(&mut self, path: PathBuf, source: String) {
        self.contents.insert(path, source);
    }
}
