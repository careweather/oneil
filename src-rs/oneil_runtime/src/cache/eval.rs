//! Cache for evaluated model results from `eval_model`.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use oneil_eval::output;
use oneil_shared::error::OneilError;

/// Cache of evaluated model results keyed by the root path passed to `eval_model`.
///
/// Each entry stores the full result of evaluating a model and its dependencies:
/// a map from each model path to its evaluated [`Model`] and any errors that occurred.
#[derive(Debug, Default)]
pub struct EvalCache {
    /// Evaluated model results keyed by the root path used in `eval_model`.
    results: IndexMap<PathBuf, output::Model>,
    errors: IndexMap<PathBuf, Vec<OneilError>>,
}

impl EvalCache {
    /// Creates an empty eval cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the cached evaluation result for the given root `path`, if present.
    ///
    /// The returned map associates each model path (root and dependencies) with its evaluated result.
    pub fn get(&self, path: &Path) -> Option<ModelReference<'_>> {
        self.results.get(path).map(|model| ModelReference {
            model,
            model_collection: &self.results,
        })
    }

    /// Returns the cached evaluation errors for the given `path`, if present.
    pub fn get_errors(&self, path: &Path) -> Option<&[OneilError]> {
        self.errors.get(path).map(Vec::as_slice)
    }

    /// Stores the evaluation `result` for each model.
    pub fn insert_all(&mut self, models: impl IntoIterator<Item = (PathBuf, output::Model)>) {
        self.results.extend(models);
    }

    /// Stores the evaluation `errors` for each model.
    pub fn insert_errors(&mut self, models: impl IntoIterator<Item = (PathBuf, Vec<OneilError>)>) {
        self.errors.extend(models);
    }
}

/// A reference to an evaluated model within a model hierarchy.
///
/// This stores a reference to a model and a reference to the
/// entire model collection.
#[derive(Debug, Clone, Copy)]
pub struct ModelReference<'result> {
    model: &'result output::Model,
    model_collection: &'result IndexMap<PathBuf, output::Model>,
}

impl<'result> ModelReference<'result> {
    /// Returns the file path of this model.
    #[must_use]
    pub fn path(&self) -> &'result Path {
        self.model.path.as_path()
    }

    /// Returns a map of submodel names to their model references.
    ///
    /// # Panics
    ///
    /// Panics if any submodel has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn submodels(&self) -> IndexMap<&'result str, Self> {
        self.model
            .submodels
            .iter()
            .map(|(name, path)| {
                let model = self
                    .model_collection
                    .get(path)
                    .expect("submodel should be visited");

                (
                    name.as_str(),
                    Self {
                        model,
                        model_collection: self.model_collection,
                    },
                )
            })
            .collect()
    }

    /// Returns a map of reference names to their model references.
    ///
    /// # Panics
    ///
    /// Panics if any reference has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn references(&self) -> IndexMap<&'result str, Self> {
        self.model
            .references
            .iter()
            .map(|(name, path)| {
                let model = self
                    .model_collection
                    .get(path)
                    .expect("reference should be visited");

                (
                    name.as_str(),
                    Self {
                        model,
                        model_collection: self.model_collection,
                    },
                )
            })
            .collect()
    }

    /// Returns a map of parameter names to their evaluated parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'result str, &'result output::Parameter> {
        self.model
            .parameters
            .iter()
            .map(|(name, parameter)| (name.as_str(), parameter))
            .collect()
    }

    /// Returns the list of evaluated test results for this model.
    #[must_use]
    pub fn tests(&self) -> Vec<&'result output::Test> {
        self.model.tests.iter().collect()
    }
}
