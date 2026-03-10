//! Error reporting for models and parameters.

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use oneil_eval::{EvalError, EvalErrors};
use oneil_resolver::ResolutionErrorCollection;
use oneil_resolver::error::{
    ModelImportResolutionError, ParameterResolutionError, PythonImportResolutionError,
    VariableResolutionError,
};
use oneil_shared::error::OneilError;
use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::{
    error::PythonImportError,
    output::error::{ModelError, RuntimeErrors},
};

impl Runtime {
    /// Returns all errors associated with the given model, as well as any
    /// models that it references that have errors.
    ///
    /// Source or parsing failures are reported as a [`ModelError::FileError`].
    /// Resolution or evaluation failures are reported as [`ModelError::EvalErrors`].
    ///
    /// If `include_indirect_errors` is true, then errors from models that are referenced
    /// by the model are always included, regardless of whether they are referenced directly.
    ///
    /// For example, imagine there is an `x` in `model_a` that references `y` in `model_b`. Neither of
    /// these parameters have errors, but `model_b` has a parameter `z` that is used in a test, and both
    /// `z` and the test have errors. If `include_indirect_errors` is true, then the errors from `model_b`
    /// will be included in the errors for `model_a`. If `include_indirect_errors` is false, then the errors from `model_b`
    /// will not be included in the errors for `model_a` since there is no direct reference to `z` or to the test.
    #[must_use]
    pub(super) fn get_model_errors(
        &self,
        model_path: &Path,
        include_indirect_errors: bool,
    ) -> RuntimeErrors {
        let path_buf = model_path.to_path_buf();

        // Handle source errors
        //
        // If the source failed to load, then there can be no
        // other errors, so we return early
        let Some(source_entry) = self.source_cache.get_entry(model_path) else {
            return RuntimeErrors::default();
        };

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
        let Some(ast_entry) = self.ast_cache.get_entry(model_path) else {
            return RuntimeErrors::default();
        };

        let ast_errors = match ast_entry {
            LoadResult::Failure => return RuntimeErrors::default(),
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
            .map(|errors| collect_ir_errors(errors, &path_buf, source, include_indirect_errors));

        // get the eval errors, if any
        let eval_errors = self
            .eval_cache
            .get_entry(model_path)
            .and_then(|entry| entry.error())
            .map(|errors| collect_eval_errors(errors, &path_buf, source, include_indirect_errors));

        // combine the IR and eval errors
        let MergedErrors {
            models_with_errors,
            python_imports_with_errors,
            model_import_errors,
            python_import_errors,
            parameter_errors,
            test_errors,
        } = merge_ir_and_eval_errors(ir_errors, eval_errors);

        let mut errors = RuntimeErrors::new();

        // add the errors for models that are referenced
        for model_path in models_with_errors {
            let model_errors = self.get_model_errors(&model_path, include_indirect_errors);
            errors.extend(model_errors);
        }

        // add the errors for Python imports that are referenced
        #[cfg(feature = "python")]
        for python_import_path in python_imports_with_errors {
            let python_import_errors = self.get_python_import_errors(&python_import_path);
            errors.extend(python_import_errors);
        }

        if let Some(ast_errors) = ast_errors {
            // if there are AST errors, add them as a file error
            errors.add_model_error(path_buf, ModelError::FileError(ast_errors));
        } else if !model_import_errors.is_empty()
            || !python_import_errors.is_empty()
            || !parameter_errors.is_empty()
            || !test_errors.is_empty()
        {
            // if there are other errors, add them as a model error
            errors.add_model_error(
                path_buf,
                ModelError::EvalErrors {
                    model_import_errors: Box::new(model_import_errors),
                    python_import_errors: Box::new(python_import_errors),
                    parameter_errors: Box::new(parameter_errors),
                    test_errors,
                },
            );
        }

        errors
    }

    /// Returns errors for the given Python import path.
    ///
    /// If the source failed to load or the Python module failed to load (e.g. file not found or load error),
    /// returns a [`RuntimeErrors`] with [`ModelError::FileError`] entries for each.
    #[must_use]
    #[cfg(feature = "python")]
    pub(super) fn get_python_import_errors(&self, python_import_path: &Path) -> RuntimeErrors {
        let path_buf = python_import_path.to_path_buf();
        let mut errors = RuntimeErrors::new();

        if let Some(Err(source_err)) = self.source_cache.get_entry(python_import_path) {
            errors.add_model_error(
                path_buf.clone(),
                ModelError::FileError(vec![OneilError::from_error(source_err, path_buf.clone())]),
            );
        }

        if let Some(Err(load_err)) = self.python_import_cache.get_entry(python_import_path)
            && let PythonImportError::LoadFailed(load_err) = load_err
        {
            errors.add_model_error(
                path_buf.clone(),
                ModelError::FileError(vec![OneilError::from_error(load_err, path_buf)]),
            );
        }

        errors
    }
}

/// Result of collecting errors from IR resolution.
#[expect(
    clippy::struct_field_names,
    reason = "removing 'errors' might be confusing"
)]
#[derive(Debug)]
struct IrErrorsResult {
    /// Model paths that have errors (for recursive collection).
    models_with_errors: IndexSet<PathBuf>,
    /// Python import paths that have errors (for recursive collection).
    python_imports_with_errors: IndexSet<PathBuf>,
    /// Model import resolution errors by reference name.
    model_import_errors: IndexMap<String, OneilError>,
    /// Python import resolution errors by path.
    python_import_errors: IndexMap<PathBuf, OneilError>,
    /// Parameter resolution errors by parameter name.
    parameter_errors: IndexMap<String, Vec<OneilError>>,
    /// Test resolution errors.
    test_errors: Vec<OneilError>,
}

