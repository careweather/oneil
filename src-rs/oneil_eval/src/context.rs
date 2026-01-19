use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;

use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    builtin::{BuiltinFunction, BuiltinMap},
    error::{EvalError, ModelError},
    result,
    value::{Unit, Value},
};

#[derive(Debug, Clone)]
pub struct Model {
    parameters: IndexMap<String, Result<result::Parameter, Vec<EvalError>>>,
    submodels: IndexMap<String, PathBuf>,
    references: IndexMap<String, PathBuf>,
    tests: Vec<Result<result::Test, Vec<EvalError>>>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            parameters: IndexMap::new(),
            submodels: IndexMap::new(),
            references: IndexMap::new(),
            tests: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalContext<F: BuiltinFunction> {
    models: IndexMap<PathBuf, Model>,
    // TODO: update this to hold the actual Python import results
    python_imports: IndexMap<PathBuf, Result<(), EvalError>>,
    current_model: Option<PathBuf>,
    active_python_imports: HashSet<PathBuf>,
    active_references: HashSet<PathBuf>,
    builtins: BuiltinMap<F>,
}

impl<F: BuiltinFunction> EvalContext<F> {
    pub fn new(builtins: BuiltinMap<F>) -> Self {
        Self {
            models: IndexMap::new(),
            python_imports: IndexMap::new(),
            current_model: None,
            active_python_imports: HashSet::new(),
            active_references: HashSet::new(),
            builtins,
        }
    }

    /// Looks up the given builtin variable and returns the corresponding value.
    ///
    /// # Panics
    ///
    /// Panics if the builtin value is not defined. This should never be the case.
    /// If it is, then there is a bug either in the model resolver when it resolves builtin variables
    /// or in the builtin map when it defines the builtin values.
    pub fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Value {
        self.builtins
            .values
            .get(identifier.as_str())
            .expect("builtin value should be defined")
            .clone()
    }

