//! Model loader error conversion for the Oneil CLI
//!
//! This module provides functionality for converting model loader errors from the
//! Oneil model loader library into the unified error format used by the Oneil CLI.
//! It handles complex errors that occur during model loading, including import errors,
//! circular dependency errors, resolution errors, and various types of validation errors.
//!
//! The module processes errors from multiple sources:
//! - Import errors (missing Python files)
//! - Circular dependency errors in model references
//! - Model parsing errors (syntax, file I/O)
//! - Resolution errors (undefined variables, parameters, submodels)
//! - Test resolution errors

use std::{fs, path::Path};

use oneil_error::OneilError;
use oneil_ir::reference::{ModelPath, PythonPath};
use oneil_model_loader::{
    ModelErrorMap,
    error::{
        CircularDependencyError, LoadError, ResolutionErrors, SubmodelResolutionError,
        SubmodelTestInputResolutionError, VariableResolutionError,
    },
};

use crate::{
    convert_error::{file, parser},
    file_parser::{DoesNotExistError, LoadingError},
};

/// Converts a model error map into a collection of unified CLI errors
///
/// Takes a `ModelErrorMap` containing various types of errors from the model loading
/// process and converts them into a unified format suitable for display in the CLI.
/// The errors are processed in a specific order to maintain consistency.
///
/// # Arguments
///
/// * `error_map` - The model error map containing all errors from the loading process
///
/// # Returns
///
/// Returns a vector of `Error` instances, one for each error in the error map.
/// Model errors are sorted by offset to maintain the order of appearance in the source file.
///
/// # Note
///
/// This function processes errors in the following order:
/// 1. Import errors (Python file validation)
/// 2. Circular dependency errors
/// 3. Model errors (parsing and resolution)
pub fn convert_map(error_map: &ModelErrorMap<LoadingError, DoesNotExistError>) -> Vec<OneilError> {
    let mut errors = Vec::new();

    for (path, import_error) in error_map.get_import_errors() {
        let error = convert_import_error(path, import_error);
        errors.push(error);
    }

    for (path, dep_errors) in error_map.get_circular_dependency_errors() {
        for dep_error in dep_errors {
            let error = convert_circular_dependency_error(path, dep_error);
            errors.push(error);
        }
    }

    for (path, model_error) in error_map.get_model_errors() {
        let mut model_errors = convert_model_errors(path, model_error);

        // sort the errors by offset so that the errors are in order of
        // appearance within the file
        model_errors.sort_by_key(|error| {
            let location = error.location();
            location.map(|location| location.offset())
        });

        errors.extend(model_errors);
    }

    errors
}

/// Converts a Python import error into a unified CLI error format
///
/// Creates an error message indicating that a referenced Python file does not exist.
/// This is used when Oneil models reference Python files that cannot be found.
///
/// # Arguments
///
/// * `python_path` - The path to the Python file that was expected to exist
/// * `error` - The error indicating the file does not exist
///
/// # Returns
///
/// Returns a new `Error` instance with a message indicating the Python file is missing.
///
/// # Panics
///
/// Panics if the Python path and error path do not match, which should never happen
/// in normal operation.
fn convert_import_error(python_path: &PythonPath, error: &DoesNotExistError) -> OneilError {
    assert_eq!(
        python_path.as_ref(),
        error.path(),
        "python path and error path should match"
    );

    OneilError::from_error(error, python_path.as_ref().to_path_buf())
}

/// Converts a circular dependency error into a unified CLI error format
///
/// Creates an error message showing the circular dependency chain that was detected
/// during model loading. This helps users identify and resolve circular references
/// between models.
///
/// # Arguments
///
/// * `model_path` - The path to the model where the circular dependency was detected
/// * `error` - The circular dependency error containing the dependency chain
///
/// # Returns
///
/// Returns a new `Error` instance with a message showing the circular dependency chain.
fn convert_circular_dependency_error(
    model_path: &ModelPath,
    error: &CircularDependencyError,
) -> OneilError {
    let path = model_path.as_ref();

    OneilError::from_error(error, path.to_path_buf())
}

/// Converts model loading errors into unified CLI errors
///
/// Handles both parsing errors and resolution errors that occur during model loading.
/// For parsing errors, it delegates to the appropriate conversion functions. For
/// resolution errors, it processes them in detail to provide specific error messages.
///
/// # Arguments
///
/// * `model_path` - The path to the model that contains the errors
/// * `errors` - The load error containing either parsing or resolution errors
///
/// # Returns
///
/// Returns a vector of `Error` instances for all errors found in the model.
fn convert_model_errors(
    model_path: &ModelPath,
    errors: &LoadError<LoadingError>,
) -> Vec<OneilError> {
    let path = model_path.as_ref();

    match errors {
        LoadError::ParseError(parse_error) => match parse_error {
            LoadingError::InvalidFile(error) => {
                let error = file::convert(path, error);
                vec![error]
            }
            LoadingError::Parser(errors_with_partial_result) => {
                let errors = &errors_with_partial_result.errors;
                parser::convert_all(path, errors)
            }
        },
        LoadError::ResolutionErrors(resolution_errors) => {
            convert_resolution_errors(model_path, resolution_errors)
        }
    }
}

