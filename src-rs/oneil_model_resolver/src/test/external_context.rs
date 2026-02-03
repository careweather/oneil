//! Test implementation of [`ExternalResolutionContext`].
//!
//! Provides a configurable external context for tests that need builtin
//! lookups, model AST loading, and/or Python import validation.
//! [`load_ast`](ExternalResolutionContext::load_ast) returns ASTs registered via
//! [`with_model_asts`](Self::with_model_asts); otherwise returns `Err(())`.
//! [`load_python_import`](ExternalResolutionContext::load_python_import) returns
//! `Ok(())` only for paths registered via [`with_python_imports_ok`](Self::with_python_imports_ok).

use std::collections::HashSet;
use std::path::PathBuf;

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_ir as ir;

use crate::util::{
    AstLoadingFailedError, ExternalResolutionContext, PythonImportLoadingFailedError,
};

/// Test double for [`ExternalResolutionContext`].
///
/// Configurable builtin values, builtin functions, model ASTs (via
/// [`with_model_asts`](Self::with_model_asts)), and Python import paths (via
/// [`with_python_imports_ok`](Self::with_python_imports_ok)).
#[derive(Debug, Default)]
pub struct TestExternalContext {
    builtin_variables: HashSet<String>,
    builtin_functions: HashSet<String>,

    /// Model path -> AST; paths are derived from the given path's stem (e.g. "test.on" -> ModelPath("test.on")).
    model_asts: IndexMap<ir::ModelPath, ast::Model>,

    /// Python paths for which `load_python_import` should return `Ok(())`.
    /// Stored as path strings (e.g. `my_python` or `subdir/my_python`) that are
    /// compared against the path stem before the `.py` extension when used with
    /// [`ModelPath::get_sibling_path`]; the resolver then builds [`ir::PythonPath`] from that.
    python_imports_ok: HashSet<PathBuf>,
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

    /// Registers Python paths for which `load_python_import` should return `Ok(())`.
    ///
    /// Paths are compared against the resolved Python path (as from
    /// `model_path.get_sibling_path(import_path)` with `.py` extension). Use the
    /// same path strings as in the import (e.g. `"my_python"`, `"subdir/my_python"`).
    #[must_use]
    pub fn with_python_imports_ok(
        mut self,
        paths: impl IntoIterator<Item = impl AsRef<std::path::Path>>,
    ) -> Self {
        for p in paths {
            let mut path = p.as_ref().to_path_buf();
            path.set_extension("py");
            self.python_imports_ok.insert(path);
        }
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
    ) -> Result<(), PythonImportLoadingFailedError> {
        if self.python_imports_ok.contains(python_path.as_ref()) {
            Ok(())
        } else {
            Err(PythonImportLoadingFailedError)
        }
    }
}
