//! Cache for parsed AST models and associated parse errors.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ast as ast;
use oneil_shared::error::OneilError;

use crate::output::error::ParseError;

/// Result of loading an AST: either a successful model or a [`ParseError`].
pub type AstLoadResult = Result<ast::Model, ParseError>;

/// Cache of parsed AST models keyed by path.
///
/// Each entry is the output of `load_ast`: either a successfully parsed model
/// or a [`ParseError`] (file error or parse errors with partial AST).
#[derive(Debug, Default)]
pub struct AstCache {
    entries: IndexMap<PathBuf, AstLoadResult>,
}

impl AstCache {
    /// Creates an empty AST cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached AST for `path`, if present (success or partial).
    pub fn get(&self, path: &Path) -> Option<&ast::Model> {
        let entry = self.entries.get(path)?;

        match entry {
            Ok(m) => Some(m),
            Err(ParseError::ParseErrors { partial_ast, .. }) => Some(partial_ast),
            Err(ParseError::File(_)) => None,
        }
    }

    /// Returns the cached parse errors for `path`, if present.
    pub fn get_errors(&self, path: &Path) -> Option<&ParseError> {
        let entry = self.entries.get(path)?;
        entry.as_ref().err()
    }

    /// Returns the full cached entry for `path`.
    pub fn get_entry(&self, path: &Path) -> Option<&AstLoadResult> {
        self.entries.get(path)
    }

    /// Stores a successfully parsed `model` for `path`.
    pub fn insert_ok(&mut self, path: PathBuf, model: ast::Model) {
        self.entries.insert(path, Ok(model));
    }

    /// Stores a partial `model` and its `errors` for `path`.
    pub fn insert_err(&mut self, path: PathBuf, model: ast::Model, errors: Vec<OneilError>) {
        self.entries.insert(
            path,
            Err(ParseError::ParseErrors {
                errors,
                partial_ast: Box::new(model),
            }),
        );
    }
}
