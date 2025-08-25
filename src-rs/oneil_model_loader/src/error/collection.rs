//! Error collection and management for model loading.
//!
//! This module provides data structures for collecting and managing errors that occur
//! during the model loading process. It supports collecting errors from multiple
//! models and different error types, allowing for comprehensive error reporting.
//!
//! # Key Types
//!
//! - `ModelErrorMap`: Collects errors for multiple models, including parse errors,
//!   resolution errors, circular dependency errors, and Python import errors
//! - `ParameterErrorMap`: Collects parameter resolution errors for a single model
//!
//! # Error Separation
//!
//! The model separates different types of errors to allow for different handling
//! strategies:
//!
//! - **Model errors**: Parse and resolution errors for specific models
//! - **Circular dependency errors**: Detected circular dependencies (stored separately
//!   because they're discovered before model resolution)
//! - **Import errors**: Python import validation errors
//! - **Parameter errors**: Parameter resolution errors within a model

use std::collections::{HashMap, HashSet};

use oneil_ir::reference::{Identifier, ModelPath, PythonPath};

use crate::error::{CircularDependencyError, LoadError, ParameterResolutionError};

// note that circular dependency errors are stored seperately from model errors
// since circular dependencies are discovered before the model is resolved, and
// returning them back up the loading stack would require a lot of extra work

/// A collection of errors that occurred during model loading.
///
/// This struct maintains separate collections for different types of errors that can
/// occur during the model loading process. It provides methods for adding errors
/// and querying which models have errors.
///
/// # Error Types
///
/// - **Model errors**: Parse and resolution errors for specific models
/// - **Circular dependency errors**: Detected circular dependencies (stored separately
///   because they're discovered before model resolution)
/// - **Import errors**: Python import validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelErrorMap<Ps, Py> {
    model: HashMap<ModelPath, LoadError<Ps>>,
    circular_dependency: HashMap<ModelPath, Vec<CircularDependencyError>>,
    import: HashMap<PythonPath, Py>,
}

impl<Ps, Py> ModelErrorMap<Ps, Py> {
    /// Returns a reference to the map of model errors.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing model paths and their associated load errors.
    #[must_use]
    pub const fn get_model_errors(&self) -> &HashMap<ModelPath, LoadError<Ps>> {
        &self.model
    }

    /// Returns a reference to the map of circular dependency errors.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing model paths and their associated circular dependency errors.
    #[must_use]
    pub const fn get_circular_dependency_errors(
        &self,
    ) -> &HashMap<ModelPath, Vec<CircularDependencyError>> {
        &self.circular_dependency
    }

    /// Returns a reference to the map of import errors.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing Python paths and their associated import errors.
    #[must_use]
    pub const fn get_import_errors(&self) -> &HashMap<PythonPath, Py> {
        &self.import
    }

    /// Creates a new empty error map.
    ///
    /// # Returns
    ///
    /// A new `ModelErrorMap` with no errors.
    #[must_use]
    pub fn new() -> Self {
        Self {
            model: HashMap::new(),
            circular_dependency: HashMap::new(),
            import: HashMap::new(),
        }
    }

    /// Adds a model error for the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has an error
    /// * `error` - The error that occurred for this model
    ///
    /// # Panics
    ///
    /// Panics if a model error already exists for the given model path.
    /// This ensures that each model can only have one error recorded.
    pub(crate) fn add_model_error(&mut self, model_path: ModelPath, error: LoadError<Ps>) {
        assert!(!self.model.contains_key(&model_path));
        self.model.insert(model_path, error);
    }

    /// Adds a circular dependency error for the specified model.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has a circular dependency
    /// * `circular_dependency` - The circular dependency error
    ///
    /// Multiple circular dependency errors can be added for the same model,
    /// as a model might be involved in multiple circular dependency cycles.
    pub(crate) fn add_circular_dependency_error(
        &mut self,
        model_path: ModelPath,
        circular_dependency: CircularDependencyError,
    ) {
        self.circular_dependency
            .entry(model_path)
            .or_default()
            .push(circular_dependency);
    }

