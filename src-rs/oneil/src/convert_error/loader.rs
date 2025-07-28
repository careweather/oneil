use std::fs;

use oneil_ir::reference::{ModelPath, PythonPath};
use oneil_model_loader::{
    ModelErrorMap,
    error::{
        CircularDependencyError, ImportResolutionError, LoadError, ResolutionErrors,
        SubmodelResolutionError,
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
        let model_errors = convert_model_errors(path, model_error);
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
    let source = match source {
        Ok(source) => Some(source.as_str()),
        Err(error) => {
            let file_error = file::convert(path, &error);
            errors.push(file_error);
            None
        }
    };

    for (python_path, import_error) in resolution_errors.get_import_errors() {
        // These are intentionally ignored because they indicate that a python
        // file failed to resolve correctly. These errors should be indicated
        // by corresponding import errors in `convert_import_error`.
        ignore_error();
    }

    for (identifier, submodel_resolution_error) in
        resolution_errors.get_submodel_resolution_errors()
    {
        match submodel_resolution_error {
            SubmodelResolutionError::ModelHasError(model_path) => {
                ignore_error();
            }
            SubmodelResolutionError::UndefinedSubmodel(model_path, identifier) => {
                let message = submodel_resolution_error.to_string();
                let start = identifier.span().start();
                let length = identifier.span().length();
                let location = source.map(|source| (source, start, length));
                let error = Error::new_from_span(path.to_path_buf(), message, location);
                errors.push(error);
            }
        }
    }

    for (identifier, parameter_resolution_errors) in
        resolution_errors.get_parameter_resolution_errors()
    {
        for parameter_resolution_error in parameter_resolution_errors {
            let message = parameter_resolution_error.to_string();
            let (start, length): (usize, usize) = todo!();
            let location = source.as_ref().map(|source| (source, start, length));
            let error = Error::new_from_span(path.to_path_buf(), message, location);
            errors.push(error);
        }
    }

    for (test_index, test_resolution_errors) in resolution_errors.get_model_test_resolution_errors()
    {
        for test_resolution_error in test_resolution_errors {
            let message = test_resolution_error.to_string();
            let (start, length): (usize, usize) = todo!();
            let location = source.as_ref().map(|source| (source, start, length));
            let error = Error::new_from_span(path.to_path_buf(), message, location);
            errors.push(error);
        }
    }

    for (submodel_identifier, submodel_test_input_resolution_errors) in
        resolution_errors.get_submodel_test_input_resolution_errors()
    {
        for submodel_test_input_resolution_error in submodel_test_input_resolution_errors {
            let message = submodel_test_input_resolution_error.to_string();
            let (start, length): (usize, usize) = todo!();
            let location = source.as_ref().map(|source| (source, start, length));
            let error = Error::new_from_span(path.to_path_buf(), message, location);
            errors.push(error);
        }
    }

    errors
}

// This function is used to ignore errors that are not relevant to the user.
//
// Usually, this is because the error is a propagated error from another error,
// such as `ImportResolutionError` or `SubmodelResolutionError::ModelHasError`.
pub fn ignore_error() {}
