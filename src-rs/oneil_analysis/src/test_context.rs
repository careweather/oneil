//! Test double for [`ExternalAnalysisContext`](crate::ExternalAnalysisContext).

use indexmap::IndexMap;
use oneil_frontend::InstancedModel;
use oneil_output::{Model, Parameter, Test, Value};
use oneil_shared::{
    EvalInstanceKey,
    load_result::LoadResult,
    paths::ModelPath,
    symbols::{BuiltinValueName, ParameterName, TestIndex},
};

use crate::{
    context::ExternalAnalysisContext,
    output::error::{GetTestValueError, GetValueError, ModelEvalHasErrors},
};

/// Returns a [`ModelPath`] for use in tests (path without extension).
#[must_use]
pub fn test_model_path(s: &str) -> ModelPath {
    ModelPath::from_str_no_ext(s)
}

/// Test double for [`ExternalAnalysisContext`].
///
/// Populate with [`Self::insert_model_ir`], [`Self::insert_evaluated_parameter`],
/// [`Self::insert_evaluated_test`], [`Self::insert_evaluated_model`], and
/// [`Self::insert_builtin`] before calling analysis functions.
#[derive(Debug, Default)]
pub struct TestAnalysisContext {
    model_ir: IndexMap<EvalInstanceKey, InstancedModel>,
    /// Evaluated parameter lookup table. Missing key → model absent (`None`).
    /// `Err` variants mirror runtime lookup failures.
    parameters: IndexMap<(EvalInstanceKey, ParameterName), Result<Parameter, GetValueError>>,
    tests: IndexMap<(EvalInstanceKey, TestIndex), Result<Test, GetTestValueError>>,
    builtins: IndexMap<BuiltinValueName, Value>,
    evaluated_models: IndexMap<ModelPath, LoadResult<Model, ModelEvalHasErrors>>,
}

impl TestAnalysisContext {
    /// Creates an empty test analysis context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers IR for an evaluation instance (used to build the dependency graph).
    pub fn insert_model_ir(&mut self, key: EvalInstanceKey, model: InstancedModel) {
        self.model_ir.insert(key, model);
    }

    /// Registers a successfully evaluated parameter.
    pub fn insert_evaluated_parameter(&mut self, key: &EvalInstanceKey, parameter: Parameter) {
        let name = parameter.ident.clone();
        self.parameters.insert((key.clone(), name), Ok(parameter));
    }

    /// Registers a parameter lookup error (model- or parameter-level).
    pub fn insert_parameter_error(
        &mut self,
        key: &EvalInstanceKey,
        parameter_name: ParameterName,
        error: GetValueError,
    ) {
        self.parameters
            .insert((key.clone(), parameter_name), Err(error));
    }

    /// Registers a successfully evaluated test.
    pub fn insert_evaluated_test(
        &mut self,
        key: &EvalInstanceKey,
        test_index: TestIndex,
        test: Test,
    ) {
        self.tests.insert((key.clone(), test_index), Ok(test));
    }

    /// Registers a test lookup error (model- or test-level).
    pub fn insert_test_error(
        &mut self,
        key: &EvalInstanceKey,
        test_index: TestIndex,
        error: GetTestValueError,
    ) {
        self.tests.insert((key.clone(), test_index), Err(error));
    }

    /// Registers a builtin value.
    pub fn insert_builtin(&mut self, name: BuiltinValueName, value: Value) {
        self.builtins.insert(name, value);
    }

    /// Registers an evaluated model for [`ExternalAnalysisContext::get_evaluated_model`].
    pub fn insert_evaluated_model(
        &mut self,
        path: ModelPath,
        model: LoadResult<Model, ModelEvalHasErrors>,
    ) {
        self.evaluated_models.insert(path, model);
    }
}

impl ExternalAnalysisContext for TestAnalysisContext {
    fn get_all_model_ir(&self) -> IndexMap<EvalInstanceKey, &InstancedModel> {
        self.model_ir
            .iter()
            .map(|(key, model)| (key.clone(), model))
            .collect()
    }

    fn lookup_builtin_variable(&self, identifier: &BuiltinValueName) -> Option<&Value> {
        self.builtins.get(identifier)
    }

    fn get_evaluated_model(
        &self,
        model_path: &ModelPath,
    ) -> Option<LoadResult<&Model, ModelEvalHasErrors>> {
        self.evaluated_models
            .get(model_path)
            .map(|result| result.as_ref().map_err(|_| ModelEvalHasErrors))
    }

    fn lookup_parameter_value(
        &self,
        instance_key: &EvalInstanceKey,
        parameter_name: &ParameterName,
    ) -> Option<Result<&Parameter, GetValueError>> {
        self.parameters
            .get(&(instance_key.clone(), parameter_name.clone()))
            .map(|result| match result {
                Ok(parameter) => Ok(parameter),
                Err(GetValueError::Model) => Err(GetValueError::Model),
                Err(GetValueError::Parameter) => Err(GetValueError::Parameter),
            })
    }

    fn lookup_test_value(
        &self,
        instance_key: &EvalInstanceKey,
        test_index: TestIndex,
    ) -> Option<Result<&Test, GetTestValueError>> {
        self.tests
            .get(&(instance_key.clone(), test_index))
            .map(|result| match result {
                Ok(test) => Ok(test),
                Err(GetTestValueError::Model) => Err(GetTestValueError::Model),
                Err(GetTestValueError::Test) => Err(GetTestValueError::Test),
            })
    }
}
