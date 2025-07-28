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

use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use oneil_ir::{
    reference::{Identifier, ModelPath, PythonPath},
    span::WithSpan,
    test::TestIndex,
};

use crate::error::display;

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

/// Represents an error that occurred during Python import validation.
///
/// This error type is used when a Python import declaration cannot be validated,
/// typically because the referenced Python file does not exist or cannot be imported.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResolutionError;

impl ImportResolutionError {
    /// Creates a new import resolution error.
    ///
    /// # Returns
    ///
    /// A new `ImportResolutionError` instance.
    pub fn new() -> Self {
        Self
    }

    pub fn to_string(&self) -> String {
        display::import_resolution_error_to_string(self)
    }
}

impl Display for ImportResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Represents an error that occurred during submodel resolution.
///
/// This error type is used when a `use model` declaration cannot be resolved to
/// its corresponding model. This can happen when the referenced model has errors
/// or when the submodel identifier is not defined in the referenced model.
#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelResolutionError {
    /// The referenced model has errors, preventing submodel resolution.
    ModelHasError(ModelPath),
    /// The submodel identifier is not defined in the referenced model.
    UndefinedSubmodel(Option<ModelPath>, WithSpan<Identifier>),
}

impl SubmodelResolutionError {
    /// Creates a new error indicating that the referenced model has errors.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has errors
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::ModelHasError` variant.
    pub fn model_has_error(model_path: ModelPath) -> Self {
        Self::ModelHasError(model_path)
    }

    /// Creates a new error indicating that the submodel is undefined in the referenced model.
    ///
    /// # Arguments
    ///
    /// * `parent_model_path` - The path of the parent model that contains the submodel reference
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_model_path: ModelPath,
        identifier: WithSpan<Identifier>,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_model_path), identifier)
    }

    pub fn to_string(&self) -> String {
        display::submodel_resolution_error_to_string(self)
    }
}

impl Display for SubmodelResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Represents an error that occurred during parameter resolution.
///
/// This error type is used when a parameter reference cannot be resolved to its
/// actual parameter definition. This can happen due to circular dependencies or
/// variable resolution errors within the parameter's value.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterResolutionError {
    /// A circular dependency was detected during parameter resolution.
    CircularDependency(Vec<Identifier>),
    /// A variable resolution error occurred within the parameter's value.
    VariableResolution(VariableResolutionError),
}

impl ParameterResolutionError {
    /// Creates a new error indicating a circular dependency in parameter resolution.
    ///
    /// # Arguments
    ///
    /// * `circular_dependency` - The list of parameter identifiers that form the circular dependency
    ///
    /// # Returns
    ///
    /// A new `ParameterResolutionError::CircularDependency` variant.
    pub fn circular_dependency(circular_dependency: Vec<Identifier>) -> Self {
        Self::CircularDependency(circular_dependency)
    }

    /// Creates a new error indicating a variable resolution error within a parameter.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `ParameterResolutionError::VariableResolution` variant.
    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }

    pub fn to_string(&self) -> String {
        display::parameter_resolution_error_to_string(self)
    }
}

impl Display for ParameterResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<VariableResolutionError> for ParameterResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

/// Represents an error that occurred during model test resolution.
///
/// This error type is used when a model test cannot be resolved, typically due
/// to variable resolution errors within the test's expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTestResolutionError(VariableResolutionError);

impl ModelTestResolutionError {
    /// Creates a new model test resolution error.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `ModelTestResolutionError` instance.
    pub fn new(error: VariableResolutionError) -> Self {
        Self(error)
    }

    pub fn to_string(&self) -> String {
        display::model_test_resolution_error_to_string(self)
    }

    pub fn get_error(&self) -> &VariableResolutionError {
        &self.0
    }
}

impl Display for ModelTestResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<VariableResolutionError> for ModelTestResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::new(error)
    }
}

/// Represents an error that occurred during submodel test input resolution.
///
/// This error type is used when a submodel test input cannot be resolved, typically
/// due to variable resolution errors within the input's expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelTestInputResolutionError {
    /// A variable resolution error occurred within the test input.
    VariableResolution(VariableResolutionError),
}

impl SubmodelTestInputResolutionError {
    /// Creates a new error indicating a variable resolution error in a test input.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `SubmodelTestInputResolutionError::VariableResolution` variant.
    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }

    pub fn to_string(&self) -> String {
        display::submodel_test_resolution_error_to_string(self)
    }
}

impl Display for SubmodelTestInputResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<VariableResolutionError> for SubmodelTestInputResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

/// Represents an error that occurred during variable resolution within expressions.
///
/// This error type is used when a variable reference within an expression cannot
/// be resolved. This is the most fundamental resolution error type, as it represents
/// the failure to resolve a simple identifier to its definition.
#[derive(Debug, Clone, PartialEq)]
pub enum VariableResolutionError {
    /// The model that should contain the variable has errors.
    ModelHasError(ModelPath),
    /// The parameter that should contain the variable has errors.
    ParameterHasError(Identifier),
    /// The resolution of a submodel that is referenced by a variable has failed.
    SubmodelResolutionFailed(Identifier),
    /// The parameter is not defined in the current context.
    UndefinedParameter(Option<ModelPath>, Identifier),
    /// The submodel is not defined in the current context.
    UndefinedSubmodel(Option<ModelPath>, Identifier),
}

impl VariableResolutionError {
    /// Creates a new error indicating that the model has errors.
    ///
    /// # Arguments
    ///
    /// * `model_path` - The path of the model that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ModelHasError` variant.
    pub fn model_has_error(model_path: ModelPath) -> Self {
        Self::ModelHasError(model_path)
    }

    /// Creates a new error indicating that the parameter has errors.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the parameter that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ParameterHasError` variant.
    pub fn parameter_has_error(identifier: Identifier) -> Self {
        Self::ParameterHasError(identifier)
    }

    /// Creates a new error indicating that resolution of a submodel that is
    /// referenced by a variable has failed.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::SubmodelResolutionFailed` variant.
    pub fn submodel_resolution_failed(identifier: Identifier) -> Self {
        Self::SubmodelResolutionFailed(identifier)
    }

    /// Creates a new error indicating that the parameter is undefined.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the undefined parameter
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedParameter` variant.
    pub fn undefined_parameter(identifier: Identifier) -> Self {
        Self::UndefinedParameter(None, identifier)
    }

    /// Creates a new error indicating that the parameter is undefined in a specific submodel.
    ///
    /// # Arguments
    ///
    /// * `submodel_path` - The path of the submodel where the parameter should be defined
    /// * `identifier` - The identifier of the undefined parameter
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedParameter` variant.
    pub fn undefined_parameter_in_submodel(
        submodel_path: ModelPath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedParameter(Some(submodel_path), identifier)
    }

    /// Creates a new error indicating that the submodel is undefined.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel(identifier: Identifier) -> Self {
        Self::UndefinedSubmodel(None, identifier)
    }

    /// Creates a new error indicating that the submodel is undefined in a specific parent model.
    ///
    /// # Arguments
    ///
    /// * `parent_model_path` - The path of the parent model where the submodel should be defined
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_model_path: ModelPath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_model_path), identifier)
    }

    pub fn to_string(&self) -> String {
        display::variable_resolution_error_to_string(self)
    }
}

impl Display for VariableResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
