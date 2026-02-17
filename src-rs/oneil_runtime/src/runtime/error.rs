//! Error reporting for models and parameters.

#![allow(clippy::pedantic, reason = "this is temporary")]

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_eval::EvalError;
use oneil_resolver::error::{ModelImportResolutionError, PythonImportResolutionError};
use oneil_shared::error::OneilError;
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::error::{ModelError, RuntimeErrors};

impl Runtime {
    /// Returns all errors associated with the given model, as well as any
    /// models that it references that have errors.
    ///
    /// Source or parsing failures are reported as a [`ModelError::FileError`].
    /// Resolution or evaluation failures are reported as [`ModelError::EvalErrors`].
    #[must_use]
    pub fn get_model_errors(&self, model_path: &Path) -> RuntimeErrors {
        let path_buf = model_path.to_path_buf();

        // Handle source errors
        //
        // If the source failed to load, then there can be no
        // other errors, so we return early
        let source_entry = self
            .source_cache
            .get_entry(model_path)
            .expect("source has already been attempted to be loaded");

        let source = match source_entry {
            Ok(source) => source,
            Err(source_err) => {
                let mut errors = RuntimeErrors::default();

                errors.add_model_error(
                    path_buf.clone(),
                    ModelError::FileError(vec![OneilError::from_error(source_err, path_buf)]),
                );

                return errors;
            }
        };

        // Get the AST errors, if any
        let ast_entry = self
            .ast_cache
            .get_entry(model_path)
            .expect("ast has already been attempted to be loaded");

        let ast_errors = match ast_entry {
            LoadResult::Failure => panic!(
                "this only occurs if the source load failed, which should have been handled above"
            ),
            LoadResult::Partial(_, parser_errors) => {
                let errors: Vec<OneilError> = parser_errors
                    .iter()
                    .map(|e| OneilError::from_error_with_source(e, path_buf.clone(), source))
                    .collect();

                Some(errors)
            }
            LoadResult::Success(_) => None,
        };

        // get the IR errors, if any
        let ir_errors = self
            .ir_cache
            .get_entry(model_path)
            .and_then(|entry| entry.error())
            .map(|errors| {
                // get the model import errors
                let mut model_import_errors = IndexMap::new();
                let mut models_with_errors = IndexSet::new();
                for (ref_name, (_submodel_name, ref_error)) in
                    errors.get_model_import_resolution_errors()
                {
                    if let Some(model_path) = get_model_path_from_model_import_error(ref_error) {
                        models_with_errors.insert(model_path);
                    }

                    let error =
                        OneilError::from_error_with_source(ref_error, path_buf.clone(), source);

                    model_import_errors.insert(ref_name.to_string(), error);
                }

                // get the Python import errors
                let mut python_import_errors = IndexMap::new();
                let mut python_imports_with_errors = IndexSet::new();
                for (python_path, err) in errors.get_python_import_resolution_errors() {
                    if let Some(python_path) = get_python_path_from_python_import_error(err) {
                        python_imports_with_errors.insert(python_path);
                    }

                    let error = OneilError::from_error_with_source(err, path_buf.clone(), source);

                    python_import_errors.insert(python_path.as_ref().to_path_buf(), error);
                }

                // get the parameter errors
                let mut parameter_errors = IndexMap::new();
                for (param_name, param_errs) in errors.get_parameter_resolution_errors() {
                    let errors: Vec<OneilError> = param_errs
                        .iter()
                        .map(|e| OneilError::from_error_with_source(e, path_buf.clone(), source))
                        .collect();

                    parameter_errors.insert(param_name.as_str().to_string(), errors);
                }

                // get the test errors
                let mut test_errors = Vec::new();
                for (_, test_errs) in errors.get_test_resolution_errors() {
                    let errors: Vec<OneilError> = test_errs
                        .iter()
                        .map(|e| OneilError::from_error_with_source(e, path_buf.clone(), source))
                        .collect();

                    test_errors.extend(errors);
                }

                (
                    models_with_errors,
                    python_imports_with_errors,
                    model_import_errors,
                    python_import_errors,
                    parameter_errors,
                    test_errors,
                )
            });

        // get the eval errors, if any
        let eval_errors = self
            .eval_cache
            .get_entry(model_path)
            .and_then(|entry| entry.error())
            .map(|errors| {
                let mut models_with_errors = IndexSet::new();

                // get the parameter errors
                let mut parameter_errors = IndexMap::new();
                for (name, param_errs) in &errors.parameters {
                    let models_with_errors_in_param: IndexSet<PathBuf> = param_errs
                        .iter()
                        .filter_map(|error| {
                            if let EvalError::ParameterHasError { model_path, .. } = error {
                                model_path.clone()
                            } else {
                                None
                            }
                        })
                        .collect();
                    models_with_errors.extend(models_with_errors_in_param);

                    let errors: Vec<OneilError> = param_errs
                        .iter()
                        .map(|e| OneilError::from_error(e, path_buf.clone()))
                        .collect();

                    parameter_errors.insert(name.clone(), errors);
                }

                // get the test errors
                let mut test_errors = Vec::new();
                for test_err in &errors.tests {
                    if let EvalError::ParameterHasError { model_path, .. } = test_err
                        && let Some(path) = model_path
                    {
                        models_with_errors.insert(path.clone());
                    }

                    let error =
                        OneilError::from_error_with_source(test_err, path_buf.clone(), source);

                    test_errors.push(error);
                }

                (models_with_errors, parameter_errors, test_errors)
            });

        // combine the IR and eval errors
        let (
            models_with_errors,
            python_imports_with_errors,
            model_import_errors,
            python_import_errors,
            parameter_errors,
            test_errors,
        ) = match (ir_errors, eval_errors) {
            (
                Some((
                    ir_models_with_errors,
                    ir_python_imports_with_errors,
                    ir_model_import_errors,
                    ir_python_import_errors,
                    ir_parameter_errors,
                    ir_test_errors,
                )),
                Some((eval_models_with_errors, eval_parameter_errors, eval_test_errors)),
            ) => (
                ir_models_with_errors
                    .intersection(&eval_models_with_errors)
                    .cloned()
                    .collect(),
                ir_python_imports_with_errors,
                ir_model_import_errors,
                ir_python_import_errors,
                ir_parameter_errors
                    .into_iter()
                    .chain(eval_parameter_errors)
                    .collect(),
                ir_test_errors.into_iter().chain(eval_test_errors).collect(),
            ),
            (
                Some((
                    ir_models_with_errors,
                    ir_python_imports_with_errors,
                    ir_model_import_errors,
                    ir_python_import_errors,
                    ir_parameter_errors,
                    ir_test_errors,
                )),
                None,
            ) => (
                ir_models_with_errors,
                ir_python_imports_with_errors,
                ir_model_import_errors,
                ir_python_import_errors,
                ir_parameter_errors,
                ir_test_errors,
            ),
            (None, Some((eval_models_with_errors, eval_parameter_errors, eval_test_errors))) => (
                eval_models_with_errors,
                IndexSet::new(),
                IndexMap::new(),
                IndexMap::new(),
                eval_parameter_errors,
                eval_test_errors,
            ),
            (None, None) => (
                IndexSet::new(),
                IndexSet::new(),
                IndexMap::new(),
                IndexMap::new(),
                IndexMap::new(),
                Vec::new(),
            ),
        };

        let mut errors = RuntimeErrors::new();

        // add the errors for models that are referenced
        for model_path in models_with_errors {
            let model_errors = self.get_model_errors(&model_path);
            errors.extend(model_errors);
        }

        // add the errors for Python imports that are referenced
        #[cfg(feature = "python")]
        for python_import_path in python_imports_with_errors {
            let python_import_errors = self.get_python_import_errors(&python_import_path);
            errors.extend(python_import_errors);
        }

        // if there are AST errors, add them as a file error
        if let Some(ast_errors) = ast_errors {
            errors.add_model_error(path_buf, ModelError::FileError(ast_errors));
            return errors;
        }

        // if there are other errors, add them as a model error
        if !model_import_errors.is_empty()
            || !python_import_errors.is_empty()
            || !parameter_errors.is_empty()
            || !test_errors.is_empty()
        {
            errors.add_model_error(
                path_buf,
                ModelError::EvalErrors {
                    model_import_errors,
                    python_import_errors,
                    parameter_errors,
                    test_errors,
                },
            );

            return errors;
        }

        errors
    }

