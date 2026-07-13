//! Test support for analysis tree tests.
//!
//! Provides [`TestAnalysisContext`] that implements [`ExternalAnalysisContext`]
//! with manually registered IR, evaluated parameters/tests, and builtins.

use indexmap::{IndexMap, IndexSet};
use oneil_frontend::InstancedModel;
use oneil_ir as ir;
use oneil_output::{self as output, Model, Parameter, PrintLevel, Test, Value};
use oneil_shared::{
    EvalInstanceKey, InstancePath,
    labels::ParameterLabel,
    load_result::LoadResult,
    paths::ModelPath,
    span::Span,
    symbols::{BuiltinValueName, ParameterName, ReferenceName, TestIndex},
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
    ) -> Option<Result<Parameter, GetValueError>> {
        self.parameters
            .get(&(instance_key.clone(), parameter_name.clone()))
            .map(|result| match result {
                Ok(parameter) => Ok(parameter.clone()),
                Err(GetValueError::Model) => Err(GetValueError::Model),
                Err(GetValueError::Parameter) => Err(GetValueError::Parameter),
            })
    }

    fn lookup_test_value(
        &self,
        instance_key: &EvalInstanceKey,
        test_index: TestIndex,
    ) -> Option<Result<Test, GetTestValueError>> {
        self.tests
            .get(&(instance_key.clone(), test_index))
            .map(|result| match result {
                Ok(test) => Ok(test.clone()),
                Err(GetTestValueError::Model) => Err(GetTestValueError::Model),
                Err(GetTestValueError::Test) => Err(GetTestValueError::Test),
            })
    }
}

/// Builds a simple IR parameter with the given dependencies and a numeric literal body.
#[must_use]
pub fn ir_parameter(name: &str, dependencies: ir::Dependencies) -> ir::Parameter {
    let span = Span::synthetic();
    ir::Parameter::new(
        dependencies,
        ParameterName::from(name),
        span.clone(),
        span,
        ParameterLabel::from(name),
        None,
        None,
        ir::ParameterValue::simple(
            ir::Expr::literal(Span::synthetic(), ir::Literal::number(0.0)),
            None,
        ),
        ir::Limits::default(),
        false,
        ir::TraceLevel::None,
        None,
    )
}

/// Builds an IR test with the given dependencies and a boolean literal body.
#[must_use]
pub fn ir_test(dependencies: ir::Dependencies) -> ir::Test {
    ir::Test::new(
        Span::synthetic(),
        ir::TraceLevel::None,
        ir::Expr::literal(Span::synthetic(), ir::Literal::boolean(true)),
        dependencies,
        None,
        None,
    )
}

/// Builds an evaluated parameter with a scalar numeric value and no dependencies.
#[must_use]
pub fn evaluated_parameter(name: &str, value: f64) -> Parameter {
    evaluated_parameter_with_deps(name, value, output::DependencySet::default())
}

/// Builds an evaluated parameter with a scalar numeric value and the given dependencies.
#[must_use]
pub fn evaluated_parameter_with_deps(
    name: &str,
    value: f64,
    dependencies: output::DependencySet,
) -> Parameter {
    Parameter {
        ident: ParameterName::from(name),
        label: ParameterLabel::from(name),
        value: Value::from(value),
        print_level: PrintLevel::None,
        debug_info: None,
        dependencies,
        expr_span: Span::synthetic(),
        warnings: Vec::new(),
    }
}

/// Builds an evaluated model with the given parameters and references.
#[must_use]
pub fn evaluated_model(
    path: &ModelPath,
    parameters: IndexMap<ParameterName, Parameter>,
    references: IndexMap<ReferenceName, EvalInstanceKey>,
) -> Model {
    Model {
        path: path.clone(),
        instance_path: InstancePath::root(),
        submodels: IndexSet::new(),
        references,
        parameters,
        tests: IndexMap::new(),
    }
}

/// Builds a passed evaluated test.
#[must_use]
pub fn evaluated_test_passed() -> Test {
    Test {
        expr_span: Span::synthetic(),
        result: output::TestResult::Passed,
        warnings: Vec::new(),
    }
}

/// Builds a failed evaluated test.
#[must_use]
pub fn evaluated_test_failed() -> Test {
    Test {
        expr_span: Span::synthetic(),
        result: output::TestResult::Failed {
            debug_info: Box::new(output::DebugInfo {
                builtin_dependency_values: IndexMap::new(),
                parameter_dependency_values: IndexMap::new(),
                external_dependency_values: IndexMap::new(),
            }),
        },
        warnings: Vec::new(),
    }
}

/// Builds an [`InstancedModel`] with the given parameters, tests, and references.
#[must_use]
pub fn instanced_model(
    path: &ModelPath,
    parameters: IndexMap<ParameterName, ir::Parameter>,
    tests: IndexMap<TestIndex, ir::Test>,
    references: IndexMap<oneil_shared::symbols::ReferenceName, oneil_frontend::ReferenceImport>,
) -> InstancedModel {
    InstancedModel::new(
        path.clone(),
        IndexMap::new(),
        IndexMap::new(),
        references,
        IndexMap::new(),
        parameters,
        tests,
        None,
    )
}
