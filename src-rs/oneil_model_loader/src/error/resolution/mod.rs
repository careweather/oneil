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
mod model_test;
mod parameter;
mod submodel;
mod submodel_test_input;
mod variable;

use std::collections::HashMap;

use oneil_ir::{
    reference::{Identifier, PythonPath},
    span::WithSpan,
    test::TestIndex,
};

pub use import::ImportResolutionError;
pub use model_test::ModelTestResolutionError;
pub use parameter::ParameterResolutionError;
pub use submodel::SubmodelResolutionError;
pub use submodel_test_input::SubmodelTestInputResolutionError;
pub use variable::VariableResolutionError;

/// A collection of all resolution errors that occurred during model loading.
///
/// This struct aggregates errors from all resolution phases, including import validation,
/// submodel resolution, parameter resolution, and test resolution. It provides methods
/// for checking if any errors occurred and accessing the different error categories.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionErrors {
    import_errors: HashMap<WithSpan<PythonPath>, ImportResolutionError>,
    submodel_resolution_errors: HashMap<Identifier, SubmodelResolutionError>,
    parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
    model_test_resolution_errors: HashMap<TestIndex, Vec<ModelTestResolutionError>>,
    submodel_test_resolution_errors: HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
}

impl ResolutionErrors {
    /// Creates a new collection of resolution errors.
    ///
    /// # Arguments
    ///
    /// * `import_errors` - Errors that occurred during Python import validation
    /// * `submodel_resolution_errors` - Errors that occurred during submodel resolution
    /// * `parameter_resolution_errors` - Errors that occurred during parameter resolution
    /// * `model_test_resolution_errors` - Errors that occurred during model test resolution
    /// * `submodel_test_resolution_errors` - Errors that occurred during submodel test resolution
    ///
    /// # Returns
    ///
    /// A new `ResolutionErrors` instance containing all the specified errors.
    pub fn new(
        import_errors: HashMap<WithSpan<PythonPath>, ImportResolutionError>,
        submodel_resolution_errors: HashMap<Identifier, SubmodelResolutionError>,
        parameter_resolution_errors: HashMap<Identifier, Vec<ParameterResolutionError>>,
        model_test_resolution_errors: HashMap<TestIndex, Vec<ModelTestResolutionError>>,
        submodel_test_resolution_errors: HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
    ) -> Self {
        Self {
            import_errors,
            submodel_resolution_errors,
            parameter_resolution_errors,
            model_test_resolution_errors,
            submodel_test_resolution_errors,
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
            && self.model_test_resolution_errors.is_empty()
            && self.submodel_test_resolution_errors.is_empty()
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
    pub fn get_import_errors(&self) -> &HashMap<WithSpan<PythonPath>, ImportResolutionError> {
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

    /// Returns a reference to the map of model test resolution errors.
    ///
    /// This method provides access to any errors that occurred during model test resolution.
    /// The errors are mapped from the test index to a vector of `ModelTestResolutionError`s.
    /// Multiple errors can occur for a single test, for example when a test references
    /// undefined variables or has invalid assertions.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing test indices and their associated resolution errors.
    pub fn get_model_test_resolution_errors(
        &self,
    ) -> &HashMap<TestIndex, Vec<ModelTestResolutionError>> {
        &self.model_test_resolution_errors
    }

    /// Returns a reference to the map of submodel test resolution errors.
    ///
    /// This method provides access to any errors that occurred during submodel test input resolution.
    /// The errors are mapped from the submodel identifier to a vector of `SubmodelTestInputResolutionError`s.
    /// These errors occur when test inputs for a submodel cannot be resolved, for example when the input
    /// references undefined variables or has invalid values.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing submodel identifiers and their associated test input resolution errors.
    pub fn get_submodel_test_input_resolution_errors(
        &self,
    ) -> &HashMap<Identifier, Vec<SubmodelTestInputResolutionError>> {
        &self.submodel_test_resolution_errors
    }
}
