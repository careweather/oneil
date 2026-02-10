use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use oneil_ir as ir;
use oneil_output as output;
use oneil_shared::span::Span;

use crate::error::{EvalError, EvalErrors};

/// Error indicating that an IR model could not be loaded.
#[derive(Debug, Clone, Copy)]
pub struct IrLoadError;

/// Context provided by the runtime for resolving IR, builtins, and units during evaluation.
pub trait ExternalEvaluationContext {
    /// Returns the IR model at the given path if it has been loaded.
    fn lookup_ir(&self, path: impl AsRef<Path>) -> Option<Result<&ir::Model, IrLoadError>>;

    /// Returns the value of a builtin variable by identifier, if it exists.
    fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Option<&output::Value>;

    /// Evaluates a builtin function by identifier with the given arguments, if it exists.
    ///
    /// If the function does not exist, returns `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if there was an error evaluating the builtin function.
    fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Vec<EvalError>>>;

    /// Evaluates an imported function by identifier with the given arguments, if it exists.
    ///
    /// If the function does not exist, returns `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if there was an error evaluating the imported function.
    fn evaluate_imported_function(
        &self,
        python_path: &ir::PythonPath,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Option<Result<output::Value, Vec<EvalError>>>;

    /// Returns a unit by name if it is defined in the builtin context.
    fn lookup_unit(&self, name: &str) -> Option<&output::Unit>;

    /// Returns the map of available unit prefixes (e.g. "k" -> 1000.0).
    fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)>;
}

/// Represents a model in progress of being evaluated.
#[derive(Debug, Clone)]
struct ModelInProgress {
    had_resolution_errors: bool,
    parameters: IndexMap<String, Result<output::Parameter, Vec<EvalError>>>,
    submodels: IndexMap<String, String>,
    references: IndexMap<String, PathBuf>,
    tests: Vec<Result<output::Test, Vec<EvalError>>>,
}

impl ModelInProgress {
    /// Creates a new empty model.
    pub fn new() -> Self {
        Self {
            had_resolution_errors: false,
            parameters: IndexMap::new(),
            submodels: IndexMap::new(),
            references: IndexMap::new(),
            tests: Vec::new(),
        }
    }
}

impl Default for ModelInProgress {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation context that tracks models, their parameters, dependencies, and builtin functions.
///
/// The context maintains state during evaluation, including:
/// - Evaluated models and their parameters
/// - Active models
/// - External context
#[derive(Debug)]
pub struct EvalContext<'external, E: ExternalEvaluationContext> {
    models: IndexMap<PathBuf, ModelInProgress>,
    active_models: Vec<PathBuf>,
    external_context: &'external mut E,
}

impl<'external, E: ExternalEvaluationContext> EvalContext<'external, E> {
    /// Creates a new evaluation context with the given builtin functions.
    #[must_use]
    pub fn new(external_context: &'external mut E) -> Self {
        Self {
            models: IndexMap::new(),
            active_models: Vec::new(),
            external_context,
        }
    }

    /// Consumes the context and returns the accumulated models and errors.
    ///
    /// Each entry maps a model path to its evaluated [`Model`] and any
    /// [`ModelError`]s that occurred during evaluation (e.g. from parameters or
    /// tests that failed).
    #[must_use]
    pub fn into_result(self) -> IndexMap<PathBuf, Result<output::Model, EvalErrors>> {
        let mut result = IndexMap::new();

        // for each model, collect the parameters and tests, and any errors
        for (path, model) in self.models {
            // collect the parameters and any errors
            let mut parameters = IndexMap::new();
            let mut parameter_errors = IndexMap::new();
            for (name, result) in model.parameters {
                match result {
                    Ok(param) => {
                        parameters.insert(name, param);
                    }

                    Err(errs) => {
                        parameter_errors.insert(name, errs);
                    }
                }
            }

            // collect the tests and any errors
            let mut tests = Vec::new();
            let mut test_errors = Vec::new();
            for test in model.tests {
                match test {
                    Ok(test) => tests.push(test),
                    Err(errs) => {
                        test_errors.extend(errs);
                    }
                }
            }

            // create the output model
            let output_model = output::Model {
                path: path.clone(),
                submodels: model.submodels,
                references: model.references,
                parameters,
                tests,
            };

            if parameter_errors.is_empty() && test_errors.is_empty() && !model.had_resolution_errors
            {
                result.insert(path, Ok(output_model));
            } else {
                result.insert(
                    path,
                    Err(EvalErrors {
                        partial_result: output_model,
                        had_resolution_errors: model.had_resolution_errors,
                        parameters: parameter_errors,
                        tests: test_errors,
                    }),
                );
            }
        }

        result
    }

    /// Looks up an IR model by path.
    ///
    /// # Panics
    ///
    /// Panics if the model is not found. This should never be the case.
    pub fn get_ir(&self, path: impl AsRef<Path>) -> Result<ir::Model, IrLoadError> {
        self.external_context
            .lookup_ir(path)
            .expect("model should be found")
            // TODO: figure out how to get rid of this clone
            .cloned()
    }

