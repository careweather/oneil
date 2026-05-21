//! Generic path-keyed cache using [`LoadResult`], and a source cache for raw file contents.

use std::{
    collections::BTreeSet,
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Component, PathBuf},
    sync::Arc,
};

use indexmap::{IndexMap, IndexSet};
use oneil_parser::error::ParserError;
use oneil_py_call_cache::{
    FileCache, FunctionCall, FunctionCallResult, ImportHash, ReadCacheError, WriteCacheError,
};
use oneil_python::{PythonEvalError, function::PythonModule};
use oneil_shared::{
    EvalInstanceKey,
    error::{AsOneilDiagnostic, Context, DiagnosticKind, OneilDiagnostic},
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
#[derive(Clone)]
pub struct PythonCallCache {
    cache_dir: PathBuf,
    entries: IndexMap<PythonPath, FileCache>,
    updated_root_models: IndexSet<ModelPath>,
    cache_read_policy: CacheReadPolicy,
    cache_write_policy: CacheWritePolicy,
    warnings: IndexMap<PythonPath, IndexSet<CacheWarning>>,

    /// Whether to overwrite outdated caches.
    overwrite_outdated_caches: IndexMap<PythonPath, bool>,
}

impl fmt::Debug for PythonCallCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PythonCallCache")
            .field("cache_dir", &self.cache_dir)
            .field("entries", &self.entries)
            .field("updated_root_models", &self.updated_root_models)
            .field("cache_read_policy", &self.cache_read_policy)
            .field("cache_write_policy", &self.cache_write_policy)
            .finish_non_exhaustive()
    }
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
            overwrite_outdated_caches: IndexMap::new(),
            warnings: IndexMap::new(),
        }
    }

    /// Begins a new evaluation.
    pub fn begin_evaluation(&mut self) {
        self.updated_root_models.clear();
        self.overwrite_outdated_caches.clear();
        self.warnings.clear();
    }

    /// Returns cache warnings as diagnostics keyed by the on-disk cache file path.
    #[must_use]
    pub fn warning_diagnostics(&self) -> Vec<OneilDiagnostic> {
        self.warnings
            .iter()
            .flat_map(|(python_path, warnings)| {
                let cache_path = self.get_cache_path(python_path);
                warnings
                    .iter()
                    .map(move |warning| OneilDiagnostic::from_error(warning, cache_path.clone()))
            })
            .collect()
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
        if matches!(self.cache_read_policy, CacheReadPolicy::Never) {
            return None;
        }

        let current_hash = ImportHash::from(module_hash);

        // try to load the cache entry for the python path
        self.load(python_path, current_hash);
        let cached_hash = self.entries.get(python_path)?.hash;
        let hash_matches = cached_hash == current_hash;

        if !hash_matches {
            if !self.allow_cache_read_on_hash_mismatch(python_path) {
                return None;
            }

            self.record_warning(
                python_path,
                CacheWarningKind::StaleCacheOnRead {
                    function_name: identifier.clone(),
                },
            );
        }

        let function_calls = self
            .entries
            .get(python_path)?
            .function_calls
            .get(identifier)?;
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
        if matches!(self.cache_write_policy, CacheWritePolicy::Never) {
            return;
        }

        let module_hash = python_module.get_hash();
        let current_hash = ImportHash::from(module_hash);
        let module_dependencies: BTreeSet<_> =
            python_module.get_imports().iter().cloned().collect();

        // Try to load the cache entry for the python path. If it fails,
        // create a new cache entry.
        self.load(python_path, current_hash);

        // check if the cached hash differs from the current hash
        let cached_hash = self.entries.get(python_path).map(|cache| cache.hash);
        let hash_mismatch = cached_hash.is_some_and(|hash| hash != current_hash);

        // if the hash mismatch and the user does not allow it, return
        if hash_mismatch {
            if !self.allow_cache_write_on_hash_mismatch(python_path) {
                return;
            }

            self.record_warning(python_path, CacheWarningKind::OverwriteCacheOnHashMismatch);
        }

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

        self.entries
            .entry(python_path.clone())
            .and_modify(|cache| {
                // if the cache entry exists but the hash does not match, we need
                // to clear the cache entry
                if cache.hash != module_hash {
                    *cache = FileCache::new(
                        python_path.clone(),
                        current_hash,
                        module_dependencies.clone(),
                    );
                }
            })
            .or_insert_with(|| {
                // if the cache entry does not exist, create a new one
                FileCache::new(
                    python_path.clone(),
                    current_hash,
                    module_dependencies.clone(),
                )
            });

        // get immutable reference to the cache entry
        let cache = self
            .entries
            .get(python_path)
            .expect("cache entry was just inserted");

        let call_result = FunctionCallResult::from(result.clone());

        let output_mismatch = !hash_mismatch
            && cache
                .function_calls
                .get(identifier)
                .and_then(|calls| calls.iter().find(|call| call.inputs == args))
                .is_some_and(|call| call.output != call_result);

        let allow_output_update = !output_mismatch
            || self.allow_cache_write_on_entry_output_mismatch(python_path, identifier);

        if output_mismatch && !allow_output_update {
            self.record_warning(
                python_path,
                CacheWarningKind::StaleCacheEntryNotUpdated {
                    function_name: identifier.clone(),
                },
            );
        }

        let will_overwrite_output = output_mismatch && allow_output_update;
        if will_overwrite_output {
            self.record_warning(
                python_path,
                CacheWarningKind::OverwriteCacheEntryOnOutputMismatch {
                    function_name: identifier.clone(),
                },
            );
        }

        // get mutable reference to the cache entry
        let cache = self
            .entries
            .get_mut(python_path)
            .expect("cache entry was just inserted");

        // Find the matching function call for the given identifier and
        // arguments (if it exists)
        let cached_function_calls = cache.function_calls.entry(identifier.clone()).or_default();
        let matching_function_call = cached_function_calls
            .iter_mut()
            .find(|call| call.inputs == args);

        // If the function call exists, update it. Otherwise, create a new one.
        if let Some(matching_function_call) = matching_function_call {
            // If the result has changed, update the output and clear the root models.
            if matching_function_call.output != call_result && allow_output_update {
                matching_function_call.output = call_result;
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
                output: call_result,
            });
        }
    }

    /// Loads a cache entry from disk.
    ///
    /// If the cache entry already exists in memory, this does nothing. I/O failures
    /// (e.g. missing file) are ignored. Invalid JSON records a [`CacheWarningKind::InvalidCacheJson`].
    fn load(&mut self, python_path: &PythonPath, module_hash: ImportHash) {
        match self.cache_read_policy {
            CacheReadPolicy::Never => return,
            CacheReadPolicy::Always | CacheReadPolicy::Prompt(_) => (),
        }

        if self.entries.contains_key(python_path) {
            return;
        }

        let cache_path = self.get_cache_path(python_path);
        match FileCache::read_from_path(cache_path) {
            Ok(cache) => {
                // if the hashes are equal or if we allow overwriting the cache, insert the cache entry
                //
                // if overwriting is allowed, the cache entry will be overwritten with a new entry during
                // the next call to `insert`
                if cache.hash == module_hash
                    || !self.allow_cache_write_on_hash_mismatch(python_path)
                {
                    self.entries.insert(python_path.clone(), cache);
                }
            }
            Err(ReadCacheError::Serde(err)) => {
                self.record_warning(
                    python_path,
                    CacheWarningKind::InvalidCacheJson {
                        detail: err.to_string(),
                    },
                );
            }
            Err(ReadCacheError::Io(_)) => (),
        }
    }

    /// Saves all cache entries to disk.
    ///
    /// # Errors
    ///
    /// Returns a vector of [`WriteCacheError`] if the cache files cannot be written.
    fn save_all(&mut self) -> Result<(), Vec<WriteCacheError>> {
        match self.cache_write_policy {
            CacheWritePolicy::Never => return Ok(()),
            CacheWritePolicy::Always | CacheWritePolicy::Prompt(_) => (),
        }

        let mut errors = Vec::new();
        let mut stale_caches = Vec::new();
        for (python_path, cache) in &self.entries {
            if self
                .overwrite_outdated_caches
                .get(python_path)
                .is_none_or(|overwrite| *overwrite)
            {
                let cache_path = self.get_cache_path(python_path);
                match cache.write_to_path(cache_path) {
                    Ok(()) => (),
                    Err(e) => errors.push(e),
                }
            } else {
                stale_caches.push(python_path.clone());
            }
        }

        for python_path in stale_caches {
            self.record_warning(&python_path, CacheWarningKind::StaleCacheNotUpdated);
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

    /// Returns whether a stale cached result may be used on read when the module hash differs.
    fn allow_cache_read_on_hash_mismatch(&self, python_path: &PythonPath) -> bool {
        match &self.cache_read_policy {
            CacheReadPolicy::Always => true,
            CacheReadPolicy::Never => false,
            CacheReadPolicy::Prompt(prompter) => {
                let context = CachePromptContext {
                    python_path: python_path.clone(),
                    function_name: None,
                };

                prompter.prompt(CachePromptKind::UseStaleCacheOnRead, &context)
            }
        }
    }

    /// Returns whether a cache file may be replaced when the module hash differs.
    fn allow_cache_write_on_hash_mismatch(&mut self, python_path: &PythonPath) -> bool {
        match &self.cache_write_policy {
            CacheWritePolicy::Always => true,
            CacheWritePolicy::Never => false,
            CacheWritePolicy::Prompt(prompter) => {
                // if the user has already answered this question, return the answer
                if let Some(overwrite_allowed) = self.overwrite_outdated_caches.get(python_path) {
                    return *overwrite_allowed;
                }

                let context = CachePromptContext {
                    python_path: python_path.clone(),
                    function_name: None,
                };

                let overwrite_allowed =
                    prompter.prompt(CachePromptKind::OverwriteCacheOnHashMismatch, &context);

                // store the answer for future calls
                self.overwrite_outdated_caches
                    .insert(python_path.clone(), overwrite_allowed);

                overwrite_allowed
            }
        }
    }

    /// Returns whether an existing cached output may be replaced.
    fn allow_cache_write_on_entry_output_mismatch(
        &self,
        python_path: &PythonPath,
        identifier: &PyFunctionName,
    ) -> bool {
        match &self.cache_write_policy {
            CacheWritePolicy::Always => true,
            CacheWritePolicy::Never => false,
            CacheWritePolicy::Prompt(prompter) => {
                let context = CachePromptContext {
                    python_path: python_path.clone(),
                    function_name: Some(identifier.clone()),
                };

                prompter.prompt(CachePromptKind::OverwriteCacheOnOutputMismatch, &context)
            }
        }
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

    /// Records a cache warning for the given Python module path.
    fn record_warning(&mut self, python_path: &PythonPath, kind: CacheWarningKind) {
        self.warnings
            .entry(python_path.clone())
            .or_default()
            .insert(CacheWarning {
                python_path: python_path.clone(),
                kind,
            });
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
#[derive(Clone)]
pub enum CacheReadPolicy {
    /// Always read from the cache.
    Always,
    /// Never read from the cache.
    Never,
    /// Ask before reading from the cache.
    Prompt(CachePrompterRef),
}

impl fmt::Debug for CacheReadPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => f.write_str("Always"),
            Self::Never => f.write_str("Never"),
            Self::Prompt(_) => f.write_str("Prompt"),
        }
    }
}

/// When the Python call cache may write to the cache.
#[derive(Clone)]
pub enum CacheWritePolicy {
    /// Always write to the cache.
    Always,
    /// Never write to the cache.
    Never,
    /// Ask before writing to the cache.
    Prompt(CachePrompterRef),
}

impl fmt::Debug for CacheWritePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => f.write_str("Always"),
            Self::Never => f.write_str("Never"),
            Self::Prompt(_) => f.write_str("Prompt"),
        }
    }
}

