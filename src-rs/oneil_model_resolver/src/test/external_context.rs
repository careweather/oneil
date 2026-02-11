//! Test implementation of [`ExternalResolutionContext`].
//!
//! Provides a configurable external context for tests that need builtin
//! lookups, model AST loading, and/or Python import validation.
//! [`load_ast`](ExternalResolutionContext::load_ast) returns ASTs registered via
//! [`with_model_asts`](Self::with_model_asts); otherwise returns `Err(())`.
//! [`load_python_import`](ExternalResolutionContext::load_python_import) returns
//! `Ok(&IndexSet<String>)` (the set of function names) for paths registered via
//! [`with_python_imports_ok`](Self::with_python_imports_ok); use
//! [`with_python_import_functions`](Self::with_python_import_functions) to set the function list.

use std::collections::HashSet;
use std::path::PathBuf;

use indexmap::{IndexMap, IndexSet};
use oneil_ast as ast;
use oneil_ir as ir;

use crate::util::{
    AstLoadingFailedError, ExternalResolutionContext, PythonImportLoadingFailedError,
};

/// Test double for [`ExternalResolutionContext`].
///
/// Configurable builtin values, builtin functions, model ASTs (via
/// [`with_model_asts`](Self::with_model_asts)), and Python import paths with
/// their function lists (via [`with_python_imports_ok`](Self::with_python_imports_ok) and
/// [`with_python_import_functions`](Self::with_python_import_functions)).
#[derive(Debug, Default)]
pub struct TestExternalContext {
    /// Builtin variables that are valid.
    builtin_variables: HashSet<String>,

    /// Builtin functions that are valid.
    builtin_functions: HashSet<String>,

    /// Model path -> AST; paths are derived from the given path's stem (e.g. "test.on" -> ModelPath("test.on")).
    model_asts: IndexMap<ir::ModelPath, ast::Model>,

    /// Python path (with `.py` extension) -> set of callable function names returned by `load_python_import`.
    python_imports: IndexMap<PathBuf, IndexSet<String>>,
}

impl TestExternalContext {
    /// Creates a new empty test external context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the given identifiers as builtin values.
    #[must_use]
    pub fn with_builtin_variables(
        mut self,
        variables: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.builtin_variables
            .extend(variables.into_iter().map(ToString::to_string));

        self
    }

    /// Registers the given identifiers as builtin functions.
    #[must_use]
    pub fn with_builtin_functions(
        mut self,
        functions: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.builtin_functions
            .extend(functions.into_iter().map(ToString::to_string));

        self
    }

    /// Registers model ASTs for [`load_ast`](ExternalResolutionContext::load_ast).
    ///
    /// Each path is normalized to a [`ModelPath`](ir::ModelPath) using the path's
    /// stem (e.g. `"test.on"` or `"test"` -> `ModelPath("test.on")`). When the
    /// resolver calls `load_ast` for that path, the corresponding AST is returned.
    #[must_use]
    pub fn with_model_asts(
        mut self,
        models: impl IntoIterator<Item = (impl AsRef<std::path::Path>, ast::Model)>,
    ) -> Self {
        for (path, model) in models {
            let path = path.as_ref().to_path_buf();
            let stem = path.file_stem().map_or_else(|| path.clone(), PathBuf::from);
            self.model_asts.insert(ir::ModelPath::new(stem), model);
        }
        self
    }

    /// Registers Python paths for which `load_python_import` should return `Ok` with an empty function set.
    ///
    /// Paths are compared against the resolved Python path (as from
    /// `model_path.get_sibling_path(import_path)` with `.py` extension). Use the
    /// same path strings as in the import (e.g. `"my_python"`, `"subdir/my_python"`).
    /// Use [`with_python_import_functions`](Self::with_python_import_functions) to set the function list for a path.
    #[must_use]
    pub fn with_python_imports_ok(
        mut self,
        paths: impl IntoIterator<Item = impl AsRef<std::path::Path>>,
    ) -> Self {
        for p in paths {
            let mut path = p.as_ref().to_path_buf();
            path.set_extension("py");
            self.python_imports.insert(path, IndexSet::new());
        }
        self
    }

    /// Registers the set of function names for a Python import path.
    ///
    /// The path is normalized with a `.py` extension to match how the resolver looks it up.
    /// If the path was not already registered (e.g. via [`with_python_imports_ok`](Self::with_python_imports_ok)),
    /// it is added.
    #[must_use]
    pub fn with_python_import_functions(
        mut self,
        path: impl AsRef<std::path::Path>,
        functions: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        let mut path = path.as_ref().to_path_buf();
        path.set_extension("py");
        let set: IndexSet<String> = functions
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        self.python_imports.insert(path, set);
        self
    }
}

impl ExternalResolutionContext for TestExternalContext {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.builtin_variables.contains(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.builtin_functions.contains(identifier.as_str())
    }

    fn load_ast(&mut self, path: &ir::ModelPath) -> Result<&ast::Model, AstLoadingFailedError> {
        self.model_asts.get(path).ok_or(AstLoadingFailedError)
    }

    fn load_python_import(
        &mut self,
        python_path: &ir::PythonPath,
    ) -> Result<IndexSet<String>, PythonImportLoadingFailedError> {
        self.python_imports
            .get(python_path.as_ref())
            .cloned()
            .ok_or(PythonImportLoadingFailedError)
    }
}
