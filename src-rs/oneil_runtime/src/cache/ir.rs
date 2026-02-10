//! Cache for resolved IR models and associated resolution errors.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_ir as ir;

use crate::output::error::ResolutionError;

/// Result of loading IR for a model: either the resolved model or a [`ResolutionError`].
pub type IrLoadResult = Result<ir::Model, ResolutionError>;

/// Cache of resolved IR models keyed by path.
///
/// Each entry is the result of loading that model's IR: either a successfully
/// resolved model or a [`ResolutionError`] (parse or resolution errors).
#[derive(Debug, Default)]
pub struct IrCache {
    entries: IndexMap<PathBuf, IrLoadResult>,
}

impl IrCache {
    /// Creates an empty IR cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached IR model for `path`, if present.
    ///
    /// Returns the model for `Ok` entries and for `Err(ResolutionError::ResolutionErrors { partial_ir, .. })`;
    /// returns `None` for `Err(ResolutionError::Parse(_))` (no IR available).
    pub fn get(&self, path: &Path) -> Option<&ir::Model> {
        let entry = self.entries.get(path)?;
        match entry {
            Ok(m) => Some(m),
            Err(ResolutionError::ResolutionErrors { partial_ir, .. }) => Some(partial_ir),
            Err(ResolutionError::Parse(_error)) => None,
        }
    }

    /// Returns the cached resolution error for `path`, if present.
    pub fn get_error(&self, path: &Path) -> Option<&ResolutionError> {
        let r = self.entries.get(path)?;
        r.as_ref().err()
    }

    /// Returns the full cached entry for `path`.
    pub fn get_entry(&self, path: &Path) -> Option<&IrLoadResult> {
        self.entries.get(path)
    }

    /// Stores a successfully resolved `model` for `path`.
    pub fn insert_ok(&mut self, path: PathBuf, model: ir::Model) {
        self.entries.insert(path, Ok(model));
    }

    /// Stores a `ResolutionError` for `path`.
    pub fn insert_err(&mut self, path: PathBuf, error: ResolutionError) {
        self.entries.insert(path, Err(error));
    }

    /// Returns an iterator over the paths and models in the cache.
    pub fn models_iter_maybe_partial(&self) -> impl Iterator<Item = (&PathBuf, &ir::Model)> {
        self.entries
            .iter()
            .filter_map(|(path, result)| match result {
                Ok(model) => Some((path, model)),
                Err(ResolutionError::ResolutionErrors { partial_ir, .. }) => {
                    Some((path, partial_ir))
                }
                Err(ResolutionError::Parse(_)) => None,
            })
    }
}