    /// Looks up the given builtin variable and returns the corresponding value.
    ///
    /// # Panics
    ///
    /// Panics if the builtin value is not defined. This should never be the case.
    /// If it is, then there is a bug either in the model resolver when it resolves builtin variables
    /// or in the builtin map when it defines the builtin values.
    #[must_use]
    pub fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> output::Value {
        self.external_context
            .lookup_builtin_variable(identifier)
            .expect("builtin value should be defined (checked during resolution)")
            .clone()
    }

    /// Looks up a parameter value in the current model.
    ///
    /// # Panics
    ///
    /// Panics if no current model is set or if the parameter is not defined in the model.
    pub fn lookup_parameter_value(
        &self,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<output::Value, Vec<EvalError>> {
        let current_model = self
            .active_models
            .last()
            .expect("current model should be set when looking up a parameter");

        self.lookup_model_parameter_value_internal(current_model, parameter_name, variable_span)
    }

    /// Looks up a parameter value in a specific model.
    pub fn lookup_model_parameter_value(
        &self,
        model: &ir::ModelPath,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<output::Value, Vec<EvalError>> {
        self.lookup_model_parameter_value_internal(model.as_ref(), parameter_name, variable_span)
    }

    fn lookup_model_parameter_value_internal(
        &self,
        model_path: &Path,
        parameter_name: &ir::ParameterName,
        variable_span: Span,
    ) -> Result<output::Value, Vec<EvalError>> {
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

    /// Evaluates a builtin function with the given arguments.
    ///
    /// # Panics
    ///
    /// Panics if the builtin function is not defined. This should never be the case.
    pub fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Result<output::Value, Vec<EvalError>> {
        self.external_context
            .evaluate_builtin_function(identifier, identifier_span, args)
            .expect("builtin function should be defined (checked during resolution)")
    }

    /// Evaluates an imported function with the given arguments.
    pub fn evaluate_imported_function(
        &self,
        python_path: &ir::PythonPath,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(output::Value, Span)>,
    ) -> Result<output::Value, Vec<EvalError>> {
        self.external_context
            .evaluate_imported_function(python_path, identifier, identifier_span, args)
            .expect("imported function should be defined (checked during resolution)")
    }

    /// Looks up a unit by name.
    ///
    /// # Panics
    ///
    /// Panics if the unit is not defined. This should never be the case.
    #[must_use]
    pub fn lookup_unit(&self, name: &str) -> Option<output::Unit> {
        self.external_context.lookup_unit(name).cloned()
    }

    /// Returns the available unit prefixes.
    pub fn available_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.external_context.available_prefixes()
    }

    /// Pushes the active model for evaluation.
    ///
    /// Creates a new model entry if it doesn't exist.
    pub fn push_active_model(&mut self, model_path: PathBuf) {
        self.models.entry(model_path.clone()).or_default();

        self.active_models.push(model_path);
    }

    /// Clears the active model.
    pub fn pop_active_model(&mut self, model_path: &Path) {
        assert_eq!(self.active_models.last(), Some(&model_path.to_path_buf()));

        self.active_models.pop();
    }

    /// Adds a parameter evaluation result to the current model.
    ///
    /// # Panics
    ///
    /// Panics if no current model is set or if the current model was not created.
    pub fn add_parameter_result(
        &mut self,
        parameter_name: String,
        result: Result<output::Parameter, Vec<EvalError>>,
    ) {
        // TODO: Maybe use type state pattern to enforce this?
        let Some(current_model) = self.active_models.last() else {
            panic!("current model should be set when adding a parameter result");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.parameters.insert(parameter_name, result);
    }

    /// Adds a submodel to the current model.
    ///
    /// # Panics
    ///
    /// Panics if no current model is set or if the current model was not created.
    pub(crate) fn add_submodel(&mut self, submodel_name: &str, submodel_reference_name: &str) {
        let Some(current_model) = self.active_models.last() else {
            panic!("current model should be set when adding a submodel");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.submodels.insert(
            submodel_name.to_string(),
            submodel_reference_name.to_string(),
        );
    }

    /// Adds a reference to the current model.
    ///
    /// # Panics
    ///
    /// Panics if no current model is set or if the current model was not created.
    pub(crate) fn add_reference(&mut self, reference_name: &str, reference_path: &ir::ModelPath) {
        let Some(current_model) = self.active_models.last() else {
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

    /// Adds a test evaluation result to the current model.
    ///
    /// # Panics
    ///
    /// Panics if no current model is set or if the current model was not created.
    pub(crate) fn add_test_result(&mut self, test_result: Result<output::Test, Vec<EvalError>>) {
        let Some(current_model) = self.active_models.last() else {
            panic!("current model should be set when adding a test result");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.tests.push(test_result);
    }

    /// Marks the active model as having a resolution error.
    pub(crate) fn mark_ir_load_error_for_active_model(&mut self) {
        let Some(current_model) = self.active_models.last() else {
            panic!("current model should be set when marking an IR load error");
        };

        let model = self
            .models
            .get_mut(current_model)
            .expect("current model should be created when set");

        model.had_resolution_errors = true;
    }
}
