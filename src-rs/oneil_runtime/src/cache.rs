//! Generic path-keyed cache using [`LoadResult`], and a source cache for raw file contents.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_eval as eval;
use oneil_parser::error::ParserError;
use oneil_resolver as resolver;
use oneil_shared::load_result::LoadResult;

use crate::{error::SourceError, output};

#[cfg(feature = "python")]
use crate::error::PythonImportError;

/// Cache for source file contents keyed by path.
///
/// Stores a [`Result`] per path: either the file contents as a [`String`] or a
/// [`SourceError`](crate::error::SourceError) when loading failed.
///
/// This is specialized for source files because, unlike other caches,
/// there is no possible partial result.
#[derive(Debug)]
pub struct SourceCache {
    entries: IndexMap<PathBuf, Result<String, SourceError>>,
}

impl Default for SourceCache {
    fn default() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }
}

impl SourceCache {
    /// Creates an empty source cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the full cached entry for `path`.
    #[must_use]
    pub fn get_entry(&self, path: &Path) -> Option<&Result<String, SourceError>> {
        self.entries.get(path)
    }

    /// Inserts a result for `path`, replacing any existing entry.
    pub fn insert(&mut self, path: PathBuf, result: Result<String, SourceError>) {
        self.entries.insert(path, result);
    }

    /// Returns an iterator over path–result pairs.
    pub fn iter(&self) -> indexmap::map::Iter<'_, PathBuf, Result<String, SourceError>> {
        self.entries.iter()
    }
}

/// Cache for parsed AST models keyed by path.
pub type AstCache = Cache<output::ast::ModelNode, Vec<ParserError>>;

/// Cache for resolved IR models keyed by path.
pub type IrCache = Cache<output::ir::Model, resolver::ResolutionErrorCollection>;

/// Cache for evaluated output models keyed by path.
pub type EvalCache = Cache<output::Model, eval::EvalErrors>;

/// Cache for Python import function maps keyed by path.
///
/// Stores a [`Result`] per path: either the loaded [`PythonFunctionMap`] or a
/// [`PythonImportError`](crate::error::PythonImportError) when loading failed.
#[cfg(feature = "python")]
#[derive(Debug)]
pub struct PythonImportCache {
    entries:
        IndexMap<PathBuf, Result<oneil_python::function::PythonFunctionMap, PythonImportError>>,
}

#[cfg(feature = "python")]
impl Default for PythonImportCache {
    fn default() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }
}

#[cfg(feature = "python")]
impl PythonImportCache {
    /// Creates an empty Python import cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the full cached entry for `path`.
    #[must_use]
    pub fn get_entry(
        &self,
        path: &Path,
    ) -> Option<&Result<oneil_python::function::PythonFunctionMap, PythonImportError>> {
        self.entries.get(path)
    }

    /// Inserts a result for `path`, replacing any existing entry.
    pub fn insert(
        &mut self,
        path: PathBuf,
        result: Result<oneil_python::function::PythonFunctionMap, PythonImportError>,
    ) {
        self.entries.insert(path, result);
    }
}

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
