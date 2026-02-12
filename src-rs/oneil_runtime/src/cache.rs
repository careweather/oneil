//! Generic path-keyed cache using [`LoadResult`].

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_shared::load_result::LoadResult;

use crate::output;

/// Cache for raw source file contents keyed by path.
pub type SourceCache = Cache<String, InternalIoError>;

/// Cache for parsed AST models keyed by path.
pub type AstCache = Cache<output::ast::ModelNode, output::error::ParseError>;

/// Cache for resolved IR models keyed by path.
pub type IrCache = Cache<output::ir::Model, output::error::ResolutionError>;

/// Cache for evaluated output models keyed by path.
pub type EvalCache = Cache<output::Model, output::error::EvalError>;

/// Cache for Python import function maps keyed by path.
#[cfg(feature = "python")]
pub type PythonImportCache =
    Cache<oneil_python::function::PythonFunctionMap, oneil_shared::error::OneilError>;

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

    /// Returns the value for `path`, if present.
    #[must_use]
    pub fn get_value(&self, path: &Path) -> Option<&T> {
        self.entries.get(path).and_then(LoadResult::value)
    }

    /// Returns the error for `path`, if present.
    #[must_use]
    pub fn get_error(&self, path: &Path) -> Option<&E> {
        self.entries.get(path).and_then(LoadResult::error)
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