/// Why the cache is asking for confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePromptKind {
    /// On read: cached module hash differs from the current module hash.
    UseStaleCacheOnRead,
    /// On write: cached module hash differs; the on-disk entry would be replaced.
    OverwriteCacheOnHashMismatch,
    /// On write: cached output differs from the newly evaluated result.
    OverwriteCacheOnOutputMismatch,
}

/// Details shown when prompting about cache use or updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePromptContext {
    /// Python module path for this cache file.
    pub python_path: PythonPath,
    /// Function name, when the prompt is about a specific call.
    pub function_name: Option<PyFunctionName>,
}

/// Asks the user whether to use or update cached Python call results.
pub trait CachePrompter: Send + Sync {
    /// Returns `true` if the user accepts the action described by `kind`.
    fn prompt(&self, kind: CachePromptKind, context: &CachePromptContext) -> bool;
}

/// Shared handle to a [`CachePrompter`].
pub type CachePrompterRef = Arc<dyn CachePrompter>;

/// A non-fatal issue detected while reading or writing the Python call cache.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheWarning {
    /// Python module path this warning applies to.
    pub python_path: PythonPath,
    /// Specific cache warning kind.
    pub kind: CacheWarningKind,
}

/// Kind of non-fatal issue detected while reading or writing the Python call cache.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheWarningKind {
    /// A stale cached result was used because the module hash differs.
    StaleCacheOnRead { function_name: PyFunctionName },
    /// The on-disk cache is out of date and was not updated (write denied or blocked).
    StaleCacheNotUpdated,
    /// A cached function output is out of date and was not updated.
    StaleCacheEntryNotUpdated { function_name: PyFunctionName },
    /// The cache file will be replaced because the module hash differs.
    OverwriteCacheOnHashMismatch,
    /// A cached function output will be replaced.
    OverwriteCacheEntryOnOutputMismatch { function_name: PyFunctionName },
    /// The cache file was read but JSON deserialization failed; it will be replaced.
    InvalidCacheJson { detail: String },
}

