//! Shared helpers for LSP integration tests.

use std::path::PathBuf;

use oneil_frontend::ModelDesignInfo;
use oneil_runtime::{
    CacheReadPolicy, CacheWritePolicy, Runtime,
    output::{ir, reference::ModelTemplateReference},
};
use oneil_shared::paths::{DesignPath, ModelPath, PythonPath};

/// Root directory of LSP test fixtures (`fixtures/` in this crate).
#[must_use]
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Resolves a fixture path relative to [`fixtures_dir`].
#[must_use]
pub fn fixture_path(relative: &str) -> ModelPath {
    ModelPath::from_path_with_ext(&fixtures_dir().join(relative))
}

/// Resolves a Python fixture path relative to [`fixtures_dir`].
#[must_use]
pub fn python_fixture_path(relative: &str) -> PythonPath {
    PythonPath::from_path_with_ext(&fixtures_dir().join(relative))
}

/// Resolves a design fixture path relative to [`fixtures_dir`].
#[must_use]
pub fn design_fixture_path(relative: &str) -> DesignPath {
    DesignPath::from_path_with_ext(&fixtures_dir().join(relative))
}

/// Returns a runtime configured for deterministic, side-effect-free tests.
#[must_use]
pub fn test_runtime() -> Runtime {
    Runtime::new(CacheReadPolicy::Never, CacheWritePolicy::Never)
}

/// Loads a fixture and returns the lowered template reference with design metadata.
///
/// # Panics
///
/// Panics if the model fails to load or has resolution errors.
#[track_caller]
pub fn load_model_and_design<'a>(
    runtime: &'a mut Runtime,
    relative: &str,
) -> (ModelTemplateReference<'a>, Option<ModelDesignInfo>) {
    let path = fixture_path(relative);
    let (model, design_info, errors) = runtime.load_and_lower(&path);
    assert!(
        errors.is_empty(),
        "expected no errors loading {relative}, got: {errors:?}"
    );
    (
        model.unwrap_or_else(|| panic!("expected model for fixture {relative}")),
        design_info,
    )
}

/// Loads a fixture model and returns the lowered template reference.
///
/// # Panics
///
/// Panics if the model fails to load or has resolution errors.
#[track_caller]
pub fn load_model<'a>(runtime: &'a mut Runtime, relative: &str) -> ModelTemplateReference<'a> {
    load_model_and_design(runtime, relative).0
}

/// Loads a fixture and returns its design metadata.
///
/// # Panics
///
/// Panics if the fixture fails to load, has resolution errors, or has no design metadata.
#[track_caller]
pub fn load_design(runtime: &mut Runtime, relative: &str) -> ModelDesignInfo {
    load_model_and_design(runtime, relative)
        .1
        .unwrap_or_else(|| panic!("expected design info for fixture {relative}"))
}

/// Returns a byte offset for the first variable in a parameter value matching `predicate`.
///
/// # Panics
///
/// Panics if the parameter value is not a simple expression or no matching variable is found.
#[track_caller]
pub fn variable_offset_in_parameter_expr(
    value: &ir::ParameterValue,
    predicate: impl Fn(&ir::Variable) -> bool,
) -> usize {
    let ir::ParameterValue::Simple(expr, _) = value else {
        panic!("expected simple parameter value");
    };

    let mut found = None;
    expr.walk_variables(&mut |variable| {
        if predicate(variable) {
            let span = match variable {
                oneil_runtime::output::ir::Variable::Parameter {
                    parameter_span: span,
                    ..
                }
                | oneil_runtime::output::ir::Variable::External {
                    parameter_span: span,
                    ..
                }
                | oneil_runtime::output::ir::Variable::Builtin {
                    ident_span: span, ..
                } => span,
            };
            found = Some(span.start().offset);
        }
    });
    found.expect("matching variable not found in parameter expression")
}

/// Returns a byte offset on the reference name of the first matching external variable.
///
/// # Panics
///
/// Panics if the parameter value is not a simple expression or no matching variable is found.
#[track_caller]
pub fn import_reference_offset_in_parameter_expr(
    value: &ir::ParameterValue,
    predicate: impl Fn(&ir::Variable) -> bool,
) -> usize {
    let ir::ParameterValue::Simple(expr, _) = value else {
        panic!("expected simple parameter value");
    };

    let mut found = None;
    expr.walk_variables(&mut |variable| {
        if predicate(variable)
            && let ir::Variable::External {
                reference_span: span,
                ..
            } = variable
        {
            found = Some(span.start().offset);
        }
    });
    found.expect("matching external variable not found in parameter expression")
}

/// Returns a byte offset on the function name of the first matching call.
///
/// # Panics
///
/// Panics if the parameter value is not a simple expression or no matching call is found.
#[track_caller]
pub fn function_call_offset_in_parameter_expr(
    value: &ir::ParameterValue,
    predicate: impl Fn(&ir::FunctionName) -> bool,
) -> usize {
    let ir::ParameterValue::Simple(expr, _) = value else {
        panic!("expected simple parameter value");
    };

    function_call_offset_in_expr(expr, &predicate)
        .expect("matching function call not found in parameter expression")
}

fn function_call_offset_in_expr(
    expr: &ir::Expr,
    predicate: &impl Fn(&ir::FunctionName) -> bool,
) -> Option<usize> {
    match expr {
        ir::Expr::FunctionCall {
            name,
            name_span,
            args,
            ..
        } => predicate(name)
            .then(|| name_span.start().offset)
            .or_else(|| {
                args.iter()
                    .find_map(|arg| function_call_offset_in_expr(arg, predicate))
            }),
        ir::Expr::ComparisonOp {
            left,
            right,
            rest_chained,
            ..
        } => function_call_offset_in_expr(left, predicate)
            .or_else(|| function_call_offset_in_expr(right, predicate))
            .or_else(|| {
                rest_chained
                    .iter()
                    .find_map(|(_, expr)| function_call_offset_in_expr(expr, predicate))
            }),
        ir::Expr::BinaryOp { left, right, .. } | ir::Expr::Fallback { left, right, .. } => {
            function_call_offset_in_expr(left, predicate)
                .or_else(|| function_call_offset_in_expr(right, predicate))
        }
        ir::Expr::UnaryOp { expr, .. } | ir::Expr::UnitCast { expr, .. } => {
            function_call_offset_in_expr(expr, predicate)
        }
        ir::Expr::Variable { .. } | ir::Expr::Literal { .. } => None,
    }
}
