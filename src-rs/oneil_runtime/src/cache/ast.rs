//! Cache for parsed AST models and associated parse errors.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_shared::error::OneilError;

/// Cache of parsed AST models and per-file parse errors (e.g. partial result + errors).
#[derive(Debug, Default)]
pub struct AstCache {
    /// Successfully or partially parsed models keyed by path.
    models: IndexMap<PathBuf, ast::Model>,
    /// Parse errors for a path, when parsing produced errors (possibly with a partial AST).
    errors: IndexMap<PathBuf, Vec<OneilError>>,
}

impl AstCache {
    /// Creates an empty AST cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached AST for `path`, if present (success or partial).
    pub fn get(&self, path: &Path) -> Option<&ast::Model> {
        self.models.get(path)
    }

    /// Returns the cached parse errors for `path`, if present.
    pub fn get_errors(&self, path: &Path) -> Option<&[OneilError]> {
        self.errors.get(path).map(Vec::as_slice)
    }

    /// Stores a successfully parsed `model` for `path`.
    pub fn insert_ok(&mut self, path: PathBuf, model: ast::Model) {
        self.models.insert(path, model);
    }

    /// Stores a partial `model` and its `errors` for `path`.
    pub fn insert_err(&mut self, path: PathBuf, model: ast::Model, errors: Vec<OneilError>) {
        self.models.insert(path.clone(), model);
        self.errors.insert(path, errors);
    }
}
