//! References into evaluation results and error contexts.
//!
//! Provides [`ModelReference`] for navigating evaluated models,
//! [`EvalErrorReference`] for inspecting evaluation failures,
//! [`ModelIrReference`] for navigating resolved IR models, and
//! [`ResolutionErrorReference`] for inspecting resolution failures.

use std::path::Path;

use indexmap::IndexMap;
use oneil_eval::output;
use oneil_ir as ir;
use oneil_shared::error::OneilError;

use crate::{
    cache::{EvalCache, IrCache},
    output::error::{EvalError, ResolutionError},
};

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

    /// Returns a map of submodel names to their model references or evaluation errors.
    ///
    /// # Panics
    ///
    /// Panics if any submodel has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn submodels(&self) -> IndexMap<&'result str, Result<Self, EvalErrorReference<'result>>> {
        self.model
            .submodels
            .iter()
            .map(|(name, path)| {
                let entry = self
                    .eval_cache
                    .get_entry(path)
                    .expect("submodel should be in cache");

                let result = entry
                    .as_ref()
                    .map(|model| Self {
                        model,
                        eval_cache: self.eval_cache,
                    })
                    .map_err(|eval_error| EvalErrorReference::new(eval_error, self.eval_cache));

                (name.as_str(), result)
            })
            .collect()
    }

    /// Returns a map of reference names to their model references or evaluation errors.
    ///
    /// # Panics
    ///
    /// Panics if any reference has not been visited and
    /// added to the model collection. This should never be
    /// the case as long as creating the `EvalResult`
    /// resolves successfully.
    #[must_use]
    pub fn references(&self) -> IndexMap<&'result str, Result<Self, EvalErrorReference<'result>>> {
        self.model
            .references
            .iter()
            .map(|(name, path)| {
                let entry = self
                    .eval_cache
                    .get_entry(path)
                    .expect("reference should be in cache");
                let result = entry
                    .as_ref()
                    .map(|model| Self {
                        model,
                        eval_cache: self.eval_cache,
                    })
                    .map_err(|eval_error| EvalErrorReference::new(eval_error, self.eval_cache));
                (name.as_str(), result)
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

/// A reference to an evaluation error within a model hierarchy.
///
/// Allows inspecting partial results and error details (e.g. parameter
/// or test errors) when evaluation of a model fails.
#[derive(Debug, Clone, Copy)]
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

/// A reference to a resolved IR model within a model hierarchy.
///
/// This stores a reference to an IR model, the path it was loaded from,
/// and a reference to the IR cache.
#[derive(Debug, Clone, Copy)]
pub struct ModelIrReference<'result> {
    model: &'result ir::Model,
    ir_cache: &'result IrCache,
    path: &'result Path,
}

impl<'result> ModelIrReference<'result> {
    /// Creates a new `ModelIrReference` for the given model, IR cache, and path.
    #[must_use]
    pub const fn new(
        model: &'result ir::Model,
        ir_cache: &'result IrCache,
        path: &'result Path,
    ) -> Self {
        Self {
            model,
            ir_cache,
            path,
        }
    }

    /// Returns the file path of this model.
    #[must_use]
    pub const fn path(&self) -> &'result Path {
        self.path
    }

    /// Returns a map of submodel names to their IR model references or resolution errors.
    ///
    /// # Panics
    ///
    /// Panics if any submodel's reference has not been visited and
    /// added to the IR cache.
    #[must_use]
    pub fn submodels(
        &self,
    ) -> IndexMap<&'result str, Result<Self, ResolutionErrorReference<'result>>> {
        self.model
            .get_submodels()
            .iter()
            .map(|(name, submodel_import)| {
                let ref_import = self
                    .model
                    .get_reference(submodel_import.reference_name())
                    .expect("submodel should have reference");
                let path = ref_import.path().as_ref();
                let entry = self
                    .ir_cache
                    .get_entry(path)
                    .expect("submodel should be in cache");
                let result = entry
                    .as_ref()
                    .map(|model| Self {
                        model,
                        ir_cache: self.ir_cache,
                        path,
                    })
                    .map_err(|resolution_error| {
                        ResolutionErrorReference::new(resolution_error, self.ir_cache, path)
                    });
                (name.as_str(), result)
            })
            .collect()
    }

    /// Returns a map of reference names to their IR model references or resolution errors.
    ///
    /// # Panics
    ///
    /// Panics if any reference has not been visited and
    /// added to the IR cache.
    #[must_use]
    pub fn references(
        &self,
    ) -> IndexMap<&'result str, Result<Self, ResolutionErrorReference<'result>>> {
        self.model
            .get_references()
            .iter()
            .map(|(name, ref_import)| {
                let path = ref_import.path().as_ref();
                let entry = self
                    .ir_cache
                    .get_entry(path)
                    .expect("reference should be in cache");
                let result = entry
                    .as_ref()
                    .map(|model| Self {
                        model,
                        ir_cache: self.ir_cache,
                        path,
                    })
                    .map_err(|resolution_error| {
                        ResolutionErrorReference::new(resolution_error, self.ir_cache, path)
                    });
                (name.as_str(), result)
            })
            .collect()
    }

    /// Returns a map of parameter names to their parameter data.
    #[must_use]
    pub fn parameters(&self) -> IndexMap<&'result ir::ParameterName, &'result ir::Parameter> {
        self.model.get_parameters().iter().collect()
    }

    /// Returns the list of tests for this model.
    #[must_use]
    pub fn tests(&self) -> Vec<&'result ir::Test> {
        self.model.get_tests().values().collect()
    }
}

