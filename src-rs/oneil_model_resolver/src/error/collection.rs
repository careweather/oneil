//! Error collection and management for model loading.

use std::collections::HashSet;

use indexmap::IndexMap;

use oneil_ir as ir;

use crate::error::{CircularDependencyError, LoadError, ParameterResolutionError};

// note that circular dependency errors are stored seperately from model errors
// since circular dependencies are discovered before the model is resolved, and
// returning them back up the loading stack would require a lot of extra work

/// A collection of errors that occurred during model loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelErrorMap<Ps, Py> {
    model: IndexMap<ir::ModelPath, LoadError<Ps>>,
    circular_dependency: IndexMap<ir::ModelPath, Vec<CircularDependencyError>>,
    import: IndexMap<ir::PythonPath, Py>,
}

impl<Ps, Py> ModelErrorMap<Ps, Py> {
    /// Returns a reference to the map of model errors.
    #[must_use]
    pub const fn get_model_errors(&self) -> &IndexMap<ir::ModelPath, LoadError<Ps>> {
        &self.model
    }

    /// Returns a reference to the map of circular dependency errors.
    #[must_use]
    pub const fn get_circular_dependency_errors(
        &self,
    ) -> &IndexMap<ir::ModelPath, Vec<CircularDependencyError>> {
        &self.circular_dependency
    }

    /// Returns a reference to the map of import errors.
    #[must_use]
    pub const fn get_import_errors(&self) -> &IndexMap<ir::PythonPath, Py> {
        &self.import
    }

    /// Creates a new empty error map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            model: IndexMap::new(),
            circular_dependency: IndexMap::new(),
            import: IndexMap::new(),
        }
    }

    /// Adds a model error for the specified model.
    pub(crate) fn add_model_error(&mut self, model_path: ir::ModelPath, error: LoadError<Ps>) {
        assert!(!self.model.contains_key(&model_path));
        self.model.insert(model_path, error);
    }

    /// Adds a circular dependency error for the specified model.
    pub(crate) fn add_circular_dependency_error(
        &mut self,
        model_path: ir::ModelPath,
        circular_dependency: CircularDependencyError,
    ) {
        self.circular_dependency
            .entry(model_path)
            .or_default()
            .push(circular_dependency);
    }

    /// Adds a Python import error for the specified import.
    pub(crate) fn add_import_error(&mut self, python_path: ir::PythonPath, error: Py) {
        assert!(!self.import.contains_key(&python_path));
        self.import.insert(python_path, error);
    }

    /// Returns a set of all model paths that have errors.
    pub(crate) fn get_models_with_errors(&self) -> HashSet<&ir::ModelPath> {
        self.model
            .keys()
            .chain(self.circular_dependency.keys())
            .collect()
    }

    /// Returns whether there are any errors in this error map.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.model.is_empty() && self.circular_dependency.is_empty() && self.import.is_empty()
    }

    /// Returns a set of Python paths that have import errors.
    #[cfg(test)]
    #[must_use]
    pub fn get_imports_with_errors(&self) -> HashSet<&ir::PythonPath> {
        self.import.keys().collect()
    }
}

impl<Ps, Py> Default for ModelErrorMap<Ps, Py> {
    fn default() -> Self {
        Self::new()
    }
}

/// A collection of parameter resolution errors for a single model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterErrorMap {
    errors: IndexMap<ir::Identifier, Vec<ParameterResolutionError>>,
}

impl<Ps, Py, S> From<ModelErrorMap<Ps, Py>>
    for (
        IndexMap<ir::ModelPath, (Option<LoadError<Ps>>, Option<Vec<CircularDependencyError>>), S>,
        IndexMap<ir::PythonPath, Py, S>,
    )
where
    S: ::std::hash::BuildHasher + Default,
{
    fn from(mut error: ModelErrorMap<Ps, Py>) -> Self {
        let mut all_model_errors = IndexMap::default();

        // remove model errors and corresponding circular dependency errors, if
        // any exists
        error
            .model
            .into_iter()
            .for_each(|(model_path, load_error)| {
                let circular_dependency_errors = error.circular_dependency.swap_remove(&model_path);

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
    #[must_use]
    pub fn new() -> Self {
        Self {
            errors: IndexMap::default(),
        }
    }

    /// Adds a parameter resolution error for the specified parameter.
    pub fn add_error(&mut self, identifier: ir::Identifier, error: ParameterResolutionError) {
        self.errors.entry(identifier).or_default().push(error);
    }

    /// Returns whether there are any parameter errors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns a set of all parameter identifiers that have errors.
    #[must_use]
    pub fn get_parameters_with_errors(&self) -> HashSet<&ir::Identifier> {
        self.errors.keys().collect()
    }
}

impl Default for ParameterErrorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> From<ParameterErrorMap> for IndexMap<ir::Identifier, Vec<ParameterResolutionError>, S>
where
    S: ::std::hash::BuildHasher + Default,
{
    fn from(error: ParameterErrorMap) -> Self {
        error.errors.into_iter().collect()
    }
}