    /// Returns errors for the given parameter in a model.
    ///
    /// Parameter-level errors indicate that the parameter could not be resolved or evaluated.
    #[must_use]
    pub fn get_parameter_errors(&self, _model_path: &Path, _parameter_name: &str) -> RuntimeErrors {
        RuntimeErrors::new()
    }

    /// Returns errors for the given Python import path.
    ///
    /// If the Python module failed to load (e.g. file not found or load error),
    /// returns a [`RuntimeErrors`] with a single [`ModelError::FileError`] entry.
    #[must_use]
    #[cfg(feature = "python")]
    pub fn get_python_import_errors(&self, python_import_path: &Path) -> RuntimeErrors {
        let path_buf = python_import_path.to_path_buf();
        let mut errors = RuntimeErrors::new();

        if let Some(load_err) = self.python_import_cache.get_error(python_import_path) {
            errors.add_model_error(
                path_buf.clone(),
                ModelError::FileError(vec![OneilError::from_error(load_err, path_buf)]),
            );
        }

        errors
    }
}

/// Returns the model path from a model import error when available.
fn get_model_path_from_model_import_error(err: &ModelImportResolutionError) -> Option<PathBuf> {
    match err {
        ModelImportResolutionError::ModelHasError { model_path, .. } => {
            Some(model_path.as_ref().to_path_buf())
        }

        ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path, ..
        } => Some(parent_model_path.as_ref().to_path_buf()),

        ModelImportResolutionError::ParentModelHasError { .. }
        | ModelImportResolutionError::DuplicateSubmodel { .. }
        | ModelImportResolutionError::DuplicateReference { .. } => None,
    }
}

/// Returns the Python path from a Python import error when available.
fn get_python_path_from_python_import_error(err: &PythonImportResolutionError) -> Option<PathBuf> {
    match err {
        PythonImportResolutionError::FailedValidation { python_path, .. } => {
            Some(python_path.as_ref().to_path_buf())
        }
        PythonImportResolutionError::DuplicateImport { .. }
        | PythonImportResolutionError::PythonNotEnabled { .. } => None,
    }
}
