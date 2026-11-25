//! Collection of resolution errors for the Oneil model loader.

use indexmap::IndexMap;

use oneil_shared::paths::PythonPath;
use oneil_shared::symbols::{ParameterName, ReferenceName, SubmodelName, TestIndex};

use super::design::DesignResolutionError;
use super::model_import::ModelImportResolutionError;
use super::parameter::ParameterResolutionError;
use super::python_import::PythonImportResolutionError;
use super::variable::VariableResolutionError;

/// A collection of all *resolver-time* errors that occurred during model
/// loading.
///
/// These are file-static diagnostics produced while the resolver lowers
/// each `.on` / `.one` file into an
/// [`InstancedModel`](crate::InstancedModel) template — they don't
/// require an instance graph or runtime designs to be detected.
/// Graph-time diagnostics (e.g. compilation cycles, overlay-target
/// misses, validation errors) live in the error buckets on the
/// [`InstanceGraph`](crate::InstanceGraph), never on individual
/// [`InstancedModel`](crate::InstancedModel) nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionErrorCollection {
    python_import: IndexMap<PythonPath, PythonImportResolutionError>,
    model_import: IndexMap<ReferenceName, (Option<SubmodelName>, ModelImportResolutionError)>,
    parameter: IndexMap<ParameterName, Vec<ParameterResolutionError>>,
    test: IndexMap<TestIndex, Vec<VariableResolutionError>>,
    /// Design overlay / `apply` resolution errors.
    design_resolution: Vec<DesignResolutionError>,
}

impl ResolutionErrorCollection {
    /// Creates an empty collection of resolution errors.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            python_import: IndexMap::new(),
            model_import: IndexMap::new(),
            parameter: IndexMap::new(),
            test: IndexMap::new(),
            design_resolution: Vec::new(),
        }
    }

    /// Returns whether there are any resolution errors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.python_import.is_empty()
            && self.model_import.is_empty()
            && self.parameter.is_empty()
            && self.test.is_empty()
            && self.design_resolution.is_empty()
    }

    /// Adds a Python import resolution error.
    pub fn add_import_error(
        &mut self,
        python_path: PythonPath,
        error: PythonImportResolutionError,
    ) {
        self.python_import.insert(python_path, error);
    }

    /// Adds a reference resolution error.
    pub fn add_model_import_resolution_error(
        &mut self,
        reference_name: ReferenceName,
        submodel_name: Option<SubmodelName>,
        error: ModelImportResolutionError,
    ) {
        self.model_import
            .insert(reference_name, (submodel_name, error));
    }

    /// Adds a parameter resolution error.
    pub fn add_parameter_error(
        &mut self,
        parameter_name: ParameterName,
        error: ParameterResolutionError,
    ) {
        self.parameter
            .entry(parameter_name)
            .or_default()
            .push(error);
    }

    /// Adds a test resolution error.
    pub fn add_test_error(&mut self, test_index: TestIndex, error: VariableResolutionError) {
        self.test.entry(test_index).or_default().push(error);
    }

    /// Adds a design-resolution error (for example an invalid `apply` or dependent overlay).
    pub fn add_design_resolution_error(&mut self, error: DesignResolutionError) {
        self.design_resolution.push(error);
    }

    /// Returns design-resolution errors, if any.
    #[must_use]
    pub const fn get_design_resolution_errors(&self) -> &[DesignResolutionError] {
        self.design_resolution.as_slice()
    }

    /// Returns a reference to the map of import resolution errors.
    #[must_use]
    pub const fn get_python_import_resolution_errors(
        &self,
    ) -> &IndexMap<PythonPath, PythonImportResolutionError> {
        &self.python_import
    }

    /// Returns a reference to the map of model import resolution errors.
    #[must_use]
    pub const fn get_model_import_resolution_errors(
        &self,
    ) -> &IndexMap<ReferenceName, (Option<SubmodelName>, ModelImportResolutionError)> {
        &self.model_import
    }

    /// Returns a reference to the map of parameter resolution errors.
    ///
    /// Multiple errors can occur for a single parameter, for example when a parameter
    /// has circular dependencies or references multiple undefined variables.
    #[must_use]
    pub const fn get_parameter_resolution_errors(
        &self,
    ) -> &IndexMap<ParameterName, Vec<ParameterResolutionError>> {
        &self.parameter
    }

    /// Returns a reference to the map of test resolution errors.
    #[must_use]
    pub const fn get_test_resolution_errors(
        &self,
    ) -> &IndexMap<TestIndex, Vec<VariableResolutionError>> {
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
        IndexMap<PythonPath, PythonImportResolutionError>,
        IndexMap<ReferenceName, (Option<SubmodelName>, ModelImportResolutionError)>,
        IndexMap<ParameterName, Vec<ParameterResolutionError>>,
        IndexMap<TestIndex, Vec<VariableResolutionError>>,
        Vec<DesignResolutionError>,
    ) {
        (
            self.python_import,
            self.model_import,
            self.parameter,
            self.test,
            self.design_resolution,
        )
    }
}
