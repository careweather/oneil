//! Shared IR and context fixtures for evaluator tests.

use oneil_ir as ir;
use oneil_output::{self as output, DimensionMap, EvalError, Value};
use oneil_shared::{
    EvalInstanceKey,
    labels::ParameterLabel,
    paths::PythonPath,
    span::Span,
    symbols::{
        BuiltinFunctionName, ParameterName, PyFunctionName, ReferenceName, UnitBaseName, UnitName,
        UnitPrefix,
    },
};

use crate::{
    context::EvalContext,
    eval_parameter::{self, EvalParameterResult},
    test_context::{TestExternalContext, test_model_path},
};

/// Synthetic span for constructing IR in tests.
#[must_use]
pub fn span() -> Span {
    Span::synthetic()
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

/// String literal expression.
#[must_use]
pub fn lit_string(value: &str) -> ir::Expr {
    ir::Expr::literal(span(), ir::Literal::string(value.to_string()))
}

/// Binary operation over two expressions.
#[must_use]
pub fn binary(op: ir::BinaryOp, left: ir::Expr, right: ir::Expr) -> ir::Expr {
    ir::Expr::binary_op(span(), op, left, right)
}

/// Unary operation over an expression.
#[must_use]
pub fn unary(op: ir::UnaryOp, expr: ir::Expr) -> ir::Expr {
    ir::Expr::unary_op(span(), op, expr)
}

/// Comparison between two expressions.
#[must_use]
pub fn compare(op: ir::ComparisonOp, left: ir::Expr, right: ir::Expr) -> ir::Expr {
    ir::Expr::comparison_op(span(), op, left, right, vec![])
}

/// Chained comparison expression.
#[must_use]
pub fn compare_chained(
    left: ir::Expr,
    op: ir::ComparisonOp,
    right: ir::Expr,
    rest: Vec<(ir::ComparisonOp, ir::Expr)>,
) -> ir::Expr {
    ir::Expr::comparison_op(span(), op, left, right, rest)
}

/// Parameter variable expression.
#[must_use]
pub fn param_var(name: &str) -> ir::Expr {
    ir::Expr::parameter_variable(span(), span(), ParameterName::from(name))
}

/// External parameter variable (`name.reference`).
#[must_use]
pub fn external_var(parameter_name: &str, reference_name: &str) -> ir::Expr {
    ir::Expr::external_variable(
        span(),
        ReferenceName::from(reference_name),
        span(),
        ParameterName::from(parameter_name),
        span(),
    )
}

/// Builtin function call expression.
#[must_use]
pub fn builtin_call(name: &str, args: Vec<ir::Expr>) -> ir::Expr {
    ir::Expr::function_call(
        span(),
        span(),
        ir::FunctionName::builtin(BuiltinFunctionName::from(name), span()),
        args,
    )
}

/// Imported function call expression.
#[must_use]
pub fn imported_call(python_path: &str, name: &str, args: Vec<ir::Expr>) -> ir::Expr {
    ir::Expr::function_call(
        span(),
        span(),
        ir::FunctionName::imported(
            PythonPath::from_str_no_ext(python_path),
            PyFunctionName::from(name),
            span(),
        ),
        args,
    )
}

/// Scalar number type for type-error assertions.
#[must_use]
pub const fn scalar_number_type() -> output::ValueType {
    output::ValueType::Number {
        number_type: output::NumberType::Scalar,
    }
}

/// Stub evaluated parameter for seeding an [`EvalContext`].
#[must_use]
pub fn output_parameter(name: &str, value: Value) -> output::Parameter {
    output::Parameter {
        value,
        ident: ParameterName::from(name),
        label: ParameterLabel::from(name),
        print_level: output::PrintLevel::None,
        debug_info: None,
        dependencies: output::DependencySet::default(),
        expr_span: span(),
        warnings: Vec::new(),
    }
}

/// Specification for a unit in tests.
#[derive(Debug, Clone, Copy)]
pub struct UnitSpec {
    pub prefix: Option<&'static str>,
    pub base_name: Option<&'static str>,
    pub is_db: bool,
    pub exponent: f64,
}

impl UnitSpec {
    /// Creates a unit specification.
    #[must_use]
    pub const fn new(
        prefix: Option<&'static str>,
        base_name: Option<&'static str>,
        is_db: bool,
        exponent: f64,
    ) -> Self {
        Self {
            prefix,
            base_name,
            is_db,
            exponent,
        }
    }
}

fn build_full_name(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> UnitName {
    UnitName::new(format!(
        "{}{}{}",
        if is_db { "dB" } else { "" },
        prefix.unwrap_or(""),
        base_name.unwrap_or("")
    ))
}

fn build_unit_info(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> ir::UnitInfo {
    if is_db {
        ir::UnitInfo::Db {
            prefix: prefix.map(UnitPrefix::from),
            base_name: base_name.map(UnitBaseName::from),
        }
    } else {
        ir::UnitInfo::Standard {
            prefix: prefix.map(UnitPrefix::from),
            base_name: UnitBaseName::from(base_name.expect("base name should be provided")),
        }
    }
}

fn ir_display_composite_unit(
    unit_list: impl IntoIterator<Item = UnitSpec>,
) -> ir::DisplayCompositeUnit {
    let mut units = unit_list.into_iter().map(|spec| {
        ir::DisplayCompositeUnit::BaseUnit(ir::DisplayUnit::new(
            build_full_name(spec.base_name, spec.prefix, spec.is_db).into_string(),
            spec.exponent,
        ))
    });

    let Some(first) = units.next() else {
        return ir::DisplayCompositeUnit::One;
    };

    units.fold(first, |acc, unit| {
        ir::DisplayCompositeUnit::Multiply(Box::new(acc), Box::new(unit))
    })
}

/// Builds an IR composite unit from unit specs (with a real display tree).
#[must_use]
pub fn ir_composite_unit(unit_list: impl IntoIterator<Item = UnitSpec>) -> ir::CompositeUnit {
    let unit_specs: Vec<_> = unit_list.into_iter().collect();
    let display_unit = ir_display_composite_unit(unit_specs.iter().copied());
    let unit_vec = unit_specs
        .into_iter()
        .map(|spec| {
            let full_name = build_full_name(spec.base_name, spec.prefix, spec.is_db);
            let info = build_unit_info(spec.base_name, spec.prefix, spec.is_db);
            ir::Unit::new(span(), full_name, span(), spec.exponent, None, info)
        })
        .collect::<Vec<_>>();
    ir::CompositeUnit::new(
        unit_vec,
        display_unit,
        span(),
        DimensionMap::dimensionless(),
    )
}

/// Builds optional resolved units for a parameter (None when empty).
#[must_use]
pub fn build_resolved_units(
    units: impl IntoIterator<Item = UnitSpec>,
) -> Option<ir::CompositeUnit> {
    let units: Vec<_> = units.into_iter().collect();
    if units.is_empty() {
        None
    } else {
        Some(ir_composite_unit(units))
    }
}

/// Builds a parameter from an explicit value expression, optional units, and limits.
#[must_use]
pub fn build_parameter_from_expr(
    name: &str,
    expr: ir::Expr,
    units: Option<ir::CompositeUnit>,
    limits: ir::Limits,
) -> ir::Parameter {
    ir::Parameter::new(
        ir::Dependencies::new(),
        ParameterName::from(name),
        span(),
        span(),
        ParameterLabel::from(name),
        None,
        None,
        ir::ParameterValue::simple(expr, units),
        limits,
        false,
        ir::TraceLevel::None,
        None,
    )
}

/// Builds a parameter with a literal value, optional units, and limits.
#[must_use]
pub fn build_literal_parameter(
    name: &str,
    value: ir::Literal,
    units: impl IntoIterator<Item = UnitSpec>,
    limits: ir::Limits,
) -> ir::Parameter {
    build_parameter_from_expr(
        name,
        ir::Expr::literal(span(), value),
        build_resolved_units(units),
        limits,
    )
}

/// Builds a piecewise parameter from `(value, condition)` branches.
#[must_use]
pub fn build_piecewise_parameter(
    name: &str,
    branches: impl IntoIterator<Item = (ir::Expr, ir::Expr)>,
    units: impl IntoIterator<Item = UnitSpec>,
) -> ir::Parameter {
    let piecewise = branches
        .into_iter()
        .map(|(value, condition)| ir::PiecewiseExpr::new(value, condition))
        .collect();

    ir::Parameter::new(
        ir::Dependencies::new(),
        ParameterName::from(name),
        span(),
        span(),
        ParameterLabel::from(name),
        None,
        None,
        ir::ParameterValue::piecewise(piecewise, build_resolved_units(units)),
        ir::Limits::default(),
        false,
        ir::TraceLevel::None,
        None,
    )
}

/// Builds a simple parameter with a literal numeric value.
#[must_use]
pub fn build_simple_parameter(
    name: &str,
    value: f64,
    units: impl IntoIterator<Item = UnitSpec>,
) -> ir::Parameter {
    build_literal_parameter(
        name,
        ir::Literal::Number(value),
        units,
        ir::Limits::default(),
    )
}

/// Builds a parameter whose value is a binary op over two parameter variables.
#[must_use]
pub fn build_binary_parameter(
    name: &str,
    op: ir::BinaryOp,
    left: &str,
    right: &str,
    units: impl IntoIterator<Item = UnitSpec>,
) -> ir::Parameter {
    build_parameter_from_expr(
        name,
        binary(op, param_var(left), param_var(right)),
        build_resolved_units(units),
        ir::Limits::default(),
    )
}

/// Builds a parameter whose value is `base ^ exponent`.
#[must_use]
pub fn build_exponent_parameter(
    name: &str,
    base: &str,
    exponent: f64,
    units: impl IntoIterator<Item = UnitSpec>,
) -> ir::Parameter {
    build_parameter_from_expr(
        name,
        binary(ir::BinaryOp::Pow, param_var(base), lit_number(exponent)),
        build_resolved_units(units),
        ir::Limits::default(),
    )
}

/// Builds an IR test from an expression and dependency list.
#[must_use]
pub fn make_test(expr: ir::Expr, dependencies: ir::Dependencies) -> ir::Test {
    ir::Test::new(span(), ir::TraceLevel::None, expr, dependencies, None, None)
}

/// Creates an eval context with a root `"test"` model and runs `f`.
pub fn with_root_context<R>(f: impl FnOnce(&mut EvalContext<'_, TestExternalContext>) -> R) -> R {
    let mut external = TestExternalContext::new();
    let mut context = EvalContext::new(&mut external);
    context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
    f(&mut context)
}

/// Evaluates a parameter in a fresh root `"test"` context.
///
/// # Errors
///
/// Propagates evaluation errors from [`eval_parameter::eval_parameter`].
pub fn eval_parameter_simple(
    parameter: &ir::Parameter,
) -> Result<EvalParameterResult, Vec<EvalError>> {
    with_root_context(|context| eval_parameter::eval_parameter(parameter, context))
}

/// Evaluates simple numeric parameters and seeds their results into `context`.
///
/// The context must already have an active model.
pub fn setup_context_with_parameters(
    context: &mut EvalContext<'_, TestExternalContext>,
    previous_parameters: impl IntoIterator<Item = (&'static str, f64, Vec<UnitSpec>)>,
) {
    for (name, value, units) in previous_parameters {
        let parameter = build_simple_parameter(name, value, units);
        let parameter_value =
            eval_parameter::eval_parameter(&parameter, context).expect("eval should succeed");
        context.add_parameter_result(
            ParameterName::from(name),
            Ok(output_parameter(name, parameter_value.value)),
        );
    }
}
