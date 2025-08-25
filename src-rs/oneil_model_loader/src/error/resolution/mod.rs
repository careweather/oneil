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
    reference::{Identifier, PythonPath},
    test::TestIndex,
};

pub use import::ImportResolutionError;
pub use parameter::ParameterResolutionError;
pub use submodel::SubmodelResolutionError;
pub use test::TestResolutionError;
pub use variable::VariableResolutionError;

/// A collection of all resolution errors that occurred during model loading.
///
/// This struct aggregates errors from all resolution phases, including import validation,
/// submodel resolution, parameter resolution, and test resolution. It provides methods
/// for checking if any errors occurred and accessing the different error categories.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionErrors {
    import_errors: HashMap<PythonPath, ImportResolutionError>,
    submodel_resolution_errors: HashMap<Identifier, SubmodelResolutionError>,
    parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
    test_resolution_errors: HashMap<TestIndex, Vec<TestResolutionError>>,
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
    pub fn new(
        import_errors: HashMap<PythonPath, ImportResolutionError>,
        submodel_resolution_errors: HashMap<Identifier, SubmodelResolutionError>,
        parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
        test_resolution_errors: HashMap<TestIndex, Vec<TestResolutionError>>,
    ) -> Self {
        Self {
            import_errors,
            submodel_resolution_errors,
            parameter_resolution_errors,
            test_resolution_errors,
        }
    }

    /// Returns whether there are any resolution errors.
    ///
    /// # Returns
    ///
    /// Returns `true` if there are no errors in any category, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.import_errors.is_empty()
            && self.submodel_resolution_errors.is_empty()
            && self.parameter_resolution_errors.is_empty()
            && self.test_resolution_errors.is_empty()
    }

    /// Returns a reference to the map of import resolution errors.
    ///
    /// This method provides access to any errors that occurred during Python import validation.
    /// The errors are mapped from the Python path (with source span information) to the
    /// corresponding `ImportResolutionError`.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing Python paths and their associated import resolution errors.
    pub fn get_import_errors(&self) -> &HashMap<PythonPath, ImportResolutionError> {
        &self.import_errors
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
    /// A reference to the HashMap containing submodel identifiers and their associated resolution errors.
    pub fn get_submodel_resolution_errors(&self) -> &HashMap<Identifier, SubmodelResolutionError> {
        &self.submodel_resolution_errors
    }

    /// Returns a reference to the map of parameter resolution errors.
    ///
    /// This method provides access to any errors that occurred during parameter resolution.
    /// The errors are mapped from the parameter identifier to a vector of `ParameterResolutionError`s.
    /// Multiple errors can occur for a single parameter, for example when a parameter references
    /// multiple undefined variables or has circular dependencies.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing parameter identifiers and their associated resolution errors.
    pub fn get_parameter_resolution_errors(
        &self,
    ) -> &HashMap<Identifier, Vec<ParameterResolutionError>> {
        &self.parameter_resolution_errors
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
    /// A reference to the HashMap containing test indices and their associated resolution errors.
    pub fn get_test_resolution_errors(&self) -> &HashMap<TestIndex, Vec<TestResolutionError>> {
        &self.test_resolution_errors
    }
}
