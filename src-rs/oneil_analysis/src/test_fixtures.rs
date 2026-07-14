//! Shared IR, graph, and evaluated-value fixtures for analysis tests.

use indexmap::{IndexMap, IndexSet};
use oneil_frontend::{BuiltinLookup, InstanceGraph, InstancedModel, ReferenceImport};
use oneil_ir as ir;
use oneil_output::{
    self as output, BuiltinDependency, DependencySet, ExternalDependency, Model, Parameter,
    ParameterDependency, PrintLevel, Test, Value,
};
use oneil_shared::{
    EvalInstanceKey, InstancePath,
    labels::ParameterLabel,
    paths::ModelPath,
    span::Span,
    symbols::{BuiltinValueName, ParameterName, ReferenceName, TestIndex},
};

use crate::test_context::test_model_path;

/// Synthetic span for constructing IR in tests.
#[must_use]
pub fn span() -> Span {
    Span::synthetic()
}

/// Alias for [`test_model_path`].
#[must_use]
pub fn model_path(name: &str) -> ModelPath {
    test_model_path(name)
}

/// Numeric literal expression.
#[must_use]
pub fn lit_number(value: f64) -> ir::Expr {
    ir::Expr::literal(span(), ir::Literal::number(value))
}

/// Boolean literal expression.
#[must_use]
pub fn lit_bool(value: bool) -> ir::Expr {
    ir::Expr::literal(span(), ir::Literal::boolean(value))
}

/// Parameter variable expression.
#[must_use]
pub fn param_var(name: &str) -> ir::Expr {
    ir::Expr::parameter_variable(span(), span(), ParameterName::from(name))
}

/// External variable expression (`parameter.reference` in source).
#[must_use]
pub fn external_var(parameter: &str, reference: &str) -> ir::Expr {
    ir::Expr::external_variable(
        span(),
        ReferenceName::from(reference),
        span(),
        ParameterName::from(parameter),
        span(),
    )
}

/// IR dependencies naming same-model parameters.
#[must_use]
pub fn deps_on_parameters(names: &[&str]) -> ir::Dependencies {
    let mut deps = ir::Dependencies::new();
    for name in names {
        deps.insert_parameter(ParameterName::from(*name), span());
    }
    deps
}

/// IR dependencies naming a single builtin.
#[must_use]
pub fn deps_on_builtin(name: &str) -> ir::Dependencies {
    let mut deps = ir::Dependencies::new();
    deps.insert_builtin(BuiltinValueName::from(name), span());
    deps
}

/// IR dependencies naming one external `(reference, parameter)`.
#[must_use]
pub fn deps_on_external(reference: &str, parameter: &str) -> ir::Dependencies {
    let mut deps = ir::Dependencies::new();
    deps.insert_external(
        ReferenceName::from(reference),
        ParameterName::from(parameter),
        span(),
    );
    deps
}

/// Evaluated [`DependencySet`] with only same-model parameter dependencies.
#[must_use]
pub fn parameter_deps(names: &[&str]) -> DependencySet {
    DependencySet {
        builtin_dependencies: IndexSet::new(),
        parameter_dependencies: names
            .iter()
            .map(|name| ParameterDependency {
                parameter_name: ParameterName::from(*name),
            })
            .collect(),
        external_dependencies: IndexSet::new(),
    }
}

/// Evaluated [`DependencySet`] with only builtin dependencies.
#[must_use]
pub fn builtin_deps(names: &[&str]) -> DependencySet {
    DependencySet {
        builtin_dependencies: names
            .iter()
            .map(|name| BuiltinDependency {
                name: BuiltinValueName::from(*name),
            })
            .collect(),
        parameter_dependencies: IndexSet::new(),
        external_dependencies: IndexSet::new(),
    }
}

/// Evaluated [`DependencySet`] with a single external dependency.
#[must_use]
pub fn external_deps(
    instance_key: EvalInstanceKey,
    reference: &str,
    parameter: &str,
) -> DependencySet {
    DependencySet {
        builtin_dependencies: IndexSet::new(),
        parameter_dependencies: IndexSet::new(),
        external_dependencies: IndexSet::from([ExternalDependency {
            instance_key,
            reference_name: ReferenceName::from(reference),
            parameter_name: ParameterName::from(parameter),
        }]),
    }
}

/// IR parameter with a numeric-literal body and the given dependency set.
///
/// Used by dependency-/reference-tree tests, which read
/// [`ir::Parameter::dependencies`] rather than walking the RHS expression.
#[must_use]
pub fn ir_parameter(name: &str, dependencies: ir::Dependencies) -> ir::Parameter {
    ir_parameter_with_expr(name, dependencies, lit_number(0.0))
}

/// IR parameter with the given RHS expression and empty dependency set.
///
/// Used by validation tests, which walk variables in the expression body.
#[must_use]
pub fn ir_parameter_expr(name: &str, expr: ir::Expr) -> ir::Parameter {
    ir_parameter_with_expr(name, ir::Dependencies::new(), expr)
}

/// Independent leaf parameter (`= 0`) for validation graphs.
#[must_use]
pub fn ir_parameter_leaf(name: &str) -> ir::Parameter {
    ir_parameter_expr(name, lit_number(0.0))
}

