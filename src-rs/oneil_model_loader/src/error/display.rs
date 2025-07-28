//! Error display utilities for the Oneil model loader.
//!
//! This module provides functions to convert various resolution error types into
//! user-friendly error messages. The error messages follow a consistent style
//! and provide clear information about what went wrong during model resolution.

use crate::error::{
    ImportResolutionError, ModelTestResolutionError, ParameterResolutionError,
    SubmodelResolutionError, SubmodelTestInputResolutionError, VariableResolutionError,
};

/// Converts an import resolution error to a user-friendly error message.
///
/// # Arguments
///
/// * `_error` - The import resolution error to convert
///
/// # Returns
///
/// A string containing a user-friendly error message for the import resolution error.
pub fn import_resolution_error_to_string(_error: &ImportResolutionError) -> String {
    "python import validation failed".to_string()
}

/// Converts a submodel resolution error to a user-friendly error message.
///
/// # Arguments
///
/// * `error` - The submodel resolution error to convert
///
/// # Returns
///
/// A string containing a user-friendly error message for the submodel resolution error.
pub fn submodel_resolution_error_to_string(error: &SubmodelResolutionError) -> String {
    match error {
        SubmodelResolutionError::ModelHasError(model_path) => {
            let path = model_path.as_ref().display();
            format!("submodel `{}` has errors", path)
        }
        SubmodelResolutionError::UndefinedSubmodel(parent_model_path, identifier) => {
            let identifier = identifier.value();
            match parent_model_path {
                Some(path) => {
                    let path = path.as_ref().display();
                    format!(
                        "submodel `{}` is not defined in model `{}`",
                        identifier.value(),
                        path
                    )
                }
                None => format!("submodel `{}` is not defined", identifier.value()),
            }
        }
    }
}

/// Converts a parameter resolution error to a user-friendly error message.
///
/// # Arguments
///
/// * `error` - The parameter resolution error to convert
///
/// # Returns
///
/// A string containing a user-friendly error message for the parameter resolution error.
pub fn parameter_resolution_error_to_string(error: &ParameterResolutionError) -> String {
    match error {
        ParameterResolutionError::CircularDependency(circular_dependency) => {
            let dependency_chain = circular_dependency
                .iter()
                .map(|id| format!("{:?}", id))
                .collect::<Vec<_>>()
                .join(" -> ");
            format!("circular dependency detected: {}", dependency_chain)
        }
        ParameterResolutionError::VariableResolution(variable_error) => {
            variable_resolution_error_to_string(variable_error)
        }
    }
}

/// Converts a model test resolution error to a user-friendly error message.
///
/// # Arguments
///
/// * `error` - The model test resolution error to convert
///
/// # Returns
///
/// A string containing a user-friendly error message for the model test resolution error.
pub fn model_test_resolution_error_to_string(error: &ModelTestResolutionError) -> String {
    variable_resolution_error_to_string(error.get_error())
}

/// Converts a submodel test input resolution error to a user-friendly error message.
///
/// # Arguments
///
/// * `error` - The submodel test input resolution error to convert
///
/// # Returns
///
/// A string containing a user-friendly error message for the submodel test input resolution error.
pub fn submodel_test_resolution_error_to_string(
    error: &SubmodelTestInputResolutionError,
) -> String {
    match error {
        SubmodelTestInputResolutionError::VariableResolution(variable_error) => {
            variable_resolution_error_to_string(variable_error)
        }
    }
}

/// Converts a variable resolution error to a user-friendly error message.
///
/// # Arguments
///
/// * `error` - The variable resolution error to convert
///
/// # Returns
///
/// A string containing a user-friendly error message for the variable resolution error.
pub fn variable_resolution_error_to_string(error: &VariableResolutionError) -> String {
    match error {
        VariableResolutionError::ModelHasError(model_path) => {
            format!("model `{:?}` has errors", model_path)
        }
        VariableResolutionError::ParameterHasError(identifier) => {
            format!("parameter `{:?}` has errors", identifier)
        }
        VariableResolutionError::SubmodelResolutionFailed(identifier) => {
            format!("submodel `{:?}` resolution failed", identifier)
        }
        VariableResolutionError::UndefinedParameter(model_path, identifier) => match model_path {
            Some(path) => format!(
                "parameter `{:?}` is not defined in model `{:?}`",
                identifier, path
            ),
            None => format!("parameter `{:?}` is not defined", identifier),
        },
        VariableResolutionError::UndefinedSubmodel(model_path, identifier) => match model_path {
            Some(path) => format!(
                "submodel `{:?}` is not defined in model `{:?}`",
                identifier, path
            ),
            None => format!("submodel `{:?}` is not defined", identifier),
        },
    }
}
