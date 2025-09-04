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
//!
//! # Error Hierarchy
//!
//! The error types form a hierarchy where higher-level errors (like parameter resolution)
//! can contain lower-level errors (like variable resolution). This allows for detailed
//! error reporting while maintaining a clean error structure.

mod import;
mod parameter;
mod submodel;
mod test;
mod variable;

use std::collections::HashMap;

use oneil_ir::{
    model_import::{ReferenceName, SubmodelName},
    reference::{Identifier, PythonPath},
    test::TestIndex,
};

pub use import::ImportResolutionError;
pub use parameter::ParameterResolutionError;
pub use submodel::ModelImportResolutionError;
pub use test::TestResolutionError;
pub use variable::VariableResolutionError;

/// A collection of all resolution errors that occurred during model loading.
///
/// This struct aggregates errors from all resolution phases, including import validation,
/// submodel resolution, parameter resolution, and test resolution. It provides methods
/// for checking if any errors occurred and accessing the different error categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionErrors {
    import: HashMap<PythonPath, ImportResolutionError>,
    submodel_resolution: HashMap<SubmodelName, ModelImportResolutionError>,
    reference_resolution: HashMap<ReferenceName, ModelImportResolutionError>,
    parameter_resolution: HashMap<Identifier, Vec<ParameterResolutionError>>,
    test_resolution: HashMap<TestIndex, Vec<TestResolutionError>>,
}

impl ResolutionErrors {
    /// Creates a new collection of resolution errors.
    ///
    /// # Arguments
    ///
    /// * `import_errors` - Errors that occurred during Python import validation
    /// * `submodel_resolution_errors` - Errors that occurred during submodel resolution
    /// * `parameter_resolution_errors` - Errors that occurred during parameter resolution
    /// * `test_resolution_errors` - Errors that occurred during test resolution
    ///
    /// # Returns
    ///
    /// A new `ResolutionErrors` instance containing all the specified errors.
    #[must_use]
    pub const fn new(
        import_errors: HashMap<PythonPath, ImportResolutionError>,
        submodel_resolution_errors: HashMap<SubmodelName, ModelImportResolutionError>,
        reference_resolution_errors: HashMap<ReferenceName, ModelImportResolutionError>,
        parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
        test_resolution_errors: HashMap<TestIndex, Vec<TestResolutionError>>,
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
    ///
    /// # Returns
    ///
    /// Returns `true` if there are no errors in any category, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.import.is_empty()
            && self.submodel_resolution.is_empty()
            && self.parameter_resolution.is_empty()
            && self.test_resolution.is_empty()
    }

    /// Returns a reference to the map of import resolution errors.
    ///
    /// This method provides access to any errors that occurred during Python import validation.
    /// The errors are mapped from the Python path (with source span information) to the
    /// corresponding `ImportResolutionError`.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing Python paths and their associated import resolution errors.
    #[must_use]
    pub const fn get_import_errors(&self) -> &HashMap<PythonPath, ImportResolutionError> {
        &self.import
    }

    /// Returns a reference to the map of submodel resolution errors.
    ///
    /// This method provides access to any errors that occurred during submodel resolution.
    /// The errors are mapped from the submodel identifier to the corresponding `SubmodelResolutionError`.
    /// These errors occur when a `use model` declaration cannot be resolved, either because the referenced
    /// model has errors or the submodel identifier is not defined.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing submodel identifiers and their associated resolution errors.
    #[must_use]
    pub const fn get_submodel_resolution_errors(
        &self,
    ) -> &HashMap<SubmodelName, ModelImportResolutionError> {
        &self.submodel_resolution
    }

    /// Returns a reference to the map of reference resolution errors.
    ///
    /// Only one error can occur for a single reference.
    #[must_use]
    pub const fn get_reference_resolution_errors(
        &self,
    ) -> &HashMap<ReferenceName, ModelImportResolutionError> {
        &self.reference_resolution
    }

    /// Returns a reference to the map of parameter resolution errors.
    ///
    /// Multiple errors can occur for a single parameter, for example when a parameter
    /// has circular dependencies or references multiple undefined variables.
    #[must_use]
    pub const fn get_parameter_resolution_errors(
        &self,
    ) -> &HashMap<Identifier, Vec<ParameterResolutionError>> {
        &self.parameter_resolution
    }

    /// Returns a reference to the map of test resolution errors.
    ///
    /// This method provides access to any errors that occurred during test resolution.
    /// The errors are mapped from the test index to a vector of `TestResolutionError`s.
    /// Multiple errors can occur for a single test, for example when a test references
    /// undefined variables or has invalid assertions.
    ///
    /// # Returns
    ///
    /// A reference to the `HashMap` containing test indices and their associated resolution errors.
    #[must_use]
    pub const fn get_test_resolution_errors(
        &self,
    ) -> &HashMap<TestIndex, Vec<TestResolutionError>> {
        &self.test_resolution
    }
}
