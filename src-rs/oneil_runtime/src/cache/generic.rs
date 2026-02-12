//! Generic path-keyed cache using [`LoadResult`].

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_shared::LoadResult;

/// Generic cache keyed by path, storing [`LoadResult<T, E>`] per path.
///
/// Used to cache load outcomes (success, partial, or failure) for files or
/// resources identified by path.
#[derive(Debug)]
pub struct Cache<T, E> {
    entries: IndexMap<PathBuf, LoadResult<T, E>>,
}

impl<T, E> Default for Cache<T, E> {
    fn default() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }
}

impl<T, E> Cache<T, E> {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the full cached entry for `path`.
    #[must_use]
    pub fn get_entry(&self, path: &Path) -> Option<&LoadResult<T, E>> {
        self.entries.get(path)
    }

    /// Inserts a [`LoadResult`] for `path`, replacing any existing entry.
    pub fn insert(&mut self, path: PathBuf, result: LoadResult<T, E>) {
        self.entries.insert(path, result);
    }

    /// Returns whether `path` has a cached entry.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.entries.contains_key(path)
    }

    /// Returns the number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over path–result pairs.
    pub fn iter(&self) -> indexmap::map::Iter<'_, PathBuf, LoadResult<T, E>> {
        self.entries.iter()
    }
}