    pub fn lookup_parameter_value(
        &self,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<Value, Vec<EvalError>> {
        let current_model = self
            .current_model
            .as_ref()
            .expect("current model should be set when looking up a parameter");

        self.lookup_model_parameter_value_internal(current_model, parameter_name, variable_span)
    }

    pub fn lookup_model_parameter_value(
        &self,
        model: &ir::ModelPath,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<Value, Vec<EvalError>> {
        self.lookup_model_parameter_value_internal(model.as_ref(), parameter_name, variable_span)
    }

    fn lookup_model_parameter_value_internal(
        &self,
        model_path: &Path,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<Value, Vec<EvalError>> {
        let model = self
            .models
            .get(model_path)
            .expect("current model should be created when set");

        model
            .parameters
            .get(parameter_name.as_str())
            .expect("parameter should be defined")
            .clone()
            .map(|parameter| parameter.value)
            .map_err(|_errors| {
                vec![EvalError::ParameterHasError {
                    parameter_name: parameter_name.as_str().to_string(),
                    variable_span,
                }]
            })
    }

    // TODO: figure out what error this should actually be
    pub fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        self.builtins
            .functions
            .get(identifier.as_str())
            .expect("builtin function should be defined")
            .call(identifier_span, args)
    }

    pub fn evaluate_imported_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        let _ = (self, identifier, args);
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("imported function".to_string()),
            will_be_supported: true,
        }])
    }

    pub fn lookup_unit(&self, name: &str) -> Option<Unit> {
        self.builtins.units.get(name).cloned()
    }

    pub const fn available_prefixes(&self) -> &IndexMap<String, f64> {
        &self.builtins.prefixes
    }

    pub fn load_python_import(&mut self, python_path: PathBuf, python_import_span: Span) {
        self.python_imports.insert(
            python_path,
            Err(EvalError::Unsupported {
                relevant_span: python_import_span,
                feature_name: Some("Python import".to_string()),
                will_be_supported: true,
            }),
        );
    }

    pub fn set_active_model(&mut self, model_path: PathBuf) {
        self.models
            .entry(model_path.clone())
            .or_insert_with(Model::new);

        self.current_model = Some(model_path);
    }

    pub fn clear_active_model(&mut self) {
        self.current_model = None;
    }

    pub fn clear_active_python_imports(&mut self) {
        self.active_python_imports.clear();
    }

    pub fn activate_python_import(&mut self, python_import: PathBuf) {
        self.active_python_imports.insert(python_import);
    }

    pub fn add_parameter_result(
        &mut self,
        parameter_name: String,
        result: Result<result::Parameter, Vec<EvalError>>,
    ) {
        // TODO: Maybe use type state pattern to enforce this?
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a parameter result");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.parameters.insert(parameter_name, result);
    }

    pub fn add_submodel(&mut self, submodel_name: &str, submodel_import: &ir::ModelPath) {
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a submodel");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.submodels.insert(
            submodel_name.to_string(),
            submodel_import.as_ref().to_path_buf(),
        );
    }

    pub fn add_reference(&mut self, reference_name: &str, reference_path: &ir::ModelPath) {
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a reference");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.references.insert(
            reference_name.to_string(),
            reference_path.as_ref().to_path_buf(),
        );
    }

    pub fn add_test_result(&mut self, test_result: Result<result::Test, Vec<EvalError>>) {
        let Some(current_model) = self.current_model.as_ref() else {
            panic!("current model should be set when adding a test result");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.tests.push(test_result);
    }

    pub fn activate_reference(&mut self, reference: PathBuf) {
        self.active_references.insert(reference);
    }

    /// Gets the result of a model.
    ///
    /// Returns a model with all valid results, as well as
    /// a list of any errors that occurred during evaluation.
    ///
    /// If no errors occurred, the list of errors will be empty.
    ///
    /// # Panics
    ///
    /// Panics if the model is not found.
    pub fn get_model_result(&self, model_path: &Path) -> (result::Model, Vec<ModelError>) {
        let Some(model) = self.models.get(model_path) else {
            panic!("model should have been created during evaluation");
        };

        let (submodels, submodel_errors) = self.collect_submodels(model, model_path);
        let (references, reference_errors) = self.collect_references(model, model_path);
        let (parameters, parameter_errors) = Self::collect_parameters(model, model_path);
        let (tests, test_errors) = Self::collect_tests(model, model_path);

        let model_result = result::Model {
            path: model_path.to_path_buf(),
            submodels,
            references,
            parameters,
            tests,
        };

        let errors = [
            submodel_errors,
            reference_errors,
            parameter_errors,
            test_errors,
        ]
        .concat();

        (model_result, errors)
    }

    /// Collects submodel results recursively.
    ///
    /// Returns a tuple of (successful submodels, errors).
    fn collect_submodels(
        &self,
        model: &Model,
        _model_path: &Path,
    ) -> (IndexMap<String, result::Model>, Vec<ModelError>) {
        model
            .submodels
            .iter()
            .map(|(name, submodel_path)| {
                let (submodel, submodel_errors) = self.get_model_result(submodel_path);
                (name.clone(), submodel, submodel_errors)
            })
            .fold(
                (IndexMap::new(), Vec::new()),
                |(mut submodels, mut submodel_errors), (name, submodel, errors)| {
                    submodels.insert(name, submodel);
                    submodel_errors.extend(errors);
                    (submodels, submodel_errors)
                },
            )
    }

    /// Collects reference results recursively.
    ///
    /// Returns a tuple of (successful references, errors).
    fn collect_references(
        &self,
        model: &Model,
        _model_path: &Path,
    ) -> (IndexMap<String, result::Model>, Vec<ModelError>) {
        model
            .references
            .iter()
            .map(|(name, reference_path)| {
                let (reference, reference_errors) = self.get_model_result(reference_path);
                (name.clone(), reference, reference_errors)
            })
            .fold(
                (IndexMap::new(), Vec::new()),
                |(mut references, mut reference_errors), (name, reference, errors)| {
                    references.insert(name, reference);
                    reference_errors.extend(errors);
                    (references, reference_errors)
                },
            )
    }

    /// Collects parameter results.
    ///
    /// Returns a tuple of (successful parameters, errors).
    fn collect_parameters(
        model: &Model,
        model_path: &Path,
    ) -> (IndexMap<String, result::Parameter>, Vec<ModelError>) {
        model
            .parameters
            .iter()
            .map(|(name, parameter_result)| {
                parameter_result
                    .clone()
                    .map(|parameter_result| (name.clone(), parameter_result))
            })
            .fold(
                (IndexMap::new(), Vec::new()),
                |(mut parameters, mut parameter_errors), result| match result {
                    Ok((name, parameter_result)) => {
                        parameters.insert(name, parameter_result);
                        (parameters, parameter_errors)
                    }
                    Err(errors) => {
                        let errors = errors.into_iter().map(|error| ModelError {
                            model_path: model_path.to_path_buf(),
                            error,
                        });

                        parameter_errors.extend(errors);
                        (parameters, parameter_errors)
                    }
                },
            )
    }

    /// Collects test results.
    ///
    /// Returns a tuple of (successful tests, errors).
    fn collect_tests(model: &Model, model_path: &Path) -> (Vec<result::Test>, Vec<ModelError>) {
        model.tests.iter().fold(
            (Vec::new(), Vec::new()),
            |(mut tests, mut test_errors), test_result| match test_result.clone() {
                Ok(test_result) => {
                    tests.push(test_result);
                    (tests, test_errors)
                }
                Err(errors) => {
                    let errors = errors.into_iter().map(|error| ModelError {
                        model_path: model_path.to_path_buf(),
                        error,
                    });

                    test_errors.extend(errors);
                    (tests, test_errors)
                }
            },
        )
    }
}