/// Converts resolution errors into unified CLI errors
///
/// Processes various types of resolution errors that occur during model loading,
/// including submodel resolution, parameter resolution, and test resolution errors.
/// Attempts to read the source file to provide location information for the errors.
///
/// # Arguments
///
/// * `model_path` - The path to the model that contains the resolution errors
/// * `resolution_errors` - The collection of resolution errors to convert
///
/// # Returns
///
/// Returns a vector of `Error` instances for all resolution errors found.
///
/// # Note
///
/// This function attempts to read the source file to provide location information.
/// If the file cannot be read, it adds a file reading error and processes the
/// resolution errors without location information.
fn convert_resolution_errors(
    model_path: &ModelPath,
    resolution_errors: &ResolutionErrors,
) -> Vec<OneilError> {
    let mut errors = Vec::new();

    let path = model_path.as_ref();

    let source = fs::read_to_string(path);
    let source = match &source {
        Ok(source) => Some(source.as_str()),
        Err(error) => {
            let file_error = file::convert(path, &error);
            errors.push(file_error);
            None
        }
    };

    // convert import errors
    for (_python_path, _import_error) in resolution_errors.get_import_errors() {
        // These are intentionally ignored because they indicate that a python
        // file failed to resolve correctly. These errors should be indicated
        // by corresponding import errors in `convert_import_error`.
        ignore_error();
    }

    // convert submodel resolution errors
    for (_identifier, submodel_resolution_error) in
        resolution_errors.get_submodel_resolution_errors()
    {
        match submodel_resolution_error {
            SubmodelResolutionError::ModelHasError { .. } => {
                ignore_error();
            }

            SubmodelResolutionError::UndefinedSubmodel { .. } => {
                let error = match source {
                    Some(source) => OneilError::from_error_with_source(
                        submodel_resolution_error,
                        path.to_path_buf(),
                        source,
                    ),
                    None => OneilError::from_error(submodel_resolution_error, path.to_path_buf()),
                };
                errors.push(error);
            }
        }
    }

    // convert parameter resolution errors
    for (_identifier, parameter_resolution_errors) in
        resolution_errors.get_parameter_resolution_errors()
    {
        for parameter_resolution_error in parameter_resolution_errors {
            let error = match source {
                Some(source) => OneilError::from_error_with_source(
                    parameter_resolution_error,
                    path.to_path_buf(),
                    source,
                ),
                None => OneilError::from_error(parameter_resolution_error, path.to_path_buf()),
            };
            errors.push(error);
        }
    }

    // convert model test resolution errors
    for (_test_index, test_resolution_errors) in
        resolution_errors.get_model_test_resolution_errors()
    {
        for test_resolution_error in test_resolution_errors {
            let error =
                convert_variable_resolution_error(path, source, test_resolution_error.get_error());

            if let Some(error) = error {
                errors.push(error);
            }
        }
    }

    // convert submodel test input resolution errors
    for (_submodel_identifier, submodel_test_input_resolution_errors) in
        resolution_errors.get_submodel_test_input_resolution_errors()
    {
        for submodel_test_input_resolution_error in submodel_test_input_resolution_errors {
            match submodel_test_input_resolution_error {
                SubmodelTestInputResolutionError::VariableResolution(variable_resolution_error) => {
                    let error =
                        convert_variable_resolution_error(path, source, variable_resolution_error);

                    if let Some(error) = error {
                        errors.push(error);
                    }
                }
            }
        }
    }

    errors
}

/// Converts a variable resolution error into a unified CLI error format
///
/// Handles various types of variable resolution errors, including undefined variables,
/// type mismatches, and other variable-related issues. Provides source location
/// information when available.
///
/// # Arguments
///
/// * `path` - The path to the file containing the variable resolution error
/// * `source` - Optional source file contents for location calculation
/// * `variable_resolution_error` - The variable resolution error to convert
///
/// # Returns
///
/// Returns `Some(Error)` if the error should be reported, or `None` if the error
/// should be ignored (e.g., for certain types of resolution errors that are
/// handled elsewhere).
///
/// # Note
///
/// Some variable resolution errors are intentionally ignored because they are
/// secondary to other errors or are handled by different error reporting mechanisms.
fn convert_variable_resolution_error(
    path: &Path,
    source: Option<&str>,
    variable_resolution_error: &VariableResolutionError,
) -> Option<OneilError> {
    match variable_resolution_error {
        VariableResolutionError::UndefinedParameter { .. }
        | VariableResolutionError::UndefinedSubmodel { .. } => match source {
            Some(source) => {
                let error = OneilError::from_error_with_source(
                    variable_resolution_error,
                    path.to_path_buf(),
                    source,
                );
                Some(error)
            }
            None => {
                let error = OneilError::from_error(variable_resolution_error, path.to_path_buf());
                Some(error)
            }
        },
        VariableResolutionError::ModelHasError { .. } => {
            // This error is intentionally ignored because it indicates that the
            // model being referenced has errors, which will be reported separately.
            None
        }
        VariableResolutionError::ParameterHasError { .. } => {
            // This error is intentionally ignored because it indicates that the
            // parameter has errors, which will be reported separately.
            None
        }
        VariableResolutionError::SubmodelResolutionFailed { .. } => {
            // This error is intentionally ignored because it indicates that the
            // submodel resolution failed, which will be reported separately.
            None
        }
    }
}

/// Placeholder function for intentionally ignored errors
///
/// This function is used as a placeholder when certain errors are intentionally
/// ignored during error conversion. It serves as documentation that the error
/// is being handled elsewhere or is not relevant for user reporting.
///
/// # Note
///
/// This function does nothing and is used purely for documentation purposes
/// to indicate where errors are intentionally ignored.
pub fn ignore_error() {}
