//! Cache for evaluated model results from `eval_model`.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_eval::Model;

/// Cache of evaluated model results keyed by the root path passed to `eval_model`.
///
/// Each entry stores the full result of evaluating a model and its dependencies:
/// a map from each model path to its evaluated [`Model`].
#[derive(Debug, Default)]
pub struct EvalCache {
    /// Evaluated model results keyed by the root path used in `eval_model`.
    results: IndexMap<PathBuf, Model>,
}

impl EvalCache {
    /// Creates an empty eval cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached evaluation result for the given root `path`, if present.
    ///
    /// The returned map associates each model path (root and dependencies) with its evaluated result.
    pub fn get(&self, path: &Path) -> Option<&Model> {
        self.results.get(path)
    }

    /// Stores the evaluation `result` for the given root `path`.
    pub fn insert_all(&mut self, models: impl IntoIterator<Item = (PathBuf, Model)>) {
        self.results.extend(models);
    }
}
