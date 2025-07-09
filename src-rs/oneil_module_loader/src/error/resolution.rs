//! Resolution error types for the Oneil module loader.
//!
//! This module defines error types that can occur during the resolution phase of
//! module loading. Resolution errors occur when references cannot be resolved to
//! their actual definitions, such as when a submodel reference points to a
//! non-existent module or when a parameter reference cannot be found.
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

use std::collections::HashMap;

use oneil_module::{
    reference::{Identifier, ModulePath, PythonPath},
    test::TestIndex,
};

/// A collection of all resolution errors that occurred during module loading.
///
/// This struct aggregates errors from all resolution phases, including import validation,
/// submodel resolution, parameter resolution, and test resolution. It provides methods
/// for checking if any errors occurred and accessing the different error categories.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionErrors {
    import_errors: HashMap<PythonPath, ImportResolutionError>,
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
        import_errors: HashMap<PythonPath, ImportResolutionError>,
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
}

/// Represents an error that occurred during submodel resolution.
///
/// This error type is used when a `use model` declaration cannot be resolved to
/// its corresponding module. This can happen when the referenced module has errors
/// or when the submodel identifier is not defined in the referenced module.
#[derive(Debug, Clone, PartialEq)]
pub enum SubmodelResolutionError {
    /// The referenced module has errors, preventing submodel resolution.
    ModuleHasError(ModulePath),
    /// The submodel identifier is not defined in the referenced module.
    UndefinedSubmodel(Option<ModulePath>, Identifier),
}

impl SubmodelResolutionError {
    /// Creates a new error indicating that the referenced module has errors.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has errors
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::ModuleHasError` variant.
    pub fn module_has_error(module_path: ModulePath) -> Self {
        Self::ModuleHasError(module_path)
    }

    /// Creates a new error indicating that the submodel is undefined in the referenced module.
    ///
    /// # Arguments
    ///
    /// * `parent_module_path` - The path of the parent module that contains the submodel reference
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `SubmodelResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_module_path: ModulePath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_module_path), identifier)
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
    /// The module that should contain the variable has errors.
    ModuleHasError(ModulePath),
    /// The parameter that should contain the variable has errors.
    ParameterHasError(Identifier),
    /// The submodel that should contain the variable has errors.
    SubmodelHasError(Identifier),
    /// The parameter is not defined in the current context.
    UndefinedParameter(Option<ModulePath>, Identifier),
    /// The submodel is not defined in the current context.
    UndefinedSubmodel(Option<ModulePath>, Identifier),
}

impl VariableResolutionError {
    /// Creates a new error indicating that the module has errors.
    ///
    /// # Arguments
    ///
    /// * `module_path` - The path of the module that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::ModuleHasError` variant.
    pub fn module_has_error(module_path: ModulePath) -> Self {
        Self::ModuleHasError(module_path)
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

    /// Creates a new error indicating that the submodel has errors.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the submodel that has errors
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::SubmodelHasError` variant.
    pub fn submodel_has_error(identifier: Identifier) -> Self {
        Self::SubmodelHasError(identifier)
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
        submodel_path: ModulePath,
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

    /// Creates a new error indicating that the submodel is undefined in a specific parent module.
    ///
    /// # Arguments
    ///
    /// * `parent_module_path` - The path of the parent module where the submodel should be defined
    /// * `identifier` - The identifier of the undefined submodel
    ///
    /// # Returns
    ///
    /// A new `VariableResolutionError::UndefinedSubmodel` variant.
    pub fn undefined_submodel_in_submodel(
        parent_module_path: ModulePath,
        identifier: Identifier,
    ) -> Self {
        Self::UndefinedSubmodel(Some(parent_module_path), identifier)
    }
}
