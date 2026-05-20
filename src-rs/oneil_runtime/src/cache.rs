//! Generic path-keyed cache using [`LoadResult`], and a source cache for raw file contents.

use std::{
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Component, PathBuf},
};

use indexmap::{IndexMap, IndexSet};
use oneil_parser::error::ParserError;
use oneil_py_call_cache::{
    FileCache, FunctionCall, FunctionCallResult, ImportHash, ReadCacheError, WriteCacheError,
};
use oneil_python::{PythonEvalError, function::PythonModule};
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::{ModelPath, PythonPath, SourcePath},
    symbols::PyFunctionName,
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

/// Cache for Python function calls keyed by path.
#[derive(Debug, Clone)]
pub struct PythonCallCache {
    cache_dir: PathBuf,
    entries: IndexMap<PythonPath, FileCache>,
    updated_root_models: IndexSet<ModelPath>,
    cache_read_policy: CacheReadPolicy,
    cache_write_policy: CacheWritePolicy,
}

impl PythonCallCache {
    /// Creates a new Python call cache in the given directory.
    pub fn new(
        cache_dir: PathBuf,
        cache_read_policy: CacheReadPolicy,
        cache_write_policy: CacheWritePolicy,
    ) -> Self {
        Self {
            cache_dir,
            entries: IndexMap::new(),
            updated_root_models: IndexSet::new(),
            cache_read_policy,
            cache_write_policy,
        }
    }

    /// Begins a new evaluation.
    pub fn begin_evaluation(&mut self) {
        self.updated_root_models.clear();
    }

    /// Ends the current evaluation.
    pub fn end_evaluation(&mut self) -> Result<(), Vec<WriteCacheError>> {
        // clear all stale cached function call
        let stale_caches = self.clear_stale_entries();

        // remove the stale cache files
        for python_path in stale_caches {
            let cache_path = self.get_cache_path(&python_path);
            if cache_path.exists() {
                std::fs::remove_file(cache_path).expect("failed to remove stale cache file");
            }
        }

        // save all the cache entries to disk
        self.save_all()
    }

    /// Gets a cached result for a function call.
    pub fn get(
        &mut self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        args: &[output::Value],
        module_hash: u64,
    ) -> Option<Result<output::Value, PythonEvalError>> {
        match self.cache_read_policy {
            CacheReadPolicy::Never => {
                return None;
            }
            CacheReadPolicy::Always => (),
            CacheReadPolicy::Prompt => {
                todo!()
            }
        }

        // try to load the cache entry for the python path
        //
        // for now, we simply ignore the error
        let _ = self.load(python_path);

        let cache = self
            .entries
            .get(python_path)
            .filter(|cache| cache.hash == module_hash)?;

        let function_calls = cache.function_calls.get(identifier)?;
        let function_call = function_calls.iter().find(|call| call.inputs == args)?;

        Some(function_call.output.clone().into())
    }

    /// Inserts a result for a function call.
    pub fn insert(
        &mut self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
        args: &[output::Value],
        result: &Result<output::Value, PythonEvalError>,
        root_model: &ModelPath,
        python_module: &PythonModule,
    ) {
        match self.cache_write_policy {
            CacheWritePolicy::Always => (),
            CacheWritePolicy::Never => return,
            CacheWritePolicy::Prompt => todo!(),
        }

        let module_hash = python_module.get_hash();
        let module_dependencies: BTreeSet<_> =
            python_module.get_imports().iter().cloned().collect();

        // Try to load the cache entry for the python path. If it fails,
        // create a new cache entry.
        //
        // NOTE: currently, if the cache entry exists but the JSON entry
        // fails to parse (e.g. because the version is incompatible), this
        // will **overwrite** the existing cache with a new one. It would
        // make more sense to handle an invalid cache entry with a warning
        // or something like that. However, to avoid getting too far into
        // the weeds on the first draft, we simply overwrite the existing
        // cache with a new one.
        let _ = self.load(python_path);

        // If the root model is not in the updated root models set, remove
        // all references to the root model, then add it to the updated
        // root models
        //
        // This potentially invalidates all cached function calls for the
        // root model. If the entries aren't referenced during the current
        // evaluation of the root model, they will be cleared when
        // `end_evaluation` is called.
        if !self.updated_root_models.contains(root_model) {
            self.clear_root_model(root_model);
            self.updated_root_models.insert(root_model.clone());
        }

        let cache = self
            .entries
            .entry(python_path.clone())
            .and_modify(|cache| {
                // if the cache entry exists but the hash does not match, we need
                // to clear the cache entry
                if cache.hash != module_hash {
                    *cache = FileCache::new(
                        python_path.clone(),
                        ImportHash::from(module_hash),
                        module_dependencies.clone(),
                    );
                }
            })
            .or_insert_with(|| {
                // if the cache entry does not exist, create a new one
                FileCache::new(
                    python_path.clone(),
                    ImportHash::from(module_hash),
                    module_dependencies.clone(),
                )
            });

        // Find the matching function call for the given identifier and
        // arguments (if it exists)
        let cached_function_calls = cache.function_calls.entry(identifier.clone()).or_default();
        let matching_function_call = cached_function_calls
            .iter_mut()
            .find(|call| call.inputs == args);

        let result = FunctionCallResult::from(result.clone());

        // If the function call exists, update it. Otherwise, create a new one.
        if let Some(matching_function_call) = matching_function_call {
            // If the result has changed, update the output and clear the root models.
            if matching_function_call.output != result {
                matching_function_call.output = result;
                matching_function_call.root_models.clear();
            }

            // Add the root model to the function call.
            matching_function_call
                .root_models
                .insert(root_model.clone());
        } else {
            // If the function call does not exist, create a new one.
            cached_function_calls.push(FunctionCall {
                root_models: BTreeSet::from_iter([root_model.clone()]),
                inputs: args.to_vec(),
                output: result,
            });
        }
    }

