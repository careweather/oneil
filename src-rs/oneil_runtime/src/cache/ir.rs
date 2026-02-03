//! Cache for resolved IR models and associated resolution errors.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ir as ir;
use oneil_shared::error::OneilError;

/// Cache of resolved IR models and per-file resolution errors.
#[derive(Debug, Default)]
pub struct IrCache {
    /// Resolved IR models keyed by path.
    models: IndexMap<PathBuf, ir::Model>,
    /// Resolution errors for a path, when resolution produced errors.
    errors: IndexMap<PathBuf, Vec<OneilError>>,
}

impl IrCache {
    /// Creates an empty IR cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached IR model for `path`, if present.
    pub fn get(&self, path: &Path) -> Option<&ir::Model> {
        self.models.get(path)
    }

    /// Returns the cached resolution errors for `path`, if present.
    pub fn get_errors(&self, path: &Path) -> Option<&[OneilError]> {
        self.errors.get(path).map(Vec::as_slice)
    }

    /// Stores a resolved `model` for `path`.
    pub fn insert(&mut self, path: PathBuf, model: ir::Model) {
        self.models.insert(path, model);
    }

    /// Stores resolution `errors` for `path`.
    pub fn insert_errors(&mut self, path: PathBuf, errors: Vec<OneilError>) {
        self.errors.insert(path, errors);
    }
}