/// Parameter whose RHS is a bare parameter reference.
#[must_use]
pub fn ir_parameter_depends_on(name: &str, dep: &str) -> ir::Parameter {
    ir_parameter_expr(name, param_var(dep))
}

/// IR parameter with explicit dependencies and RHS expression.
#[must_use]
pub fn ir_parameter_with_expr(
    name: &str,
    dependencies: ir::Dependencies,
    expr: ir::Expr,
) -> ir::Parameter {
    ir::Parameter::new(
        dependencies,
        ParameterName::from(name),
        span(),
        span(),
        ParameterLabel::from(name),
        None,
        None,
        ir::ParameterValue::simple(expr, None),
        ir::Limits::default(),
        false,
        ir::TraceLevel::None,
        None,
    )
}

/// IR test with the given dependencies and a boolean-true body.
#[must_use]
pub fn ir_test(dependencies: ir::Dependencies) -> ir::Test {
    ir_test_with_expr(dependencies, lit_bool(true))
}

/// IR test with an empty dependency set and the given body expression.
#[must_use]
pub fn ir_test_expr(expr: ir::Expr) -> ir::Test {
    ir_test_with_expr(ir::Dependencies::new(), expr)
}

/// IR test with explicit dependencies and body expression.
#[must_use]
pub fn ir_test_with_expr(dependencies: ir::Dependencies, expr: ir::Expr) -> ir::Test {
    ir::Test::new(span(), ir::TraceLevel::None, expr, dependencies, None, None)
}

/// Evaluated parameter with a scalar numeric value and no dependencies.
#[must_use]
pub fn evaluated_parameter(name: &str, value: f64) -> Parameter {
    evaluated_parameter_with_deps(name, value, DependencySet::default())
}

/// Evaluated parameter with a scalar numeric value and the given dependencies.
#[must_use]
pub fn evaluated_parameter_with_deps(
    name: &str,
    value: f64,
    dependencies: DependencySet,
) -> Parameter {
    Parameter {
        ident: ParameterName::from(name),
        label: ParameterLabel::from(name),
        value: Value::from(value),
        print_level: PrintLevel::None,
        debug_info: None,
        dependencies,
        expr_span: span(),
        warnings: Vec::new(),
    }
}

/// Evaluated model with the given parameters and references.
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

/// Passed evaluated test.
#[must_use]
pub fn evaluated_test_passed() -> Test {
    Test {
        expr_span: span(),
        result: output::TestResult::Passed,
        warnings: Vec::new(),
    }
}

/// Failed evaluated test.
#[must_use]
pub fn evaluated_test_failed() -> Test {
    Test {
        expr_span: span(),
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

/// Cross-file reference import pointing at `path`.
#[must_use]
pub fn reference_import(alias: &str, path: &ModelPath) -> ReferenceImport {
    ReferenceImport::new(ReferenceName::from(alias), span(), None, None, path.clone())
}

/// [`InstancedModel`] with the given parameters, tests, and references.
#[must_use]
pub fn instanced_model(
    path: &ModelPath,
    parameters: IndexMap<ParameterName, ir::Parameter>,
    tests: IndexMap<TestIndex, ir::Test>,
    references: IndexMap<ReferenceName, ReferenceImport>,
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

/// Instantiated model with only parameters (no tests or references).
#[must_use]
pub fn instanced_model_params(
    path: &ModelPath,
    parameters: IndexMap<ParameterName, ir::Parameter>,
) -> InstancedModel {
    instanced_model(path, parameters, IndexMap::new(), IndexMap::new())
}

/// Instantiated model with parameters and cross-file references.
#[must_use]
pub fn instanced_model_with_refs(
    path: &ModelPath,
    parameters: IndexMap<ParameterName, ir::Parameter>,
    references: IndexMap<ReferenceName, ReferenceImport>,
) -> InstancedModel {
    instanced_model(path, parameters, IndexMap::new(), references)
}

/// Graph rooted at `root` with an empty reference pool.
#[must_use]
pub fn graph_from_root(root: InstancedModel) -> InstanceGraph {
    let mut graph = InstanceGraph::empty(root.path().clone());
    *graph.root = root;
    graph
}

/// Graph with `root` and a single pool entry.
#[must_use]
pub fn graph_with_pool(
    root: InstancedModel,
    pool_path: ModelPath,
    pool_model: InstancedModel,
) -> InstanceGraph {
    let mut graph = graph_from_root(root);
    graph.reference_pool.insert(pool_path, Box::new(pool_model));
    graph
}

/// Stub [`BuiltinLookup`] for validation classification tests.
pub struct StubBuiltins {
    names: Vec<&'static str>,
}

impl StubBuiltins {
    /// Creates a stub that recognizes the given builtin value names.
    #[must_use]
    pub fn new(names: &[&'static str]) -> Self {
        Self {
            names: names.to_vec(),
        }
    }
}

impl BuiltinLookup for StubBuiltins {
    fn has_builtin_value(&self, name: &str) -> bool {
        self.names.contains(&name)
    }
}