    /// Loads a cache entry from disk.
    ///
    /// If the cache entry already exists, this does nothing.
    ///
    /// # Errors
    ///
    /// Returns [`ReadCacheError`] if the cache file cannot be read.
    fn load(&mut self, python_path: &PythonPath) -> Result<(), ReadCacheError> {
        match self.cache_read_policy {
            CacheReadPolicy::Always => (),
            CacheReadPolicy::Never => return Ok(()),
            CacheReadPolicy::Prompt => todo!(),
        }

        if self.entries.contains_key(python_path) {
            return Ok(());
        }

        let cache_path = self.get_cache_path(python_path);
        let cache = FileCache::read_from_path(cache_path)?;
        self.entries.insert(python_path.clone(), cache);
        Ok(())
    }

    /// Saves all cache entries to disk.
    ///
    /// # Errors
    ///
    /// Returns a vector of [`WriteCacheError`] if the cache files cannot be written.
    fn save_all(&self) -> Result<(), Vec<WriteCacheError>> {
        match self.cache_write_policy {
            CacheWritePolicy::Always => (),
            CacheWritePolicy::Never => return Ok(()),
            CacheWritePolicy::Prompt => todo!(),
        }

        let mut errors = Vec::new();
        for (model_path, cache) in &self.entries {
            let cache_path = self.get_cache_path(model_path);
            match cache.write_to_path(cache_path) {
                Ok(()) => (),
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Removes all references to a root model from all cached function calls.
    fn clear_root_model(&mut self, root_model: &ModelPath) {
        for cache in self.entries.values_mut() {
            for function_calls in cache.function_calls.values_mut() {
                for function_call in function_calls.iter_mut() {
                    function_call.root_models.remove(root_model);
                }
            }
        }
    }

    /// Clear all entries that are no longer referenced by any root models.
    fn clear_stale_entries(&mut self) -> IndexSet<PythonPath> {
        for cache in self.entries.values_mut() {
            // clear all cached function calls that are no longer
            // referenced by any root models
            for function_calls in cache.function_calls.values_mut() {
                function_calls.retain(|call| !call.root_models.is_empty());
            }

            // clear all functions that have no cached function calls
            cache
                .function_calls
                .retain(|_function_name, function_calls| !function_calls.is_empty());
        }

        // clear all the caches that have no function calls
        self.entries
            .extract_if(.., |_python_path, cache| cache.function_calls.is_empty())
            .map(|(python_path, _cache)| python_path)
            .collect()
    }

    fn get_cache_path(&self, python_path: &PythonPath) -> PathBuf {
        let cache_relative_path = python_path
            .as_path()
            .with_extension("json")
            .components()
            // convert to a path that can be used in the cache directory
            .fold(PathBuf::new(), append_normalized_component);

        self.cache_dir.join(cache_relative_path)
    }
}

fn append_normalized_component(mut path: PathBuf, component: Component<'_>) -> PathBuf {
    match component {
        Component::Prefix(_) => {
            path.push("__prefix__");
        }
        Component::RootDir => {
            path.push("__root__");
        }
        Component::CurDir => {}
        // in order to avoid overwriting files outside of the cache directory,
        // we convert ".." to "__parent__"
        Component::ParentDir => {
            if let Some(parent) = path.parent()
                && !parent.ends_with("__parent__")
            {
                path = parent.to_path_buf();
            } else {
                path.push("__parent__");
            }
        }
        Component::Normal(os_str) => {
            path.push(os_str);
        }
    }
    path
}

/// When the Python call cache may read from the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheReadPolicy {
    /// Always read from the cache.
    Always,
    /// Never read from the cache.
    Never,
    /// Ask before reading from the cache.
    Prompt,
}

/// When the Python call cache may write to the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheWritePolicy {
    /// Always write to the cache.
    Always,
    /// Never write to the cache.
    Never,
    /// Ask before writing to the cache.
    Prompt,
}
