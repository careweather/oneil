//! Shared IR and graph fixtures for validation tests.

use indexmap::IndexMap;
use oneil_frontend::{BuiltinLookup, InstanceGraph, InstancedModel, ReferenceImport};
use oneil_ir as ir;
pub use oneil_ir::test_helpers::expr::{external_var, lit_number, param_var};
use oneil_ir::test_helpers::{parameter::build_parameter_with_dependencies, test::make_test};
use oneil_shared::{
    paths::ModelPath,
    span::Span,
    symbols::{ParameterName, ReferenceName, TestIndex},
};

/// Synthetic span for constructing IR in tests.
#[must_use]
pub fn span() -> Span {
    Span::synthetic()
}

/// Returns a synthetic model path for validation tests.
#[must_use]
pub fn model_path(name: &str) -> ModelPath {
    ModelPath::from_str_no_ext(name)
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
    build_parameter_with_dependencies(name, dependencies, expr)
}

/// IR test with an empty dependency set and the given body expression.
#[must_use]
pub fn ir_test_expr(expr: ir::Expr) -> ir::Test {
    ir_test_with_expr(ir::Dependencies::new(), expr)
}

/// IR test with explicit dependencies and body expression.
#[must_use]
pub fn ir_test_with_expr(dependencies: ir::Dependencies, expr: ir::Expr) -> ir::Test {
    make_test(expr, dependencies)
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