impl AsOneilDiagnostic for CacheWarning {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Warning
    }

    fn message(&self) -> String {
        match &self.kind {
            CacheWarningKind::StaleCacheOnRead { function_name } => format!(
                "using outdated cached result for `{}`",
                function_name.as_str()
            ),
            CacheWarningKind::StaleCacheNotUpdated => {
                "Oneil cache is out of date and was not updated".to_string()
            }
            CacheWarningKind::StaleCacheEntryNotUpdated { function_name } => format!(
                "outdated cached result for `{}` was not updated",
                function_name.as_str()
            ),
            CacheWarningKind::OverwriteCacheOnHashMismatch => {
                "overwriting outdated Oneil cache".to_string()
            }
            CacheWarningKind::OverwriteCacheEntryOnOutputMismatch { function_name } => {
                format!(
                    "updating outdated cached result for `{}`",
                    function_name.as_str()
                )
            }
            CacheWarningKind::InvalidCacheJson { detail: _ } => {
                "replacing invalid Oneil cache".to_string()
            }
        }
    }

    fn context(&self) -> Vec<Context> {
        let module_note = Context::Note(format!(
            "cache for python module `{}`",
            self.python_path.as_path().display()
        ));

        match &self.kind {
            CacheWarningKind::StaleCacheOnRead { .. }
            | CacheWarningKind::StaleCacheNotUpdated
            | CacheWarningKind::StaleCacheEntryNotUpdated { .. }
            | CacheWarningKind::OverwriteCacheOnHashMismatch
            | CacheWarningKind::OverwriteCacheEntryOnOutputMismatch { .. } => vec![module_note],
            CacheWarningKind::InvalidCacheJson { detail } => {
                vec![Context::Note(detail.clone()), module_note]
            }
        }
    }
}

