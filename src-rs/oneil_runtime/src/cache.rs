//! Generic path-keyed cache using [`LoadResult`], and a source cache for raw file contents.

use std::hash::{DefaultHasher, Hash, Hasher};

use indexmap::IndexMap;
use oneil_parser::error::ParserError;
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::{ModelPath, PythonPath, SourcePath},
};

use crate::{
    error::{PythonImportError, SourceError},
    output,
};

/// Content hash for cached source, used to detect when file contents change.
pub fn source_hash(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

/// Result of inserting a source into the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertSourceResult {
    /// The path had no prior cache entry; the source was inserted fresh.
    InsertedNewSource,
    /// The path was already cached and the content has changed; the old entry
    /// was replaced. Derived caches (AST, unit graphs, eval) must be
    /// invalidated.
    UpdatedExistingSource,
    /// A source with the same hash already exists in the cache; nothing was
    /// changed.
    MatchingSourceExists,
}

/// Cached source for a path, with an optional content hash when load succeeded.
#[derive(Debug)]
struct SourceCacheEntry {
    /// Hash of the source when load succeeded; `None` when load failed.
    pub hash: u64,
    /// The loaded source or the load error.
    pub source: String,
}

/// Cache for source file contents keyed by path.
///
/// Stores a [`Result`] per path: either the file contents as a [`SourceCacheEntry`] or a
/// [`SourceError`](crate::error::SourceError) when loading failed.
///
/// This is specialized for source files because, unlike other caches,
/// there is no possible partial result.
#[derive(Debug)]
pub struct SourceCache {
    entries: IndexMap<SourcePath, Result<SourceCacheEntry, SourceError>>,
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

    /// Returns the cached result for `path`, if present.
    #[must_use]
    pub fn get_entry(&self, path: &SourcePath) -> Option<Result<&str, &SourceError>> {
        self.entries
            .get(path)
            .map(|result| result.as_ref().map(|entry| entry.source.as_str()))
    }

    /// Inserts a result for `path`, replacing any existing entry. Computes and stores the content
    /// hash when the load succeeded.
    pub fn insert(
        &mut self,
        path: SourcePath,
        result: Result<String, SourceError>,
    ) -> InsertSourceResult {
        let is_update = self.entries.contains_key(&path);
        match result {
            Ok(source) => {
                let hash = source_hash(source.as_str());
                if self.contains_matching(&path, hash) {
                    InsertSourceResult::MatchingSourceExists
                } else {
                    self.entries
                        .insert(path, Ok(SourceCacheEntry { hash, source }));
                    if is_update {
                        InsertSourceResult::UpdatedExistingSource
                    } else {
                        InsertSourceResult::InsertedNewSource
                    }
                }
            }
            Err(e) => {
                self.entries.insert(path, Err(e));
                if is_update {
                    InsertSourceResult::UpdatedExistingSource
                } else {
                    InsertSourceResult::InsertedNewSource
                }
            }
        }
    }

    /// Checks if the cache contains an entry for `path` matching `source`. Uses hashes to determine
    /// equality.
    #[must_use]
    fn contains_matching(&self, path: &SourcePath, hash: u64) -> bool {
        self.entries
            .get(path)
            .is_some_and(|result| result.as_ref().is_ok_and(|entry| entry.hash == hash))
    }

    /// Returns an iterator over path–result pairs.
    pub fn paths(&self) -> impl Iterator<Item = &SourcePath> {
        self.entries.iter().map(|(path, _)| path)
    }
}

/// Cache for parsed AST models keyed by path.
pub type AstCache = ModelCache<output::ast::ModelNode, Vec<ParserError>>;

/// Cache for evaluated output models keyed by file path and import instance.
#[derive(Debug, Default)]
pub struct EvalCache {
    entries: IndexMap<EvalInstanceKey, LoadResult<output::Model, output::ModelEvalErrors>>,
}

impl EvalCache {
    /// Creates an empty evaluation cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached entry for the root instance of `path`, if present.
    #[must_use]
    pub fn get_entry(
        &self,
        path: &ModelPath,
    ) -> Option<&LoadResult<output::Model, output::ModelEvalErrors>> {
        self.entries.get(&EvalInstanceKey::root(path.clone()))
    }

    /// Returns the cached entry for a specific evaluated instance.
    #[must_use]
    pub fn get_entry_instance(
        &self,
        key: &EvalInstanceKey,
    ) -> Option<&LoadResult<output::Model, output::ModelEvalErrors>> {
        self.entries.get(key)
    }

    /// Inserts a result for an evaluated instance, replacing any existing entry.
    pub fn insert(
        &mut self,
        key: EvalInstanceKey,
        result: LoadResult<output::Model, output::ModelEvalErrors>,
    ) {
        self.entries.insert(key, result);
    }

    /// Clears all cached evaluations.
    ///
    /// The eval cache cannot be selectively invalidated: when any source file
    /// changes, models that transitively depend on it hold stale results. A full
    /// clear is the only safe option; re-evaluation is always done fresh by
    /// [`eval_model_with_designs`](oneil_eval::eval_model_with_designs) anyway.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Iterates all instance keys and their load results.
    pub fn iter(
        &self,
    ) -> indexmap::map::Iter<'_, EvalInstanceKey, LoadResult<output::Model, output::ModelEvalErrors>>
    {
        self.entries.iter()
    }
}

/// Generic cache keyed by path, storing [`LoadResult<T, E>`] per path.
///
/// Used to cache load outcomes (success, partial, or failure) for files or
/// resources identified by path.
#[derive(Debug)]
pub struct ModelCache<T, E> {
    entries: IndexMap<ModelPath, LoadResult<T, E>>,
}

impl<T, E> Default for ModelCache<T, E> {
    fn default() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }
}

impl<T, E> ModelCache<T, E> {
    /// Creates an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the full cached entry for `path`.
    #[must_use]
    pub fn get_entry(&self, path: &ModelPath) -> Option<&LoadResult<T, E>> {
        self.entries.get(path)
    }

    /// Inserts a [`LoadResult`] for `path`, replacing any existing entry.
    pub fn insert(&mut self, path: ModelPath, result: LoadResult<T, E>) {
        self.entries.insert(path, result);
    }

    /// Removes the cached entry for `path`, if present.
    pub fn remove(&mut self, path: &ModelPath) {
        self.entries.swap_remove(path);
    }
}

/// Cache for Python import function maps keyed by path.
///
/// Stores a [`Result`] per path: either the loaded [`PythonFunctionMap`] or a
/// [`PythonImportError`](crate::error::PythonImportError) when loading failed.
#[derive(Debug)]
pub struct PythonImportCache {
    entries: IndexMap<PythonPath, Result<oneil_python::function::PythonModule, PythonImportError>>,
}

impl Default for PythonImportCache {
    fn default() -> Self {
        Self {
            entries: IndexMap::new(),
        }
    }
}

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
        path: &PythonPath,
    ) -> Option<&Result<oneil_python::function::PythonModule, PythonImportError>> {
        self.entries.get(path)
    }

    /// Inserts a result for `path`, replacing any existing entry.
    pub fn insert(
        &mut self,
        path: PythonPath,
        result: Result<oneil_python::function::PythonModule, PythonImportError>,
    ) {
        self.entries.insert(path, result);
    }

    /// Removes the cached entry for `path`, if present.
    pub fn remove(&mut self, path: &PythonPath) {
        self.entries.swap_remove(path);
    }
}