/// A reference to a resolution error within a model hierarchy.
///
/// Allows inspecting partial IR and error details (e.g. parameter
/// or test resolution errors) when resolution of a model fails.
#[derive(Debug, Clone, Copy)]
pub struct ResolutionErrorReference<'result> {
    resolution_error: &'result ResolutionError,
    ir_cache: &'result IrCache,
    path: &'result Path,
}

impl<'result> ResolutionErrorReference<'result> {
    /// Creates a new `ResolutionErrorReference` for the given resolution error, IR cache, and path.
    #[must_use]
    pub const fn new(
        resolution_error: &'result ResolutionError,
        ir_cache: &'result IrCache,
        path: &'result Path,
    ) -> Self {
        Self {
            resolution_error,
            ir_cache,
            path,
        }
    }

    /// Returns the partial IR for this model, if any.
    #[must_use]
    pub fn partial_ir(&self) -> Option<ModelIrReference<'result>> {
        match self.resolution_error {
            ResolutionError::ResolutionErrors { partial_ir, .. } => Some(ModelIrReference::new(
                partial_ir.as_ref(),
                self.ir_cache,
                self.path,
            )),
            ResolutionError::Parse(_) => None,
        }
    }

    /// Returns the circular dependency errors for this model, if any.
    #[must_use]
    pub fn circular_dependency_errors(&self) -> Option<Vec<&'result OneilError>> {
        match self.resolution_error {
            ResolutionError::ResolutionErrors {
                circular_dependency_errors,
                ..
            } => Some(circular_dependency_errors.iter().collect()),
            ResolutionError::Parse(_) => None,
        }
    }

    /// Returns the Python import errors for this model, if any.
    #[must_use]
    pub fn python_import_errors(
        &self,
    ) -> Option<IndexMap<&'result Path, Vec<&'result OneilError>>> {
        match self.resolution_error {
            ResolutionError::ResolutionErrors {
                python_import_errors,
                ..
            } => Some(
                python_import_errors
                    .iter()
                    .map(|(path, errors)| (path.as_ref(), errors.iter().collect()))
                    .collect(),
            ),
            ResolutionError::Parse(_) => None,
        }
    }

    /// Returns the model import errors for this model, if any.
    #[must_use]
    pub fn model_import_errors(&self) -> Option<IndexMap<&'result Path, Vec<&'result OneilError>>> {
        match self.resolution_error {
            ResolutionError::ResolutionErrors {
                model_import_errors,
                ..
            } => Some(
                model_import_errors
                    .iter()
                    .map(|(path, errors)| (path.as_ref(), errors.iter().collect()))
                    .collect(),
            ),
            ResolutionError::Parse(_) => None,
        }
    }

    /// Returns the parameter errors for this model, if any.
    #[must_use]
    pub fn parameter_errors(&self) -> Option<IndexMap<&'result str, Vec<&'result OneilError>>> {
        match self.resolution_error {
            ResolutionError::ResolutionErrors {
                parameter_errors, ..
            } => Some(
                parameter_errors
                    .iter()
                    .map(|(name, errors)| (name.as_str(), errors.iter().collect()))
                    .collect(),
            ),
            ResolutionError::Parse(_) => None,
        }
    }

    /// Returns the test errors for this model, if any.
    #[must_use]
    pub fn test_errors(&self) -> Option<Vec<&'result OneilError>> {
        match self.resolution_error {
            ResolutionError::ResolutionErrors { test_errors, .. } => {
                Some(test_errors.iter().collect())
            }
            ResolutionError::Parse(_) => None,
        }
    }
}