#[cfg(test)]
mod python_call_cache_tests {
    use std::sync::{Arc, Mutex};

    use indexmap::{IndexMap, IndexSet};
    use oneil_output::Value;
    use oneil_py_call_cache::FunctionCallResult;
    use oneil_python::function::PythonModule;
    use oneil_shared::{
        error::DiagnosticKind,
        paths::{ModelPath, PythonPath},
        symbols::PyFunctionName,
    };

    use super::*;

    struct TestPrompter {
        responses: Mutex<Vec<bool>>,
        kinds: Mutex<Vec<CachePromptKind>>,
    }

    impl TestPrompter {
        fn new(responses: impl IntoIterator<Item = bool>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().collect()),
                kinds: Mutex::new(Vec::new()),
            }
        }

        fn kinds(&self) -> Vec<CachePromptKind> {
            self.kinds
                .lock()
                .expect("mutex should not be poisoned")
                .clone()
        }
    }

    impl CachePrompter for TestPrompter {
        fn prompt(&self, kind: CachePromptKind, _context: &CachePromptContext) -> bool {
            self.kinds
                .lock()
                .expect("mutex should not be poisoned")
                .push(kind);
            self.responses
                .lock()
                .expect("mutex should not be poisoned")
                .pop()
                .expect("unexpected cache prompt")
        }
    }

    fn python_path() -> PythonPath {
        PythonPath::from_str_no_ext("module")
    }

    fn function_name() -> PyFunctionName {
        PyFunctionName::from("f")
    }

    fn root_model() -> ModelPath {
        ModelPath::from_str_no_ext("model")
    }

    fn file_cache(hash: u64, output: Value) -> FileCache {
        let mut cache = FileCache::new(python_path(), ImportHash::from(hash), BTreeSet::new());
        cache.function_calls.insert(
            function_name(),
            vec![FunctionCall {
                root_models: BTreeSet::new(),
                inputs: vec![Value::Number(oneil_output::Number::Scalar(1.0))],
                output: FunctionCallResult::Success(output),
            }],
        );
        cache
    }

    fn python_module(hash: u64) -> PythonModule {
        PythonModule::new(None, IndexMap::new(), IndexSet::new(), hash)
    }

    #[test]
    fn get_prompt_denied_on_hash_mismatch_returns_none() {
        let test_prompter = Arc::new(TestPrompter::new([false]));
        let mut cache = PythonCallCache::new(
            PathBuf::from("/tmp/test-cache"),
            CacheReadPolicy::Prompt(Arc::<TestPrompter>::clone(&test_prompter)),
            CacheWritePolicy::Never,
        );
        let cached_output = Value::Number(oneil_output::Number::Scalar(2.0));
        cache
            .entries
            .insert(python_path(), file_cache(1, cached_output));

        let result = cache.get(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            2,
        );

        assert!(result.is_none());
        assert_eq!(
            test_prompter.kinds(),
            vec![CachePromptKind::UseStaleCacheOnRead]
        );
    }

    #[test]
    fn get_prompt_accepted_on_hash_mismatch_returns_cached_value() {
        let test_prompter = Arc::new(TestPrompter::new([true]));
        let mut cache = PythonCallCache::new(
            PathBuf::from("/tmp/test-cache"),
            CacheReadPolicy::Prompt(Arc::<TestPrompter>::clone(&test_prompter)),
            CacheWritePolicy::Never,
        );
        let cached_output = Value::Number(oneil_output::Number::Scalar(2.0));
        cache
            .entries
            .insert(python_path(), file_cache(1, cached_output.clone()));

        let result = cache.get(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            2,
        );

        assert_eq!(result, Some(Ok(cached_output)));
        assert_eq!(
            test_prompter.kinds(),
            vec![CachePromptKind::UseStaleCacheOnRead]
        );
    }

    #[test]
    fn insert_prompt_denied_on_hash_mismatch_leaves_cache_unchanged() {
        let test_prompter = Arc::new(TestPrompter::new([false]));
        let mut cache = PythonCallCache::new(
            PathBuf::from("/tmp/test-cache"),
            CacheReadPolicy::Never,
            CacheWritePolicy::Prompt(Arc::<TestPrompter>::clone(&test_prompter)),
        );
        let original = file_cache(1, Value::Number(oneil_output::Number::Scalar(2.0)));
        cache.entries.insert(python_path(), original.clone());

        cache.insert(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            &Ok(Value::Number(oneil_output::Number::Scalar(3.0))),
            &root_model(),
            &python_module(2),
        );

        assert_eq!(cache.entries.get(&python_path()), Some(&original));
        assert_eq!(
            test_prompter.kinds(),
            vec![CachePromptKind::OverwriteCacheOnHashMismatch]
        );
    }

    #[test]
    fn insert_prompt_denied_on_output_mismatch_keeps_cached_output() {
        let test_prompter = Arc::new(TestPrompter::new([false]));
        let mut cache = PythonCallCache::new(
            PathBuf::from("/tmp/test-cache"),
            CacheReadPolicy::Never,
            CacheWritePolicy::Prompt(Arc::<TestPrompter>::clone(&test_prompter)),
        );
        let original_output = Value::Number(oneil_output::Number::Scalar(2.0));
        cache
            .entries
            .insert(python_path(), file_cache(1, original_output.clone()));

        cache.insert(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            &Ok(Value::Number(oneil_output::Number::Scalar(3.0))),
            &root_model(),
            &python_module(1),
        );

        let cached_call = cache
            .entries
            .get(&python_path())
            .and_then(|entry| entry.function_calls.get(&function_name()))
            .and_then(|calls| calls.first())
            .expect("cached call");
        assert_eq!(
            cached_call.output,
            FunctionCallResult::Success(original_output)
        );
        assert!(cached_call.root_models.contains(&root_model()));
        assert_eq!(
            test_prompter.kinds(),
            vec![CachePromptKind::OverwriteCacheOnOutputMismatch]
        );
        let diags = cache.warning_diagnostics();
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0]
                .message()
                .contains("outdated cached result for `f` was not updated")
        );
    }

    #[test]
    fn get_always_on_hash_mismatch_emits_stale_warning_on_cache_path() {
        let cache_dir = PathBuf::from("/tmp/oneil-cache-test-stale-read");
        let mut cache = PythonCallCache::new(
            cache_dir.clone(),
            CacheReadPolicy::Always,
            CacheWritePolicy::Never,
        );
        let cached_output = Value::Number(oneil_output::Number::Scalar(2.0));
        cache
            .entries
            .insert(python_path(), file_cache(1, cached_output.clone()));

        let result = cache.get(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            2,
        );

        assert_eq!(result, Some(Ok(cached_output)));
        let diags = cache.warning_diagnostics();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind(), DiagnosticKind::Warning);
        assert!(
            diags[0]
                .message()
                .contains("using outdated cached result for `f`")
        );
        assert!(diags[0].path().starts_with(&cache_dir));
        assert!(diags[0].path().ends_with("module.json"));
    }

    #[test]
    fn insert_always_on_hash_mismatch_emits_overwrite_warning_on_cache_path() {
        let cache_dir = PathBuf::from("/tmp/oneil-cache-test-hash-overwrite");
        let mut cache = PythonCallCache::new(
            cache_dir.clone(),
            CacheReadPolicy::Never,
            CacheWritePolicy::Always,
        );
        cache.entries.insert(
            python_path(),
            file_cache(1, Value::Number(oneil_output::Number::Scalar(2.0))),
        );

        cache.insert(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            &Ok(Value::Number(oneil_output::Number::Scalar(3.0))),
            &root_model(),
            &python_module(2),
        );

        let diags = cache.warning_diagnostics();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind(), DiagnosticKind::Warning);
        assert!(
            diags[0]
                .message()
                .contains("overwriting outdated Oneil cache")
        );
        assert!(diags[0].path().starts_with(&cache_dir));
        assert!(diags[0].path().ends_with("module.json"));
    }

    #[test]
    fn insert_invalid_cache_json_emits_warning() {
        let cache_dir = PathBuf::from("/tmp/oneil-cache-test-invalid-json");
        let _ = std::fs::remove_dir_all(&cache_dir);
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let cache_file = cache_dir.join("module.json");
        std::fs::write(&cache_file, "{not valid json").expect("write invalid cache file");

        let mut cache =
            PythonCallCache::new(cache_dir, CacheReadPolicy::Always, CacheWritePolicy::Always);

        cache.insert(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            &Ok(Value::Number(oneil_output::Number::Scalar(3.0))),
            &root_model(),
            &python_module(1),
        );

        let diags = cache.warning_diagnostics();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind(), DiagnosticKind::Warning);
        assert!(diags[0].message().contains("replacing invalid Oneil cache"));
        assert_eq!(diags[0].path(), &cache_file);
        let context_notes: Vec<_> = diags[0]
            .context()
            .iter()
            .filter_map(|context| match context {
                Context::Note(note) => Some(note.as_str()),
                Context::Help(_) => None,
            })
            .collect();
        assert_eq!(context_notes.len(), 2);
        assert!(
            context_notes
                .iter()
                .any(|note| note.contains("cache for python module"))
        );
        assert!(
            context_notes
                .iter()
                .any(|note| !note.contains("cache for python module"))
        );
    }

    #[test]
    fn insert_missing_cache_file_does_not_emit_invalid_json_warning() {
        let cache_dir = PathBuf::from("/tmp/oneil-cache-test-missing-json");
        let _ = std::fs::remove_dir_all(&cache_dir);
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let mut cache =
            PythonCallCache::new(cache_dir, CacheReadPolicy::Always, CacheWritePolicy::Always);

        cache.insert(
            &python_path(),
            &function_name(),
            &[Value::Number(oneil_output::Number::Scalar(1.0))],
            &Ok(Value::Number(oneil_output::Number::Scalar(3.0))),
            &root_model(),
            &python_module(1),
        );

        assert!(cache.warning_diagnostics().is_empty());
    }
}
