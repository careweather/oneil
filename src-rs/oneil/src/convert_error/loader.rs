use std::{fs, path::Path};

use oneil_ir::reference::{ModelPath, PythonPath};
use oneil_model_loader::{
    ModelErrorMap,
    error::{
        CircularDependencyError, LoadError, ParameterResolutionError, ResolutionErrors,
        SubmodelResolutionError, SubmodelTestInputResolutionError, VariableResolutionError,
    },
};

use crate::{
    convert_error::{Error, file, parser},
    file_parser::{DoesNotExistError, LoadingError},
};

pub fn convert_map(error_map: &ModelErrorMap<LoadingError, DoesNotExistError>) -> Vec<Error> {
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

fn convert_import_error(python_path: &PythonPath, error: &DoesNotExistError) -> Error {
    assert_eq!(
        python_path.as_ref(),
        error.path(),
        "python path and error path should match"
    );

    let path = error.path();
    let message = format!("python file '{}' does not exist", path.display());

    Error::new(path.to_path_buf(), message)
}

fn convert_circular_dependency_error(
    model_path: &ModelPath,
    error: &CircularDependencyError,
) -> Error {
    let path = model_path.as_ref();

    let circular_dependency = error.circular_dependency();
    let circular_dependency_str = circular_dependency
        .iter()
        .map(|path| path.as_ref().display().to_string())
        .collect::<Vec<_>>()
        .join(" -> ");
    let message = format!(
        "circular dependency found in models - {}",
        circular_dependency_str
    );

    Error::new(path.to_path_buf(), message)
}

fn convert_model_errors(model_path: &ModelPath, errors: &LoadError<LoadingError>) -> Vec<Error> {
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

fn convert_resolution_errors(
    model_path: &ModelPath,
    resolution_errors: &ResolutionErrors,
) -> Vec<Error> {
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
            SubmodelResolutionError::ModelHasError(_model_path) => {
                ignore_error();
            }

            SubmodelResolutionError::UndefinedSubmodel(_model_path, identifier) => {
                let message = submodel_resolution_error.to_string();
                let start = identifier.span().start();
                let length = identifier.span().length();
                let location = source.map(|source| (source, start, length));
                let error = Error::new_from_span(path.to_path_buf(), message, location);
                errors.push(error);
            }
        }
    }

    // convert parameter resolution errors
    for (_identifier, parameter_resolution_errors) in
        resolution_errors.get_parameter_resolution_errors()
    {
        for parameter_resolution_error in parameter_resolution_errors {
            match parameter_resolution_error {
                ParameterResolutionError::CircularDependency(_identifiers) => {
                    // because this is a circular dependency, we don't have a specific
                    // location to report within the model
                    let message = parameter_resolution_error.to_string();
                    let error = Error::new(path.to_path_buf(), message);
                    errors.push(error);
                }

                ParameterResolutionError::VariableResolution(variable_resolution_error) => {
                    let error =
                        convert_variable_resolution_error(path, source, variable_resolution_error);

                    if let Some(error) = error {
                        errors.push(error);
                    }
                }
            }
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

fn convert_variable_resolution_error(
    path: &Path,
    source: Option<&str>,
    variable_resolution_error: &VariableResolutionError,
) -> Option<Error> {
    match variable_resolution_error {
        // these errors are propagated from other errors and are not relevant to the user
        VariableResolutionError::ModelHasError(_model_path) => None,
        VariableResolutionError::ParameterHasError(_identifier) => None,
        VariableResolutionError::SubmodelResolutionFailed(_identifier) => None,

        VariableResolutionError::UndefinedParameter(_model_path, identifier)
        | VariableResolutionError::UndefinedSubmodel(_model_path, identifier) => {
            let message = variable_resolution_error.to_string();
            let start = identifier.span().start();
            let length = identifier.span().length();
            let location = source.map(|source| (source, start, length));
            let error = Error::new_from_span(path.to_path_buf(), message, location);
            Some(error)
        }
    }
}

// This function is used to ignore errors that are not relevant to the user.
//
// Usually, this is because the error is a propagated error from another error,
// such as `ImportResolutionError` or `SubmodelResolutionError::ModelHasError`.
// TODO: add an option to convert *all* errors, don't ignore any
pub fn ignore_error() {}
