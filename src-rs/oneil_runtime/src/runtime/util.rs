//! Utility methods for the runtime.

use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};
use oneil_builtins::BuiltinRef;
use oneil_frontend::{CompilationUnit, instance::graph::UnitGraphCache};
use oneil_shared::paths::{ModelPath, PythonPath, SourcePath};

use super::Runtime;
use crate::{
    CacheReadPolicy, CacheWritePolicy,
    cache::{AstCache, EvalCache, PythonCallCache, PythonImportCache, SourceCache},
};

impl Runtime {
    /// Creates a new runtime instance with empty caches.
    #[must_use]
    pub fn new(cache_read_policy: CacheReadPolicy, cache_write_policy: CacheWritePolicy) -> Self {
        let cache_dir = PathBuf::from("__oncache__");

        Self {
            source_cache: SourceCache::new(),
            ast_cache: AstCache::new(),
            unit_graph_cache: UnitGraphCache::new(),
            design_info: IndexMap::new(),
            eval_cache: EvalCache::new(),
            composed_graph: None,
            python_import_cache: PythonImportCache::new(),
            python_call_cache: PythonCallCache::new(
                cache_dir,
                cache_read_policy,
                cache_write_policy,
            ),
            builtins: BuiltinRef::new(),
        }
    }

    /// Clears the runtime's caches for a given path.
    ///
    /// If the path is a model path (`.on`), clears the AST, `InstanceGraph`s for compilation units, and eval caches for that path.
    /// If the path is a Python path (`.py`), clears the Python import cache for that path.
    ///
    /// This does not clear the source cache.
    pub fn clear_non_source_caches(&mut self, path: &SourcePath) {
        if let Ok(model_path) = ModelPath::try_from(path.clone()) {
            self.ast_cache.remove(&model_path);
        }
        // Any file change can invalidate transitive dependents, so the entire
        // eval cache must be cleared rather than just the changed path's entries.
        self.eval_cache.clear();
        self.unit_graph_cache.clear();
        self.design_info.clear();
        // Drop the composed graph too: spans inside it may no longer line up
        // with the new source, and any leftover diagnostics would mislead
        // users until the next eval re-composes.
        self.composed_graph = None;

        if let Ok(python_path) = PythonPath::try_from(path.clone()) {
            self.python_import_cache.remove(&python_path);
        }
    }

    /// Gets the paths to files that the runtime relies on.
    #[must_use]
    pub fn get_watch_paths(&self) -> IndexSet<SourcePath> {
        self.source_cache.paths().cloned().collect()
    }

    /// Gets the models that the runtime has loaded.
    #[must_use]
    pub fn get_loaded_models(&self) -> IndexSet<ModelPath> {
        self.unit_graph_cache
            .keys()
            .map(CompilationUnit::source_path)
            .collect()
    }
}
