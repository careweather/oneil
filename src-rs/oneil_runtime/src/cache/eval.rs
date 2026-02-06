//! Cache for evaluated model results from `eval_model`.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_eval::output;

use crate::output::error::EvalError;

/// Result of evaluating a model: either the evaluated [`Model`](output::Model) or an [`EvalError`].
pub type EvalLoadResult = Result<output::Model, EvalError>;

/// Cache of evaluated model results keyed by path.
///
/// Each entry is the result of evaluating that model: either a successfully
/// evaluated [`Model`](output::Model) or an [`EvalError`] (resolution failure or
/// partial result with parameter/test errors).
#[derive(Debug, Default)]
pub struct EvalCache {
    entries: IndexMap<PathBuf, EvalLoadResult>,
}

impl EvalCache {
    /// Creates an empty eval cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached evaluation result for the given `path`, if present.
    ///
    /// Returns the model for `Ok` entries and for `Err(EvalError::EvalErrors { partial_result, .. })`;
    /// returns `None` for `Err(EvalError::Resolution(_))` (no model available).
    pub fn get(&self, path: &Path) -> Option<&output::Model> {
        let r = self.entries.get(path)?;
        match r {
            Ok(m) => Some(m),
            Err(EvalError::EvalErrors { partial_result, .. }) => Some(partial_result),
            Err(EvalError::Resolution(_)) => None,
        }
    }

    /// Returns the cached evaluation error for `path`, if present.
    pub fn get_error(&self, path: &Path) -> Option<&EvalError> {
        let r = self.entries.get(path)?;
        r.as_ref().err()
    }

    /// Returns the full cached entry for `path`.
    pub fn get_entry(&self, path: &Path) -> Option<&EvalLoadResult> {
        self.entries.get(path)
    }

    /// Stores a successfully evaluated `model` for `path`.
    pub fn insert_ok(&mut self, path: PathBuf, model: output::Model) {
        self.entries.insert(path, Ok(model));
    }

    /// Stores an `EvalError` for `path`.
    pub fn insert_err(&mut self, path: PathBuf, error: EvalError) {
        self.entries.insert(path, Err(error));
    }

    /// Returns an iterator over all cached paths and their evaluated model (when present).
    ///
    /// Skips entries that are `Err(EvalError::Resolution(_))` (no model).
    pub fn models_iter(&self) -> impl Iterator<Item = (&PathBuf, &output::Model)> {
        self.entries.iter().filter_map(|(path, r)| match r {
            Ok(m) => Some((path, m)),
            Err(EvalError::EvalErrors { partial_result, .. }) => Some((path, partial_result)),
            Err(EvalError::Resolution(_)) => None,
        })
    }
}
