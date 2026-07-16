//! Shared IR and context fixtures for evaluator tests.

use oneil_ir as ir;
use oneil_ir::test_helpers::{parameter::build_simple_parameter, unit::UnitSpec};
use oneil_output::{self as output, EvalError, Value};
use oneil_shared::{EvalInstanceKey, labels::ParameterLabel, span::Span, symbols::ParameterName};

use crate::{
    context::EvalContext,
    eval_parameter::{self, EvalParameterResult},
    test_context::{TestExternalContext, test_model_path},
};

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
        expr_span: Span::synthetic(),
        warnings: Vec::new(),
    }
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