/// Collects resolution errors from IR into structured error data and model/python path sets.
///
/// See [`Runtime::get_model_errors`] for more details on the `include_indirect_errors` parameter.
fn collect_ir_errors(
    errors: &ResolutionErrorCollection,
    path: &Path,
    source: &str,
    include_indirect_errors: bool,
) -> IrErrorsResult {
    // collect model import errors
    let mut model_import_errors = IndexMap::new();
    let mut models_with_errors = IndexSet::new();

    if include_indirect_errors {
        for (ref_name, (_submodel_name, ref_error)) in errors.get_model_import_resolution_errors() {
            if let Some(model_path) = get_model_path_from_model_import_error(ref_error) {
                models_with_errors.insert(model_path);
            }

            let error = OneilError::from_error_with_source(ref_error, path.to_path_buf(), source);
            model_import_errors.insert(ref_name.to_string(), error);
        }
    }

    // collect Python import errors
    let mut python_import_errors = IndexMap::new();
    let mut python_imports_with_errors = IndexSet::new();
    for (python_path, err) in errors.get_python_import_resolution_errors() {
        if let Some(python_path) = get_python_path_from_python_import_error(err) {
            python_imports_with_errors.insert(python_path);
        }

        let error = OneilError::from_error_with_source(err, path.to_path_buf(), source);
        python_import_errors.insert(python_path.as_ref().to_path_buf(), error);
    }

    let has_python_import_errors = !python_import_errors.is_empty();

    // collect parameter errors
    let mut parameter_errors = IndexMap::new();
    for (param_name, param_errs) in errors.get_parameter_resolution_errors() {
        let models_with_errors_in_param: IndexSet<PathBuf> = param_errs
            .iter()
            .filter_map(|error| {
                if let ParameterResolutionError::VariableResolution(
                    VariableResolutionError::ModelHasError { path, .. },
                ) = error
                {
                    Some(path.as_ref().to_path_buf())
                } else {
                    None
                }
            })
            .collect();
        models_with_errors.extend(models_with_errors_in_param);

        let oneil_errors: Vec<OneilError> = param_errs
            .iter()
            .filter(|e| !(has_python_import_errors && is_undefined_function_error(e)))
            .map(|e| OneilError::from_error_with_source(e, path.to_path_buf(), source))
            .collect();
        parameter_errors.insert(param_name.as_str().to_string(), oneil_errors);
    }

    // collect test errors
    let mut test_errors = Vec::new();
    for (_, test_errs) in errors.get_test_resolution_errors() {
        let models_with_errors_in_test: IndexSet<PathBuf> = test_errs
            .iter()
            .filter_map(|error| {
                if let VariableResolutionError::ModelHasError { path, .. } = error {
                    Some(path.as_ref().to_path_buf())
                } else {
                    None
                }
            })
            .collect();
        models_with_errors.extend(models_with_errors_in_test);

        let oneil_errors: Vec<OneilError> = test_errs
            .iter()
            .map(|e| OneilError::from_error_with_source(e, path.to_path_buf(), source))
            .collect();
        test_errors.extend(oneil_errors);
    }

    IrErrorsResult {
        models_with_errors,
        python_imports_with_errors,
        model_import_errors,
        python_import_errors,
        parameter_errors,
        test_errors,
    }
}

const fn is_undefined_function_error(error: &ParameterResolutionError) -> bool {
    matches!(
        error,
        ParameterResolutionError::VariableResolution(
            VariableResolutionError::UndefinedFunction { .. }
        )
    )
}

