//! Collection of resolution errors for the Oneil model loader.

use indexmap::IndexMap;

use oneil_ir as ir;

use super::circular_dependency::CircularDependencyError;
use super::import::PythonImportResolutionError;
use super::parameter::ParameterResolutionError;
use super::submodel::ModelImportResolutionError;
use super::variable::VariableResolutionError;

/// A collection of all resolution errors that occurred during model loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionErrorCollection {
    circular_dependency: Vec<CircularDependencyError>,
    python_import: IndexMap<ir::PythonPath, PythonImportResolutionError>,
    model_import:
        IndexMap<ir::ReferenceName, (Option<ir::SubmodelName>, ModelImportResolutionError)>,
    parameter: IndexMap<ir::ParameterName, Vec<ParameterResolutionError>>,
    test: IndexMap<ir::TestIndex, Vec<VariableResolutionError>>,
}

impl ResolutionErrorCollection {
    /// Creates an empty collection of resolution errors.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            circular_dependency: Vec::new(),
            python_import: IndexMap::new(),
            model_import: IndexMap::new(),
            parameter: IndexMap::new(),
            test: IndexMap::new(),
        }
    }

    /// Returns whether there are any resolution errors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.circular_dependency.is_empty()
            && self.python_import.is_empty()
            && self.model_import.is_empty()
            && self.parameter.is_empty()
            && self.test.is_empty()
    }

    /// Adds a circular dependency error.
    pub fn add_circular_dependency_error(&mut self, error: CircularDependencyError) {
        self.circular_dependency.push(error);
    }

    /// Adds a Python import resolution error.
    pub fn add_import_error(
        &mut self,
        python_path: ir::PythonPath,
        error: PythonImportResolutionError,
    ) {
        self.python_import.insert(python_path, error);
    }

    /// Adds a reference resolution error.
    pub fn add_model_import_resolution_error(
        &mut self,
        reference_name: ir::ReferenceName,
        submodel_name: Option<ir::SubmodelName>,
        error: ModelImportResolutionError,
    ) {
        self.model_import
            .insert(reference_name, (submodel_name, error));
    }

    /// Adds a parameter resolution error.
    pub fn add_parameter_error(
        &mut self,
        parameter_name: ir::ParameterName,
        error: ParameterResolutionError,
    ) {
        self.parameter
            .entry(parameter_name)
            .or_default()
            .push(error);
    }

    /// Adds a test resolution error.
    pub fn add_test_error(&mut self, test_index: ir::TestIndex, error: VariableResolutionError) {
        self.test.entry(test_index).or_default().push(error);
    }

    /// Returns a reference to the list of circular dependency errors.
    #[must_use]
    pub const fn get_circular_dependency_errors(&self) -> &[CircularDependencyError] {
        self.circular_dependency.as_slice()
    }

    /// Returns a reference to the map of import resolution errors.
    #[must_use]
    pub const fn get_python_import_resolution_errors(
        &self,
    ) -> &IndexMap<ir::PythonPath, PythonImportResolutionError> {
        &self.python_import
    }

    /// Returns a reference to the map of model import resolution errors.
    #[must_use]
    pub const fn get_model_import_resolution_errors(
        &self,
    ) -> &IndexMap<ir::ReferenceName, (Option<ir::SubmodelName>, ModelImportResolutionError)> {
        &self.model_import
    }

    /// Returns a reference to the map of parameter resolution errors.
    ///
    /// Multiple errors can occur for a single parameter, for example when a parameter
    /// has circular dependencies or references multiple undefined variables.
    #[must_use]
    pub const fn get_parameter_resolution_errors(
        &self,
    ) -> &IndexMap<ir::ParameterName, Vec<ParameterResolutionError>> {
        &self.parameter
    }

    /// Returns a reference to the map of test resolution errors.
    #[must_use]
    pub const fn get_test_resolution_errors(
        &self,
    ) -> &IndexMap<ir::TestIndex, Vec<VariableResolutionError>> {
        &self.test
    }

    /// Breaks the errors into its components.
    #[expect(
        clippy::type_complexity,
        reason = "this is just a tuple of the error maps"
    )]
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Vec<CircularDependencyError>,
        IndexMap<ir::PythonPath, PythonImportResolutionError>,
        IndexMap<ir::ReferenceName, (Option<ir::SubmodelName>, ModelImportResolutionError)>,
        IndexMap<ir::ParameterName, Vec<ParameterResolutionError>>,
        IndexMap<ir::TestIndex, Vec<VariableResolutionError>>,
    ) {
        (
            self.circular_dependency,
            self.python_import,
            self.model_import,
            self.parameter,
            self.test,
        )
    }
}
