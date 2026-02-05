//! Cache for evaluated model results from `eval_model`.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_eval::{error, output};
use oneil_shared::error::OneilError;

/// Cache of evaluated model results keyed by the root path passed to `eval_model`.
///
/// Each entry stores the full result of evaluating a model and its dependencies:
/// a map from each model path to its evaluated [`Model`] and any errors that occurred.
#[derive(Debug, Default)]
pub struct EvalCache {
    /// Evaluated model results keyed by the root path used in `eval_model`.
    results: IndexMap<PathBuf, output::Model>,
    errors: IndexMap<PathBuf, error::ModelErrors<OneilError>>,
}

impl EvalCache {
    /// Creates an empty eval cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached evaluation result for the given root `path`, if present.
    ///
    /// The returned map associates each model path (root and dependencies) with its evaluated result.
    pub fn get(&self, path: &Path) -> Option<&output::Model> {
        self.results.get(path)
    }

    /// Returns the cached evaluation errors for the given `path`, if present.
    pub fn get_errors(&self, path: &Path) -> Option<&error::ModelErrors<OneilError>> {
        self.errors.get(path)
    }

    /// Stores the evaluation `result` for each model.
    pub fn insert_all(&mut self, models: impl IntoIterator<Item = (PathBuf, output::Model)>) {
        self.results.extend(models);
    }

    /// Stores the evaluation `errors` for each model.
    pub fn insert_errors(
        &mut self,
        models: impl IntoIterator<Item = (PathBuf, error::ModelErrors<OneilError>)>,
    ) {
        self.errors.extend(models);
    }

    /// Returns an iterator over all cached model paths and their evaluated results.
    pub fn models_iter(&self) -> impl Iterator<Item = (&PathBuf, &output::Model)> {
        self.results.iter()
    }
}
