//! Utility methods for the runtime.

use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};
use oneil_builtins::BuiltinRef;
use oneil_frontend::{CompilationUnit, instance::graph::UnitGraphCache};
use oneil_shared::paths::{ModelPath, SourcePath};

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
    /// If the path is a Python file or a local import of a cached Python module, that module is dropped from the Python import cache.
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

        self.python_import_cache
            .remove_path_and_dependents(path.as_path());
    }

    /// Gets the paths to files that the runtime relies on, including local Python imports.
    #[must_use]
    pub fn get_watch_paths(&self) -> IndexSet<SourcePath> {
        let mut paths: IndexSet<SourcePath> = self.source_cache.paths().cloned().collect();
        for (python_path, result) in self.python_import_cache.iter() {
            paths.insert(SourcePath::from(python_path));
            if let Ok(module) = result {
                for imported in module.get_imports() {
                    paths.insert(SourcePath::new(imported.clone()));
                }
            }
        }
        paths
    }

    /// Gets the models that the runtime has loaded.
    #[must_use]
    pub fn get_loaded_models(&self) -> IndexSet<ModelPath> {
        self.unit_graph_cache
            .keys()
            .map(CompilationUnit::source_path)
            .collect()
    }

    /// Gets the designs that reference a given model path.
    #[must_use]
    #[expect(clippy::missing_panics_doc, reason = "panic enforces an invariant")]
    pub fn get_designs_referencing_model(
        &self,
        param_model_path: &ModelPath,
    ) -> IndexSet<ModelPath> {
        self.get_loaded_models()
            .iter()
            .filter_map(|model| {
                let (model, design_info) = self.get_loaded_model(model);
                let model = model.expect("model must be loaded");

                let (path, _) = design_info
                    .as_ref()?
                    .design_export
                    .as_ref()?
                    .target_model()?;

                (path == param_model_path).then(|| model.path().clone())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CacheReadPolicy, CacheWritePolicy};
    use indexmap::IndexSet;
    use oneil_python::function::PythonModule;
    use oneil_shared::paths::PythonPath;
    use std::path::PathBuf;

    fn runtime() -> Runtime {
        Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never)
    }

    /// Watch paths include local files imported by a cached Python module.
    #[test]
    fn watch_paths_include_python_sibling_imports() {
        let mut runtime = runtime();
        let helpers = PythonPath::from_str_no_ext("lib/helpers");
        let util = PathBuf::from("lib/util.py");
        let mut imports = IndexSet::new();
        imports.insert(util.clone());
        runtime.python_import_cache.insert(
            helpers.clone(),
            Ok(PythonModule::new(
                None,
                indexmap::IndexMap::new(),
                imports,
                0,
            )),
        );

        let watches = runtime.get_watch_paths();
        assert!(watches.contains(&SourcePath::from(&helpers)));
        assert!(watches.contains(&SourcePath::new(util)));
    }

    /// Changing a sibling import drops the cached Python module that imported it.
    #[test]
    fn clearing_a_sibling_import_invalidates_the_importer() {
        let mut runtime = runtime();
        let helpers = PythonPath::from_str_no_ext("lib/helpers");
        let util = PathBuf::from("lib/util.py");
        let mut imports = IndexSet::new();
        imports.insert(util.clone());
        runtime.python_import_cache.insert(
            helpers.clone(),
            Ok(PythonModule::new(
                None,
                indexmap::IndexMap::new(),
                imports,
                0,
            )),
        );

        runtime.clear_non_source_caches(&SourcePath::new(util));
        assert!(runtime.python_import_cache.get_entry(&helpers).is_none());
    }
}
