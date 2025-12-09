use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use oneil_ir as ir;

use crate::{
    builtin::BuiltinFunction,
    builtin::BuiltinMap,
    error::EvalError,
    value::{SizedUnit, Value},
};

#[derive(Debug, Clone)]
pub struct Model {
    parameters: HashMap<String, Result<Value, Vec<EvalError>>>,
    submodels: HashMap<String, PathBuf>,
    tests: Vec<Result<Value, Vec<EvalError>>>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            submodels: HashMap::new(),
            tests: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalContext<F: BuiltinFunction> {
    models: HashMap<PathBuf, Model>,
    // TODO: update this to hold the actual Python import results
    python_imports: HashMap<PathBuf, Result<(), EvalError>>,
    current_model: Option<PathBuf>,
    active_python_imports: HashSet<PathBuf>,
    active_references: HashSet<PathBuf>,
    builtin_values: HashMap<String, Value>,
    builtin_functions: HashMap<String, F>,
    unit_map: HashMap<String, SizedUnit>,
    prefixes: HashMap<String, f64>,
}

impl<F: BuiltinFunction> EvalContext<F> {
    pub fn new(builtins: &impl BuiltinMap<F>) -> Self {
        Self {
            models: HashMap::new(),
            python_imports: HashMap::new(),
            current_model: None,
            active_python_imports: HashSet::new(),
            active_references: HashSet::new(),
            builtin_values: builtins.builtin_values(),
            builtin_functions: builtins.builtin_functions(),
            unit_map: builtins.builtin_units(),
            prefixes: builtins.builtin_prefixes(),
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
        self.builtin_values
            .get(identifier.as_str())
            .expect("builtin value should be defined")
            .clone()
    }

    pub fn lookup_parameter(
        &self,
        parameter_name: &ir::ParameterName,
    ) -> Result<Value, Vec<EvalError>> {
        let current_model = self
            .current_model
            .as_ref()
            .expect("current model should be set when looking up a parameter");

        self.lookup_model_parameter_internal(current_model, parameter_name)
    }

    pub fn lookup_model_parameter(
        &self,
        model: &ir::ModelPath,
        parameter_name: &ir::ParameterName,
    ) -> Result<Value, Vec<EvalError>> {
        self.lookup_model_parameter_internal(model.as_ref(), parameter_name)
    }

    fn lookup_model_parameter_internal(
        &self,
        model_path: &Path,
        parameter_name: &ir::ParameterName,
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
            .map_err(|_errors| vec![EvalError::ParameterHasError])
    }

    // TODO: figure out what error this should actually be
    pub fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        args: Vec<Value>,
    ) -> Result<Value, Vec<EvalError>> {
        self.builtin_functions
            .get(identifier.as_str())
            .expect("builtin function should be defined")
            .call(args)
    }

    pub fn evaluate_imported_function(
        &self,
        identifier: &ir::Identifier,
        args: Vec<Value>,
    ) -> Result<Value, Vec<EvalError>> {
        let _ = (self, identifier, args);
        Err(vec![EvalError::Unsupported])
    }

    pub fn lookup_unit(&self, name: &str) -> Option<SizedUnit> {
        self.unit_map.get(name).cloned()
    }

    pub const fn available_prefixes(&self) -> &HashMap<String, f64> {
        &self.prefixes
    }

    pub fn load_python_import(&mut self, python_path: PathBuf) {
        self.python_imports
            .insert(python_path, Err(EvalError::Unsupported));
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
        result: Result<Value, Vec<EvalError>>,
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

    pub fn add_test_result(&mut self, test_result: Result<Value, Vec<EvalError>>) {
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
}