    /// Adds a Python import error for the specified import.
    ///
    /// # Arguments
    ///
    /// * `python_path` - The Python path that failed to import
    /// * `error` - The import error that occurred
    ///
    /// # Panics
    ///
    /// Panics if an import error already exists for the given Python path.
    pub(crate) fn add_import_error(&mut self, python_path: PythonPath, error: Py) {
        assert!(!self.import.contains_key(&python_path));
        self.import.insert(python_path, error);
    }

    /// Returns a set of all model paths that have errors.
    ///
    /// This includes models with parse/resolution errors and models with circular
    /// dependency errors.
    ///
    /// # Returns
    ///
    /// A set of model paths that have any type of error.
    pub(crate) fn get_models_with_errors(&self) -> HashSet<&ModelPath> {
        self.model
            .keys()
            .chain(self.circular_dependency.keys())
            .collect()
    }

    /// Returns whether there are any errors in this error map.
    ///
    /// This checks for all types of errors - model errors, circular dependency errors,
    /// and Python import errors.
    ///
    /// # Returns
    ///
    /// `true` if there are no errors of any type, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.model.is_empty() && self.circular_dependency.is_empty() && self.import.is_empty()
    }

    /// Returns a set of Python paths that have import errors.
    ///
    /// This function is only available in test builds.
    ///
    /// # Returns
    ///
    /// A set of Python paths that failed to import.
    #[cfg(test)]
    #[must_use]
    pub fn get_imports_with_errors(&self) -> HashSet<&PythonPath> {
        self.import.keys().collect()
    }
}

impl<Ps, Py> Default for ModelErrorMap<Ps, Py> {
    fn default() -> Self {
        Self::new()
    }
}

/// A collection of parameter resolution errors for a single model.
///
/// This struct collects parameter resolution errors that occur during the resolution
/// phase of model loading. It allows for tracking which parameters have errors and
/// what those errors are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterErrorMap {
    errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
}

impl<Ps, Py, S> From<ModelErrorMap<Ps, Py>>
    for (
        HashMap<ModelPath, (Option<LoadError<Ps>>, Option<Vec<CircularDependencyError>>), S>,
        HashMap<PythonPath, Py, S>,
    )
where
    S: ::std::hash::BuildHasher + Default,
{
    fn from(mut error: ModelErrorMap<Ps, Py>) -> Self {
        let mut all_model_errors = HashMap::default();

        // remove model errors and corresponding circular dependency errors, if
        // any exists
        error
            .model
            .into_iter()
            .for_each(|(model_path, load_error)| {
                let circular_dependency_errors = error.circular_dependency.remove(&model_path);

                all_model_errors.insert(model_path, (Some(load_error), circular_dependency_errors));
            });

        // remove any remaining circular dependency errors
        error.circular_dependency.into_iter().for_each(
            |(model_path, circular_dependency_errors)| {
                all_model_errors.insert(model_path, (None, Some(circular_dependency_errors)));
            },
        );

        let import_errors = error.import.into_iter().collect();

        (all_model_errors, import_errors)
    }
}

impl ParameterErrorMap {
    /// Creates a new empty parameter error map.
    ///
    /// # Returns
    ///
    /// A new `ParameterErrorMap` with no errors.
    #[must_use]
    pub fn new() -> Self {
        Self {
            errors: HashMap::default(),
        }
    }

    /// Adds a parameter resolution error for the specified parameter.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has an error
    /// * `error` - The parameter resolution error that occurred
    ///
    /// Multiple errors can be added for the same parameter, as a parameter might
    /// have multiple resolution issues.
    pub fn add_error(&mut self, identifier: Identifier, error: ParameterResolutionError) {
        self.errors.entry(identifier).or_default().push(error);
    }

    /// Returns whether there are any parameter errors.
    ///
    /// # Returns
    ///
    /// `true` if there are no parameter errors, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns a set of all parameter identifiers that have errors.
    ///
    /// # Returns
    ///
    /// A set of parameter identifiers that have resolution errors.
    #[must_use]
    pub fn get_parameters_with_errors(&self) -> HashSet<&Identifier> {
        self.errors.keys().collect()
    }
}

impl Default for ParameterErrorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> From<ParameterErrorMap> for HashMap<Identifier, Vec<ParameterResolutionError>, S>
where
    S: ::std::hash::BuildHasher + Default,
{
    fn from(error: ParameterErrorMap) -> Self {
        error.errors.into_iter().collect()
    }
}