/// Result of collecting errors from evaluation.
#[expect(
    clippy::struct_field_names,
    reason = "removing 'errors' might be confusing"
)]
#[derive(Debug)]
struct EvalErrorsResult {
    /// Model paths that have errors (for recursive collection).
    models_with_errors: IndexSet<PathBuf>,
    /// Parameter evaluation errors by parameter name.
    parameter_errors: IndexMap<String, Vec<OneilError>>,
    /// Test evaluation errors.
    test_errors: Vec<OneilError>,
}

/// Collects evaluation errors into structured error data and model path set.
///
/// See [`Runtime::get_model_errors`] for more details on the `include_indirect_errors` parameter.
fn collect_eval_errors(
    errors: &EvalErrors,
    path: &Path,
    source: &str,
    include_indirect_errors: bool,
) -> EvalErrorsResult {
    let mut models_with_errors = IndexSet::new();

    if include_indirect_errors {
        for reference_path in &errors.references {
            models_with_errors.insert(reference_path.clone());
        }
    }

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

        let oneil_errors: Vec<OneilError> = param_errs
            .iter()
            .map(|e| OneilError::from_error_with_source(e, path.to_path_buf(), source))
            .collect();
        parameter_errors.insert(name.clone(), oneil_errors);
    }

    let mut test_errors = Vec::new();
    for test_err in &errors.tests {
        if let EvalError::ParameterHasError { model_path, .. } = test_err
            && let Some(p) = model_path
        {
            models_with_errors.insert(p.clone());
        }

        let error = OneilError::from_error_with_source(test_err, path.to_path_buf(), source);
        test_errors.push(error);
    }

    EvalErrorsResult {
        models_with_errors,
        parameter_errors,
        test_errors,
    }
}

/// Result of merging IR and eval error results.
#[expect(
    clippy::struct_field_names,
    reason = "removing 'errors' might be confusing"
)]
#[derive(Debug)]
struct MergedErrors {
    /// Model paths that have errors (for recursive collection).
    pub models_with_errors: IndexSet<PathBuf>,
    /// Python import paths that have errors (for recursive collection).
    pub python_imports_with_errors: IndexSet<PathBuf>,
    /// Model import errors by reference name.
    pub model_import_errors: IndexMap<String, OneilError>,
    /// Python import errors by path.
    pub python_import_errors: IndexMap<PathBuf, OneilError>,
    /// Parameter errors by parameter name.
    pub parameter_errors: IndexMap<String, Vec<OneilError>>,
    /// Test errors.
    pub test_errors: Vec<OneilError>,
}

/// Merges optional IR and eval error results into a single combined result.
///
/// When both are present, model paths are intersected and parameter/test errors are concatenated.
fn merge_ir_and_eval_errors(
    ir_errors: Option<IrErrorsResult>,
    eval_errors: Option<EvalErrorsResult>,
) -> MergedErrors {
    match (ir_errors, eval_errors) {
        (Some(ir), Some(eval)) => MergedErrors {
            models_with_errors: ir
                .models_with_errors
                .union(&eval.models_with_errors)
                .cloned()
                .collect(),
            python_imports_with_errors: ir.python_imports_with_errors,
            model_import_errors: ir.model_import_errors,
            python_import_errors: ir.python_import_errors,
            // note that in the case of the same parameter/test having errors in both IR and eval,
            // the IR errors are preferred because `ir` comes later in the chain
            parameter_errors: eval
                .parameter_errors
                .into_iter()
                .chain(ir.parameter_errors)
                .collect(),
            test_errors: ir.test_errors.into_iter().chain(eval.test_errors).collect(),
        },

        (Some(ir), None) => MergedErrors {
            models_with_errors: ir.models_with_errors,
            python_imports_with_errors: ir.python_imports_with_errors,
            model_import_errors: ir.model_import_errors,
            python_import_errors: ir.python_import_errors,
            parameter_errors: ir.parameter_errors,
            test_errors: ir.test_errors,
        },

        (None, Some(eval)) => MergedErrors {
            models_with_errors: eval.models_with_errors,
            python_imports_with_errors: IndexSet::new(),
            model_import_errors: IndexMap::new(),
            python_import_errors: IndexMap::new(),
            parameter_errors: eval.parameter_errors,
            test_errors: eval.test_errors,
        },

        (None, None) => MergedErrors {
            models_with_errors: IndexSet::new(),
            python_imports_with_errors: IndexSet::new(),
            model_import_errors: IndexMap::new(),
            python_import_errors: IndexMap::new(),
            parameter_errors: IndexMap::new(),
            test_errors: Vec::new(),
        },
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
