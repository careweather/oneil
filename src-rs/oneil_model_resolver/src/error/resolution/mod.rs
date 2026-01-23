//! Resolution error types for the Oneil model loader.
//!
//! This module defines error types that can occur during the resolution phase of
//! model loading. Resolution errors occur when references cannot be resolved to
//! their actual definitions, such as when a submodel reference points to a
//! non-existent model or when a parameter reference cannot be found.
//!
//! # Error Categories
//!
//! - **Import errors**: Errors that occur during Python import validation
//! - **Submodel resolution errors**: Errors that occur when resolving `use model` declarations
//! - **Parameter resolution errors**: Errors that occur when resolving parameter references
//! - **Test resolution errors**: Errors that occur when resolving test references
//! - **Variable resolution errors**: Errors that occur when resolving variable references within expressions

mod import;
mod parameter;
mod submodel;
mod test;
mod variable;

use indexmap::IndexMap;

use oneil_ir as ir;

pub use import::ImportResolutionError;
pub use parameter::ParameterResolutionError;
pub use submodel::ModelImportResolutionError;
pub use test::TestResolutionError;
pub use variable::VariableResolutionError;

/// A collection of all resolution errors that occurred during model loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionErrors {
    import: IndexMap<ir::PythonPath, ImportResolutionError>,
    submodel_resolution: IndexMap<ir::SubmodelName, ModelImportResolutionError>,
    reference_resolution: IndexMap<ir::ReferenceName, ModelImportResolutionError>,
    parameter_resolution: IndexMap<ir::ParameterName, Vec<ParameterResolutionError>>,
    test_resolution: IndexMap<ir::TestIndex, Vec<TestResolutionError>>,
}

impl ResolutionErrors {
    /// Creates a new collection of resolution errors.
    #[must_use]
    pub const fn new(
        import_errors: IndexMap<ir::PythonPath, ImportResolutionError>,
        submodel_resolution_errors: IndexMap<ir::SubmodelName, ModelImportResolutionError>,
        reference_resolution_errors: IndexMap<ir::ReferenceName, ModelImportResolutionError>,
        parameter_resolution_errors: IndexMap<ir::ParameterName, Vec<ParameterResolutionError>>,
        test_resolution_errors: IndexMap<ir::TestIndex, Vec<TestResolutionError>>,
    ) -> Self {
        Self {
            import: import_errors,
            submodel_resolution: submodel_resolution_errors,
            reference_resolution: reference_resolution_errors,
            parameter_resolution: parameter_resolution_errors,
            test_resolution: test_resolution_errors,
        }
    }

    /// Returns whether there are any resolution errors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.import.is_empty()
            && self.submodel_resolution.is_empty()
            && self.parameter_resolution.is_empty()
            && self.test_resolution.is_empty()
    }

    /// Returns a reference to the map of import resolution errors.
    #[must_use]
    pub const fn get_import_errors(&self) -> &IndexMap<ir::PythonPath, ImportResolutionError> {
        &self.import
    }

    /// Returns a reference to the map of submodel resolution errors.
    #[must_use]
    pub const fn get_submodel_resolution_errors(
        &self,
    ) -> &IndexMap<ir::SubmodelName, ModelImportResolutionError> {
        &self.submodel_resolution
    }

    /// Returns a reference to the map of reference resolution errors.
    ///
    /// Only one error can occur for a single reference.
    #[must_use]
    pub const fn get_reference_resolution_errors(
        &self,
    ) -> &IndexMap<ir::ReferenceName, ModelImportResolutionError> {
        &self.reference_resolution
    }

    /// Returns a reference to the map of parameter resolution errors.
    ///
    /// Multiple errors can occur for a single parameter, for example when a parameter
    /// has circular dependencies or references multiple undefined variables.
    #[must_use]
    pub const fn get_parameter_resolution_errors(
        &self,
    ) -> &IndexMap<ir::ParameterName, Vec<ParameterResolutionError>> {
        &self.parameter_resolution
    }

    /// Returns a reference to the map of test resolution errors.
    #[must_use]
    pub const fn get_test_resolution_errors(
        &self,
    ) -> &IndexMap<ir::TestIndex, Vec<TestResolutionError>> {
        &self.test_resolution
    }
}
