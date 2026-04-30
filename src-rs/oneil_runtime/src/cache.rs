//! Generic path-keyed cache using [`LoadResult`], and a source cache for raw file contents.

use std::hash::{DefaultHasher, Hash, Hasher};

use indexmap::IndexMap;
use oneil_parser::error::ParserError;
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::{ModelPath, SourcePath},
};

use crate::{error::SourceError, output};

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

#[cfg(feature = "python")]
pub use python::{PythonCallCache, PythonCallCacheRecord, PythonImportCache};

#[cfg(feature = "python")]
mod python {
    use std::path::{Component, PathBuf};

    use indexmap::IndexMap;
    use oneil_py_call_cache::ImportEntry;
    use oneil_py_call_cache::{
        FileCache, FunctionCall, FunctionCallResult, ImportHash, ReadCacheError, WriteCacheError,
    };
    use oneil_python::{PythonEvalError, function::PythonModule};
    use oneil_shared::paths::ModelPath;
    use oneil_shared::{
        paths::PythonPath,
        symbols::{ParameterName, PyFunctionName, TestIndex},
    };

    use crate::error::PythonImportError;
    use crate::output;

    /// Inputs shared by [`PythonCallCache::add_parameter_entry`] and [`PythonCallCache::add_test_entry`].
    #[derive(Debug)]
    pub struct PythonCallCacheRecord<'a> {
        /// Model file whose cache entry is updated.
        pub model_path: &'a ModelPath,
        /// Path of the Python module that defined the callee.
        pub python_path: &'a PythonPath,
        /// Name of the invoked Python function.
        pub function_name: &'a PyFunctionName,
        /// Argument values passed to the call.
        pub args: &'a [output::Value],
        /// Evaluation outcome to persist.
        pub eval_result: Result<output::Value, PythonEvalError>,
        /// Loaded module metadata for import tracking.
        pub python_module: &'a PythonModule,
    }

    /// Whether a recorded call belongs to a parameter default or a test body.
    #[derive(Debug, Clone, Copy)]
    enum CallCacheTarget<'a> {
        Parameter(&'a ParameterName),
        Test(TestIndex),
    }
    /// Cache for Python import function maps keyed by path.
    ///
    /// Stores a [`Result`] per path: either the loaded [`PythonFunctionMap`] or a
    /// [`PythonImportError`](crate::error::PythonImportError) when loading failed.
    #[cfg(feature = "python")]
    #[derive(Debug)]
    pub struct PythonImportCache {
        entries:
            IndexMap<PythonPath, Result<oneil_python::function::PythonModule, PythonImportError>>,
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
    #[cfg(feature = "python")]
    #[derive(Debug)]
    pub struct PythonCallCache {
        cache_dir: PathBuf,
        entries: IndexMap<ModelPath, FileCache>,
    }

    #[cfg(feature = "python")]
    impl PythonCallCache {
        /// Creates an empty Python call cache.
        #[must_use]
        pub fn new(cache_dir: PathBuf) -> Self {
            Self {
                cache_dir,
                entries: IndexMap::new(),
            }
        }

        /// Clears the cache.
        pub fn clear(&mut self) {
            self.entries.clear();
        }

        /// Merges another cache into this one.
        ///
        /// If there are conflicting entries, the entries in the other cache are preferred.
        pub fn merge(&mut self, other: Self) {
            self.entries.extend(other.entries);
        }

        /// Returns the cached entry for `parameter` in `model_path`, if present.
        ///
        /// If the cache entry has not been loaded yet, it is loaded from disk.
        ///
        /// # Errors
        ///
        /// Returns [`ReadCacheError`] if the cache file cannot be read.
        pub fn get_parameter_entry(
            &mut self,
            model_path: &ModelPath,
            parameter_name: &ParameterName,
        ) -> Option<&[FunctionCall]> {
            self.load(model_path).ok()?;

            let entry = self.entries.get(model_path)?;
            entry.parameters.get(parameter_name).map(Vec::as_slice)
        }

        pub fn get_test_entry(
            &mut self,
            model_path: &ModelPath,
            test_index: TestIndex,
        ) -> Option<&[FunctionCall]> {
            self.load(model_path).ok()?;

            let entry = self.entries.get(model_path)?;
            entry.tests.get(&test_index).map(Vec::as_slice)
        }

        /// Appends one cached function call for `parameter_name` and updates import usage.
        pub fn add_parameter_entry(
            &mut self,
            record: PythonCallCacheRecord<'_>,
            parameter_name: &ParameterName,
        ) {
            self.push_function_call_entry(record, CallCacheTarget::Parameter(parameter_name));
        }

        /// Appends one cached function call for `test_index` and updates import usage.
        pub fn add_test_entry(&mut self, record: PythonCallCacheRecord<'_>, test_index: TestIndex) {
            self.push_function_call_entry(record, CallCacheTarget::Test(test_index));
        }

        /// Appends one [`FunctionCall`] under `target` and registers `function_name` on the matching import entry.
        fn push_function_call_entry(
            &mut self,
            record: PythonCallCacheRecord<'_>,
            target: CallCacheTarget<'_>,
        ) {
            let PythonCallCacheRecord {
                model_path,
                python_path,
                function_name,
                args,
                eval_result,
                python_module,
            } = record;

            let model_entry = self.entries.entry(model_path.clone()).or_default();
            let cached_function_call = function_call_from(function_name, args, eval_result);

            match target {
                CallCacheTarget::Parameter(parameter_name) => {
                    model_entry
                        .parameters
                        .entry(parameter_name.clone())
                        .or_default()
                        .push(cached_function_call);
                }
                CallCacheTarget::Test(test_index) => {
                    model_entry
                        .tests
                        .entry(test_index)
                        .or_default()
                        .push(cached_function_call);
                }
            }

            model_entry
                .imports
                .entry(python_path.clone())
                .or_insert_with(|| make_python_cache_import_entry(python_path, python_module))
                .functions_used
                .insert(function_name.clone());
        }

        /// Loads a cache entry from disk.
        ///
        /// If the cache entry already exists, this does nothing.
        ///
        /// # Errors
        ///
        /// Returns [`ReadCacheError`] if the cache file cannot be read.
        fn load(&mut self, model_path: &ModelPath) -> Result<(), ReadCacheError> {
            if self.entries.contains_key(model_path) {
                return Ok(());
            }

            let cache_relative_path = get_cache_relative_path(model_path);
            let cache_path = self.cache_dir.join(cache_relative_path);
            let cache = FileCache::read_from_path(cache_path)?;
            self.entries.insert(model_path.clone(), cache);
            Ok(())
        }

        /// Saves all cache entries to disk.
        ///
        /// # Errors
        ///
        /// Returns a vector of [`WriteCacheError`] if the cache files cannot be written.
        pub fn save_all(&self) -> Result<(), Vec<WriteCacheError>> {
            let mut errors = Vec::new();
            for (model_path, cache) in &self.entries {
                let cache_relative_path = get_cache_relative_path(model_path);
                let cache_path = self.cache_dir.join(cache_relative_path);
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
    }

    fn get_cache_relative_path(model_path: &ModelPath) -> PathBuf {
        model_path
            .as_path()
            .with_extension("json")
            .components()
            // convert to a path that can be used in the cache directory
            .fold(PathBuf::new(), append_normalized_component)
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

    fn function_call_from(
        function_name: &PyFunctionName,
        args: &[output::Value],
        eval_result: Result<output::Value, PythonEvalError>,
    ) -> FunctionCall {
        FunctionCall {
            function: function_name.clone(),
            inputs: args.to_vec(),
            output: FunctionCallResult::from(eval_result),
        }
    }

    fn make_python_cache_import_entry(
        python_path: &PythonPath,
        python_module: &PythonModule,
    ) -> ImportEntry {
        let name = python_path
            .as_path()
            .file_stem()
            .map_or_else(|| "<unknown>".into(), |s| s.to_string_lossy().to_string());

        let dependencies = python_module
            .get_imports()
            .iter()
            .map(|path| path.display().to_string())
            .collect();

        let hash = ImportHash::from(python_module.get_hash());

        ImportEntry::new(name, dependencies, hash)
    }
}
