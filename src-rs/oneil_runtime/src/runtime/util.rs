//! Utility methods for the runtime.

use std::path::PathBuf;

use indexmap::IndexSet;

use super::Runtime;
use crate::cache::{AstCache, EvalCache, IrCache, SourceCache};
#[cfg(feature = "python")]
use crate::cache::PythonImportCache;
use crate::std_builtin::StdBuiltins;

impl Runtime {
    /// Creates a new runtime instance with empty caches.
    #[must_use]
    pub fn new() -> Runtime {
        Self {
            source_cache: SourceCache::new(),
            ast_cache: AstCache::new(),
            ir_cache: IrCache::new(),
            eval_cache: EvalCache::new(),
            #[cfg(feature = "python")]
            python_import_cache: PythonImportCache::new(),
            builtins: StdBuiltins::new(),
        }
    }

    /// Gets the paths to files that the runtime relies on.
    #[must_use]
    pub fn get_watch_paths(&self) -> IndexSet<PathBuf> {
        self.source_cache
            .iter()
            .map(|(path, _)| path.clone())
            .collect()
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
