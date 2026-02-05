use std::path::Path;

use indexmap::IndexMap;
use oneil_eval::output;
use oneil_shared::error::OneilError;

use crate::{cache::EvalCache, output::error::EvalError};

/// A reference to an evaluated model within a model hierarchy.
///
/// This stores a reference to a model and a reference to the
/// entire model collection.
#[derive(Debug, Clone, Copy)]
pub struct ModelReference<'result> {
    model: &'result output::Model,
    eval_cache: &'result EvalCache,
}

impl<'result> ModelReference<'result> {
    /// Creates a new `ModelReference` for the given model and evaluation cache.
    #[must_use]
    pub const fn new(model: &'result output::Model, eval_cache: &'result EvalCache) -> Self {
        Self { model, eval_cache }
    }

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
                    .eval_cache
                    .get(path)
                    .expect("submodel should be visited");

                (
                    name.as_str(),
                    Self {
                        model,
                        eval_cache: self.eval_cache,
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
                    .eval_cache
                    .get(path)
                    .expect("reference should be visited");

                (
                    name.as_str(),
                    Self {
                        model,
                        eval_cache: self.eval_cache,
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

pub struct EvalErrorReference<'result> {
    eval_error: &'result EvalError,
    eval_cache: &'result EvalCache,
}

impl<'result> EvalErrorReference<'result> {
    /// Creates a new `EvalErrorReference` for the given evaluation error and evaluation cache.
    #[must_use]
    pub const fn new(eval_error: &'result EvalError, eval_cache: &'result EvalCache) -> Self {
        Self {
            eval_error,
            eval_cache,
        }
    }

    /// Returns the partial evaluation result for this model, if any.
    #[must_use]
    pub fn partial_result(&self) -> Option<ModelReference<'result>> {
        match self.eval_error {
            EvalError::EvalErrors { partial_result, .. } => {
                Some(ModelReference::new(partial_result, self.eval_cache))
            }
            EvalError::Resolution(_) => panic!("evaluation failed"),
        }
    }

    /// Returns the parameter errors for this model, if any.
    #[must_use]
    pub fn parameter_errors(&self) -> Option<IndexMap<&'result str, Vec<&'result OneilError>>> {
        match self.eval_error {
            EvalError::EvalErrors {
                parameter_errors, ..
            } => Some(
                parameter_errors
                    .iter()
                    .map(|(name, errors)| (name.as_str(), errors.iter().collect()))
                    .collect(),
            ),
            EvalError::Resolution(_) => panic!("evaluation failed"),
        }
    }

    /// Returns the test errors for this model, if any.
    #[must_use]
    pub fn test_errors(&self) -> Option<Vec<&'result OneilError>> {
        match self.eval_error {
            EvalError::EvalErrors { test_errors, .. } => Some(test_errors.iter().collect()),
            EvalError::Resolution(_) => panic!("evaluation failed"),
        }
    }
}
