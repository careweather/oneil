use std::iter;

use oneil_ir as ir;
use oneil_shared::{EvalInstanceKey, paths::ModelPath, span::Span};

use oneil_output::{
    EvalError, EvalWarning, ExpectedType, Number, Unit, UnitConversionError, Value,
    error::convert::{binary_eval_error_to_eval_error, unary_eval_error_to_eval_error},
};

use crate::{
    context::{EvalContext, ExternalEvaluationContext},
    eval_unit::eval_unit,
};

/// A per-operand result in a chained comparison: the operator, the right-hand value, and its span.
type RestComparisonResult = Result<(ir::ComparisonOp, (Value, Span)), Vec<EvalError>>;

/// Evaluates an expression in the context of the given model.
///
/// # Errors
///
/// Returns an error if the expression is invalid.
pub fn eval_expr_in_model<E: ExternalEvaluationContext>(
    expr: &ir::Expr,
    model_path: &ModelPath,
    context: &mut E,
) -> Result<Value, Vec<EvalError>> {
    let mut eval_context = EvalContext::with_preloaded_models(context);
    eval_context.set_evaluation_cache_root(model_path.clone());
    eval_context.push_active_model(EvalInstanceKey::root(model_path.clone()));

    eval_expr(expr, &mut eval_context).map(|(value, _span)| value)
}

/// Evaluates an expression and returns the resulting value.
///
/// # Errors
///
/// Returns an error if the expression is invalid.
pub fn eval_expr<'a, E: ExternalEvaluationContext>(
    expr: &'a ir::Expr,
    context: &mut EvalContext<'_, E>,
) -> Result<(Value, &'a Span), Vec<EvalError>> {
    match expr {
        ir::Expr::ComparisonOp {
            left,
            op,
            right,
            rest_chained,
            span,
        } => {
            let ComparisonSubexpressionsResult {
                left_result,
                left_result_span,
                rest_results,
            } = eval_comparison_subexpressions(left, *op, right, rest_chained, context)?;
            eval_comparison_chain(left_result, left_result_span, rest_results)
                .map(|result| (result, span))
        }
        ir::Expr::BinaryOp {
            op,
            left,
            right,
            span,
        } => {
            let BinaryOpSubexpressionsResult {
                left_result,
                left_result_span,
                right_result,
                right_result_span,
            } = eval_binary_op_subexpressions(left, right, context)?;
            eval_binary_op(
                left_result,
                left_result_span,
                *op,
                right_result,
                right_result_span,
            )
            .map(|result| (result, span))
        }
        ir::Expr::UnaryOp { op, expr, span } => {
            let (expr_result, expr_result_span) = eval_expr(expr, context)?;
            eval_unary_op(*op, expr_result, expr_result_span.clone()).map(|result| (result, span))
        }
        ir::Expr::Fallback { left, right, span } => {
            eval_fallback(left, right, context).map(|result| (result, span))
        }
        ir::Expr::FunctionCall {
            name,
            args,
            span: function_call_span,
            name_span: _,
        } => {
            let args_results = eval_function_call_args(args, context)?;
            eval_function_call(name, function_call_span.clone(), args_results, context)
                .map(|result| (result, function_call_span))
        }
        ir::Expr::UnitCast { span, expr, unit } => {
            let (expr_result, expr_result_span) = eval_expr(expr, context)?;
            let (unit_result, unit_result_span) = eval_unit(unit, context);
            eval_unit_cast(
                expr_result,
                expr_result_span.clone(),
                unit_result,
                unit_result_span,
            )
            .map(|result| (result, span))
        }
        ir::Expr::Variable { variable, span } => {
            eval_variable(variable, context).map(|result| (result, span))
        }
        ir::Expr::Literal { value, span } => {
            let literal_result = eval_literal(value);
            Ok((literal_result, span))
        }
    }
}

struct ComparisonSubexpressionsResult {
    left_result: Value,
    left_result_span: Span,
    rest_results: Vec<(ir::ComparisonOp, (Value, Span))>,
}

fn eval_comparison_subexpressions<E: ExternalEvaluationContext>(
    left: &ir::Expr,
    op: ir::ComparisonOp,
    right: &ir::Expr,
    rest_chained: &[(ir::ComparisonOp, ir::Expr)],
    context: &mut EvalContext<'_, E>,
) -> Result<ComparisonSubexpressionsResult, Vec<EvalError>> {
    let left_result = eval_expr(left, context);
    // With `&mut context`, iterators can't re-borrow — collect eagerly.
    let mut rest_results: Vec<RestComparisonResult> = Vec::with_capacity(rest_chained.len() + 1);
    for (op, right_operand) in iter::once((op, right)).chain(
        rest_chained
            .iter()
            .map(|(op, right_operand)| (*op, right_operand)),
    ) {
        rest_results.push(
            eval_expr(right_operand, context).map(|(result, span)| (op, (result, span.clone()))),
        );
    }

    let (left_result, left_result_span, rest_results) = match left_result {
        Err(left_errors) => {
            // find all evaluation errors that occurred and return them
            let errors = left_errors
                .into_iter()
                .chain(rest_results.into_iter().filter_map(Result::err).flatten())
                .collect();

            return Err(errors);
        }

        Ok((left_result, left_result_span)) => {
            let mut ok_rest_results = vec![];
            let mut err_rest_results = vec![];

            // check for evaluation errors
            for result in rest_results {
                match result {
                    Ok((op, (right_operand, right_operand_span))) => {
                        ok_rest_results.push((op, (right_operand, right_operand_span)));
                    }
                    Err(mut errors) => err_rest_results.append(&mut errors),
                }
            }

            // if any evaluation errors occurred, return them
            if !err_rest_results.is_empty() {
                return Err(err_rest_results);
            }

            // otherwise, everything was okay
            (left_result, left_result_span.clone(), ok_rest_results)
        }
    };
    Ok(ComparisonSubexpressionsResult {
        left_result,
        left_result_span,
        rest_results,
    })
}

fn eval_comparison_chain(
    left_result: Value,
    left_result_span: Span,
    rest_results: Vec<(ir::ComparisonOp, (Value, Span))>,
) -> Result<Value, Vec<EvalError>> {
    // structs only used internally in this function
    struct ComparisonSuccess {
        result: bool,
        next_lhs: (Value, Span),
    }

    struct ComparisonFailure {
        errors: Vec<EvalError>,
        last_successful_lhs: Box<(Value, Span)>,
    }

    let initial_result = Ok(ComparisonSuccess {
        result: true,
        next_lhs: (left_result, left_result_span),
    });

    let comparison_result = rest_results.into_iter().fold(
        initial_result,
        |comparison_result, (op, (rhs, rhs_span))| match comparison_result {
            Ok(ComparisonSuccess {
                next_lhs: (lhs, lhs_span),
                result,
            }) => {
                let comparison_result =
                    eval_comparison_op(&lhs, lhs_span.clone(), op, &rhs, rhs_span.clone());

                comparison_result
                    .map(|comparison_result| ComparisonSuccess {
                        result: result && comparison_result,
                        next_lhs: (rhs, rhs_span),
                    })
                    .map_err(|error| ComparisonFailure {
                        errors: vec![*error],
                        last_successful_lhs: Box::new((lhs, lhs_span)),
                    })
            }

            Err(ComparisonFailure {
                errors,
                last_successful_lhs,
            }) => {
                let (last_successful_lhs, last_successful_lhs_span) = *last_successful_lhs;
                let result = eval_comparison_op(
                    &last_successful_lhs,
                    last_successful_lhs_span.clone(),
                    op,
                    &rhs,
                    rhs_span,
                );

                let errors = if let Err(error) = result {
                    let mut comparison_errors = errors;
                    comparison_errors.push(*error);
                    comparison_errors
                } else {
                    errors
                };

                Err(ComparisonFailure {
                    errors,
                    last_successful_lhs: Box::new((last_successful_lhs, last_successful_lhs_span)),
                })
            }
        },
    );

    comparison_result
        .map(|comparison_success| Value::Boolean(comparison_success.result))
        .map_err(|comparison_failure| comparison_failure.errors)
}

fn eval_comparison_op(
    lhs: &Value,
    lhs_span: Span,
    op: ir::ComparisonOp,
    rhs: &Value,
    rhs_span: Span,
) -> Result<bool, Box<EvalError>> {
    let result = match op {
        ir::ComparisonOp::Eq => lhs.checked_eq(rhs),
        ir::ComparisonOp::NotEq => lhs.checked_ne(rhs),
        ir::ComparisonOp::LessThan => lhs.checked_lt(rhs),
        ir::ComparisonOp::LessThanEq => lhs.checked_lte(rhs),
        ir::ComparisonOp::GreaterThan => lhs.checked_gt(rhs),
        ir::ComparisonOp::GreaterThanEq => lhs.checked_gte(rhs),
    };

    result.map_err(|error| Box::new(binary_eval_error_to_eval_error(error, lhs_span, rhs_span)))
}

struct BinaryOpSubexpressionsResult {
    left_result: Value,
    left_result_span: Span,
    right_result: Value,
    right_result_span: Span,
}

fn eval_binary_op_subexpressions<E: ExternalEvaluationContext>(
    left: &ir::Expr,
    right: &ir::Expr,
    context: &mut EvalContext<'_, E>,
) -> Result<BinaryOpSubexpressionsResult, Vec<EvalError>> {
    // Sequentially evaluate each side; `&mut context` means we can't hold two
    // results tied to borrow-lifetimes at once, so copy out spans as we go.
    let left_result = eval_expr(left, context).map(|(v, s)| (v, s.clone()));
    let right_result = eval_expr(right, context).map(|(v, s)| (v, s.clone()));

    match (left_result, right_result) {
        (Ok((left_result, left_result_span)), Ok((right_result, right_result_span))) => {
            Ok(BinaryOpSubexpressionsResult {
                left_result,
                left_result_span,
                right_result,
                right_result_span,
            })
        }
        (Err(left_errors), Ok(_)) => Err(left_errors),
        (Ok(_), Err(right_errors)) => Err(right_errors),
        (Err(left_errors), Err(right_errors)) => {
            Err(left_errors.into_iter().chain(right_errors).collect())
        }
    }
}

fn eval_binary_op(
    left_result: Value,
    left_result_span: Span,
    op: ir::BinaryOp,
    right_result: Value,
    right_result_span: Span,
) -> Result<Value, Vec<EvalError>> {
    let result = match op {
        ir::BinaryOp::Add => left_result.checked_add(right_result),
        ir::BinaryOp::Sub => left_result.checked_sub(right_result),
        ir::BinaryOp::EscapedSub => left_result.checked_escaped_sub(right_result),
        ir::BinaryOp::Mul => left_result.checked_mul(right_result),
        ir::BinaryOp::Div => left_result.checked_div(right_result),
        ir::BinaryOp::EscapedDiv => left_result.checked_escaped_div(right_result),
        ir::BinaryOp::Mod => left_result.checked_rem(right_result),
        ir::BinaryOp::Pow => left_result.checked_pow(right_result),
        ir::BinaryOp::And => left_result.checked_and(right_result),
        ir::BinaryOp::Or => left_result.checked_or(right_result),
        ir::BinaryOp::MinMax => left_result.checked_min_max(right_result),
    };

    result.map_err(|error| {
        vec![binary_eval_error_to_eval_error(
            error,
            left_result_span,
            right_result_span,
        )]
    })
}

fn eval_unary_op(
    op: ir::UnaryOp,
    expr_result: Value,
    expr_result_span: Span,
) -> Result<Value, Vec<EvalError>> {
    let result = match op {
        ir::UnaryOp::Neg => expr_result.checked_neg(),
        ir::UnaryOp::Not => expr_result.checked_not(),
    };

    result.map_err(|error| vec![unary_eval_error_to_eval_error(error, expr_result_span)])
}

fn eval_fallback<E: ExternalEvaluationContext>(
    left: &ir::Expr,
    right: &ir::Expr,
    context: &mut EvalContext<'_, E>,
) -> Result<Value, Vec<EvalError>> {
    let left_result = eval_expr(left, context);

    match left_result {
        Err(left_errors) => {
            // partition the errors into Python evaluation errors and other errors
            let (python_eval_errors, rest_errors): (Vec<_>, Vec<_>) = left_errors
                .into_iter()
                .partition(|err| matches!(err, EvalError::PythonEvalError { .. }));

            if rest_errors.is_empty() {
                // if there are no other errors,
                // push the Python evaluation errors as warnings
                for err in python_eval_errors {
                    let EvalError::PythonEvalError {
                        function_name,
                        function_call_span,
                        message,
                        traceback,
                    } = err
                    else {
                        unreachable!("this is checked in the guard");
                    };

                    context.push_eval_warning(EvalWarning::UsedFallback {
                        function_name,
                        function_call_span,
                        message: message.clone(),
                        traceback: traceback.clone(),
                    });
                }

                // evaluate the right operand
                let right_result = eval_expr(right, context);
                right_result.map(|(value, _span)| value)
            } else {
                // if there are other errors, return them
                Err(rest_errors)
            }
        }
        Ok((value, _span)) => Ok(value),
    }
}
fn eval_function_call_args<E: ExternalEvaluationContext>(
    args: &[ir::Expr],
    context: &mut EvalContext<'_, E>,
) -> Result<Vec<(Value, Span)>, Vec<EvalError>> {
    let mut out_args = vec![];
    let mut errors = vec![];

    for arg in args {
        match eval_expr(arg, context) {
            Ok((value, value_span)) => out_args.push((value, value_span.clone())),
            Err(arg_errors) => errors.extend(arg_errors),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(out_args)
}

fn eval_function_call<E: ExternalEvaluationContext>(
    name: &ir::FunctionName,
    function_call_span: Span,
    args: Vec<(Value, Span)>,
    context: &mut EvalContext<'_, E>,
) -> Result<Value, Vec<EvalError>> {
    match name {
        ir::FunctionName::Builtin(fn_identifier, fn_identifier_span) => {
            context.evaluate_builtin_function(fn_identifier, fn_identifier_span.clone(), args)
        }
        ir::FunctionName::Imported {
            python_path,
            name,
            name_span: _,
        } => context
            .evaluate_imported_function(python_path, name, function_call_span, args)
            .map_err(|error| vec![*error]),
    }
}

fn eval_unit_cast(
    expr_result: Value,
    expr_result_span: Span,
    unit_result: Unit,
    unit_result_span: Span,
) -> Result<Value, Vec<EvalError>> {
    let result = expr_result.with_unit(unit_result);

    result.map_err(|error| match error {
        UnitConversionError::UnitMismatch {
            value_unit,
            target_unit,
        } => vec![EvalError::UnitMismatch {
            expected_unit: value_unit,
            expected_source_span: expr_result_span,
            found_unit: target_unit,
            found_span: unit_result_span,
        }],
        UnitConversionError::InvalidType {
            value_type,
            target_unit: _,
        } => vec![EvalError::TypeMismatch {
            expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
            expected_source_span: unit_result_span,
            found_type: *value_type,
            found_span: expr_result_span,
        }],
    })
}

fn eval_variable<E: ExternalEvaluationContext>(
    variable: &ir::Variable,
    context: &mut EvalContext<'_, E>,
) -> Result<Value, Vec<EvalError>> {
    match variable {
        ir::Variable::Builtin {
            ident,
            ident_span: _,
        } => Ok(context.lookup_builtin_variable(ident)),
        ir::Variable::Parameter {
            parameter_name,
            parameter_span,
        } => context.lookup_parameter_value(parameter_name, parameter_span.clone()),
        ir::Variable::External {
            reference_name,
            parameter_name,
            parameter_span,
            ..
        } => context.lookup_external_parameter_value(
            reference_name,
            parameter_name,
            parameter_span.clone(),
        ),
    }
}

fn eval_literal(value: &ir::Literal) -> Value {
    match value {
        ir::Literal::Boolean(boolean) => Value::Boolean(*boolean),
        ir::Literal::String(string) => Value::String(string.clone()),
        ir::Literal::Number(number) => {
            let number = Number::Scalar(*number);
            Value::Number(number)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{E, PI};

    use oneil_ir::{
        self as ir,
        test_helpers::{
            expr::{
                binary, builtin_call, builtin_var, compare, compare_chained, external_var,
                fallback, imported_call, lit_bool, lit_number, lit_string, param_var, unary,
                unit_cast,
            },
            unit::{UnitSpec, ir_composite_unit},
        },
    };
    use oneil_output::{
        self as output, Dimension, DisplayUnit, EvalError, ExpectedType, Number, Value, ValueType,
    };
    use oneil_shared::{
        EvalInstanceKey,
        paths::PythonPath,
        symbols::{ParameterName, PyFunctionName, ReferenceName},
    };

    use crate::{
        check_boolean, check_invalid_type, check_is_close, check_scalar_close, check_type_mismatch,
        check_units_dimensionally_eq,
        context::EvalContext,
        test_context::{TestExternalContext, test_model_path},
        test_fixtures::{output_parameter, scalar_number_type},
    };

    use super::*;

    /// Evaluates an expression in a fresh test context.
    fn eval(expr: &ir::Expr) -> Result<Value, Vec<EvalError>> {
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        eval_expr(expr, &mut context).map(|(value, _span)| value)
    }

    mod literals {
        use super::*;

        #[test]
        fn eval_number_literal() {
            let value = eval(&lit_number(42.5)).expect("eval should succeed");
            check_scalar_close(42.5, &value).assert();
        }

        #[test]
        fn eval_boolean_literal() {
            let value = eval(&lit_bool(true)).expect("eval should succeed");
            check_boolean(true, &value).assert();
        }

        #[test]
        fn eval_string_literal() {
            let value = eval(&lit_string("hello")).expect("eval should succeed");
            assert_eq!(value, Value::String("hello".to_string()));
        }
    }

    mod binary_ops {
        use super::*;

        #[test]
        fn eval_numeric_ops_table() {
            let cases = [
                (ir::BinaryOp::Add, 2.0, 3.0, 5.0),
                (ir::BinaryOp::Sub, 5.0, 2.0, 3.0),
                (ir::BinaryOp::EscapedSub, 5.0, 2.0, 3.0),
                (ir::BinaryOp::Mul, 4.0, 3.0, 12.0),
                (ir::BinaryOp::Div, 10.0, 4.0, 2.5),
                (ir::BinaryOp::EscapedDiv, 10.0, 4.0, 2.5),
                (ir::BinaryOp::Mod, 10.0, 3.0, 1.0),
                (ir::BinaryOp::Pow, 2.0, 3.0, 8.0),
            ];

            for (op, left, right, expected) in cases {
                let value = eval(&binary(op, lit_number(left), lit_number(right)))
                    .expect("eval should succeed");
                check_scalar_close(expected, &value).assert();
            }
        }

        #[test]
        fn eval_boolean_ops_table() {
            let cases = [
                (ir::BinaryOp::And, true, false, false),
                (ir::BinaryOp::Or, false, true, true),
            ];

            for (op, left, right, expected) in cases {
                let value = eval(&binary(op, lit_bool(left), lit_bool(right)))
                    .expect("eval should succeed");
                check_boolean(expected, &value).assert();
            }
        }

        #[test]
        fn eval_min_max() {
            let expr = binary(ir::BinaryOp::MinMax, lit_number(3.0), lit_number(7.0));
            let value = eval(&expr).expect("eval should succeed");
            let Value::Number(Number::Interval(interval)) = value else {
                panic!("expected interval number, got {value:?}");
            };
            check_is_close(3.0, interval.min()).assert();
            check_is_close(7.0, interval.max()).assert();
        }

        #[test]
        fn eval_add_type_mismatch() {
            let expr = binary(ir::BinaryOp::Add, lit_number(1.0), lit_bool(true));
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            check_type_mismatch(
                &errors[0],
                &ExpectedType::Number { number_type: None },
                &ValueType::Boolean,
            )
            .assert();
        }

        #[test]
        fn eval_collects_errors_from_both_operands() {
            // Both sides fail independently before the binary op runs.
            let expr = binary(
                ir::BinaryOp::Add,
                unary(ir::UnaryOp::Not, lit_number(1.0)),
                unary(ir::UnaryOp::Not, lit_number(2.0)),
            );
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 2);
            check_invalid_type(&errors[0], &ExpectedType::Boolean, &scalar_number_type()).assert();
            check_invalid_type(&errors[1], &ExpectedType::Boolean, &scalar_number_type()).assert();
        }
    }

    mod unary_ops {
        use super::*;

        #[test]
        fn eval_neg() {
            let expr = unary(ir::UnaryOp::Neg, lit_number(5.0));
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(-5.0, &value).assert();
        }

        #[test]
        fn eval_not() {
            let expr = unary(ir::UnaryOp::Not, lit_bool(true));
            let value = eval(&expr).expect("eval should succeed");
            check_boolean(false, &value).assert();
        }

        #[test]
        fn eval_neg_type_error() {
            let expr = unary(ir::UnaryOp::Neg, lit_bool(true));
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            check_invalid_type(
                &errors[0],
                &ExpectedType::Number { number_type: None },
                &ValueType::Boolean,
            )
            .assert();
        }

        #[test]
        fn eval_not_type_error() {
            let expr = unary(ir::UnaryOp::Not, lit_number(1.0));
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            check_invalid_type(&errors[0], &ExpectedType::Boolean, &scalar_number_type()).assert();
        }
    }

    mod comparisons {
        use super::*;

        #[test]
        fn eval_comparisons_table() {
            let cases = [
                (ir::ComparisonOp::Eq, 2.0, 2.0, true),
                (ir::ComparisonOp::Eq, 2.0, 3.0, false),
                (ir::ComparisonOp::NotEq, 2.0, 3.0, true),
                (ir::ComparisonOp::LessThan, 1.0, 2.0, true),
                (ir::ComparisonOp::LessThanEq, 2.0, 2.0, true),
                (ir::ComparisonOp::GreaterThan, 3.0, 1.0, true),
                (ir::ComparisonOp::GreaterThanEq, 3.0, 3.0, true),
            ];
            for (op, left, right, expected) in cases {
                let value = eval(&compare(op, lit_number(left), lit_number(right)))
                    .expect("eval should succeed");
                check_boolean(expected, &value).assert();
            }
        }

        #[test]
        fn eval_chained_comparison_true() {
            // 1 < 2 < 3
            let expr = compare_chained(
                lit_number(1.0),
                ir::ComparisonOp::LessThan,
                lit_number(2.0),
                vec![(ir::ComparisonOp::LessThan, lit_number(3.0))],
            );
            let value = eval(&expr).expect("eval should succeed");
            check_boolean(true, &value).assert();
        }

        #[test]
        fn eval_chained_comparison_false() {
            // 1 < 2 < 1.5  → false (2 < 1.5 fails)
            let expr = compare_chained(
                lit_number(1.0),
                ir::ComparisonOp::LessThan,
                lit_number(2.0),
                vec![(ir::ComparisonOp::LessThan, lit_number(1.5))],
            );
            let value = eval(&expr).expect("eval should succeed");
            check_boolean(false, &value).assert();
        }

        #[test]
        fn eval_comparison_type_mismatch() {
            let expr = compare(ir::ComparisonOp::LessThan, lit_number(1.0), lit_bool(true));
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            check_type_mismatch(
                &errors[0],
                &ExpectedType::Number { number_type: None },
                &ValueType::Boolean,
            )
            .assert();
        }

        #[test]
        fn eval_collects_errors_from_comparison_operands() {
            // Left and both right operands fail independently.
            let expr = compare_chained(
                unary(ir::UnaryOp::Not, lit_number(1.0)),
                ir::ComparisonOp::LessThan,
                unary(ir::UnaryOp::Not, lit_number(2.0)),
                vec![(
                    ir::ComparisonOp::LessThan,
                    unary(ir::UnaryOp::Not, lit_number(3.0)),
                )],
            );
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 3);
            check_invalid_type(&errors[0], &ExpectedType::Boolean, &scalar_number_type()).assert();
            check_invalid_type(&errors[1], &ExpectedType::Boolean, &scalar_number_type()).assert();
            check_invalid_type(&errors[2], &ExpectedType::Boolean, &scalar_number_type()).assert();
        }
    }

    mod fallback {
        use super::*;

        #[test]
        fn eval_fallback_uses_left_when_successful() {
            let expr = fallback(lit_number(1.0), lit_number(2.0));
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(1.0, &value).assert();
        }

        #[test]
        fn eval_fallback_propagates_non_python_errors() {
            // Left fails with a type error (not a PythonEvalError), so fallback
            // must not evaluate the right side.
            let expr = fallback(unary(ir::UnaryOp::Not, lit_number(1.0)), lit_number(2.0));
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            check_invalid_type(&errors[0], &ExpectedType::Boolean, &scalar_number_type()).assert();
        }
    }

    mod unit_cast {
        use super::*;

        #[test]
        fn eval_cast_number_to_meters() {
            let unit = ir_composite_unit([UnitSpec::new(None, Some("m"), false, 1.0)]);
            let expr = unit_cast(lit_number(5.0), unit);
            let value = eval(&expr).expect("eval should succeed");

            let Value::MeasuredNumber(measured) = value else {
                panic!("expected measured number, got {value:?}");
            };

            let Number::Scalar(scalar) = *measured.normalized_value().as_number() else {
                panic!("expected scalar");
            };
            check_is_close(5.0, scalar).assert();
            check_units_dimensionally_eq([(Dimension::Distance, 1.0)], measured.unit()).assert();
            check_is_close(1.0, measured.unit().magnitude).assert();
        }

        #[test]
        fn eval_cast_number_to_kilometers() {
            let unit = ir_composite_unit([UnitSpec::new(Some("k"), Some("m"), false, 1.0)]);
            let expr = unit_cast(lit_number(2.0), unit);
            let value = eval(&expr).expect("eval should succeed");

            let Value::MeasuredNumber(measured) = value else {
                panic!("expected measured number, got {value:?}");
            };

            // 2 km = 2000 m in normalized SI units
            let Number::Scalar(scalar) = *measured.normalized_value().as_number() else {
                panic!("expected scalar");
            };
            check_is_close(2000.0, scalar).assert();
            check_units_dimensionally_eq([(Dimension::Distance, 1.0)], measured.unit()).assert();
            check_is_close(1000.0, measured.unit().magnitude).assert();
        }

        #[test]
        fn eval_cast_rejects_boolean() {
            let unit = ir_composite_unit([UnitSpec::new(None, Some("m"), false, 1.0)]);
            let expr = unit_cast(lit_bool(true), unit);
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            check_type_mismatch(
                &errors[0],
                &ExpectedType::NumberOrMeasuredNumber { number_type: None },
                &ValueType::Boolean,
            )
            .assert();
        }

        #[test]
        fn eval_cast_unit_mismatch() {
            // First cast to meters, then attempt to cast the measured value to seconds.
            let meters = ir_composite_unit([UnitSpec::new(None, Some("m"), false, 1.0)]);
            let seconds = ir_composite_unit([UnitSpec::new(None, Some("s"), false, 1.0)]);
            let measured = unit_cast(lit_number(1.0), meters);
            let expr = unit_cast(measured, seconds);

            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            assert!(
                matches!(
                    &errors[0],
                    EvalError::UnitMismatch {
                        expected_unit: DisplayUnit::Unit { name: expected, exponent: 1.0 },
                        found_unit: DisplayUnit::Unit { name: found, exponent: 1.0 },
                        ..
                    } if expected == "m" && found == "s"
                ),
                "expected UnitMismatch m vs s, got {:?}",
                errors[0]
            );
        }
    }

    mod variables {
        use super::*;

        #[test]
        fn eval_builtin_pi() {
            let expr = builtin_var("pi");
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(PI, &value).assert();
        }

        #[test]
        fn eval_builtin_e() {
            let expr = builtin_var("e");
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(E, &value).assert();
        }

        #[test]
        fn eval_parameter_lookup() {
            let mut external = TestExternalContext::new();
            let mut context = EvalContext::new(&mut external);
            let model = EvalInstanceKey::root(test_model_path("test"));
            context.push_active_model(model);
            context.add_parameter_result(
                ParameterName::from("x"),
                Ok(output_parameter("x", Value::Number(Number::Scalar(10.0)))),
            );

            let value = eval_expr(&param_var("x"), &mut context)
                .expect("eval should succeed")
                .0;
            check_scalar_close(10.0, &value).assert();
        }

        #[test]
        fn eval_parameter_in_arithmetic() {
            let mut external = TestExternalContext::new();
            let mut context = EvalContext::new(&mut external);
            context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
            context.add_parameter_result(
                ParameterName::from("x"),
                Ok(output_parameter("x", Value::Number(Number::Scalar(3.0)))),
            );

            let expr = binary(ir::BinaryOp::Add, param_var("x"), lit_number(2.0));
            let value = eval_expr(&expr, &mut context)
                .expect("eval should succeed")
                .0;
            check_scalar_close(5.0, &value).assert();
        }

        #[test]
        fn eval_missing_parameter() {
            let mut external = TestExternalContext::new();
            let mut context = EvalContext::new(&mut external);
            let model = EvalInstanceKey::root(test_model_path("test"));
            context.push_active_model(model.clone());

            let errors =
                eval_expr(&param_var("missing"), &mut context).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            assert!(
                matches!(
                    &errors[0],
                    EvalError::ParameterHasError {
                        parameter_name,
                        parameter_instance_key,
                        ..
                    } if parameter_name.as_str() == "missing"
                        && parameter_instance_key == &model
                ),
                "expected ParameterHasError for missing, got {:?}",
                errors[0]
            );
        }

        #[test]
        fn eval_external_parameter_lookup() {
            let mut external = TestExternalContext::new();
            let mut context = EvalContext::new(&mut external);

            let parent = EvalInstanceKey::root(test_model_path("parent"));
            let child = EvalInstanceKey::root(test_model_path("child"));

            context.add_parameter_result_to(
                &child,
                ParameterName::from("y"),
                Ok(output_parameter("y", Value::Number(Number::Scalar(7.0)))),
            );

            context.push_active_model(parent);
            context.add_reference(ReferenceName::from("child"), child);

            let value = eval_expr(&external_var("y", "child"), &mut context)
                .expect("eval should succeed")
                .0;
            check_scalar_close(7.0, &value).assert();
        }
    }

    mod function_calls {
        use super::*;

        #[test]
        fn eval_builtin_abs() {
            let expr = builtin_call("abs", vec![lit_number(-4.0)]);
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(4.0, &value).assert();
        }

        #[test]
        fn eval_builtin_sqrt() {
            let expr = builtin_call("sqrt", vec![lit_number(9.0)]);
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(3.0, &value).assert();
        }

        #[test]
        fn eval_imported_function() {
            let mut external = TestExternalContext::new();
            external.register_imported_function(
                PythonPath::from_str_no_ext("helpers"),
                PyFunctionName::from("double"),
                |args| {
                    let Value::Number(Number::Scalar(n)) = &args[0].0 else {
                        panic!("expected scalar argument");
                    };
                    Ok(Value::Number(Number::Scalar(n * 2.0)))
                },
            );

            let mut context = EvalContext::new(&mut external);
            context.set_evaluation_cache_root(test_model_path("test"));

            let expr = imported_call("helpers", "double", vec![lit_number(21.0)]);
            let value = eval_expr(&expr, &mut context)
                .expect("eval should succeed")
                .0;
            check_scalar_close(42.0, &value).assert();
        }

        #[test]
        fn eval_imported_function_python_error() {
            let mut external = TestExternalContext::new();
            external.register_imported_function(
                PythonPath::from_str_no_ext("helpers"),
                PyFunctionName::from("fail"),
                |_args| {
                    Err(Box::new(EvalError::PythonEvalError {
                        function_name: PyFunctionName::from("fail"),
                        function_call_span: Span::synthetic(),
                        message: "boom".to_string(),
                        traceback: Some("traceback".to_string()),
                    }))
                },
            );

            let mut context = EvalContext::new(&mut external);
            context.set_evaluation_cache_root(test_model_path("test"));

            let expr = imported_call("helpers", "fail", vec![lit_number(1.0)]);
            let errors = eval_expr(&expr, &mut context).expect_err("eval should fail");
            assert_eq!(errors.len(), 1);
            assert!(
                matches!(
                    &errors[0],
                    EvalError::PythonEvalError {
                        function_name,
                        message,
                        traceback: Some(traceback),
                        ..
                    } if function_name.as_str() == "fail"
                        && message == "boom"
                        && traceback == "traceback"
                ),
                "expected PythonEvalError, got {:?}",
                errors[0]
            );
        }

        #[test]
        fn eval_fallback_uses_right_on_python_error() {
            let mut external = TestExternalContext::new();
            external.register_imported_function(
                PythonPath::from_str_no_ext("helpers"),
                PyFunctionName::from("fail"),
                |_args| {
                    Err(Box::new(EvalError::PythonEvalError {
                        function_name: PyFunctionName::from("fail"),
                        function_call_span: Span::synthetic(),
                        message: "boom".to_string(),
                        traceback: None,
                    }))
                },
            );

            let mut context = EvalContext::new(&mut external);
            context.set_evaluation_cache_root(test_model_path("test"));

            let expr = fallback(
                imported_call("helpers", "fail", vec![lit_number(1.0)]),
                lit_number(99.0),
            );
            let value = eval_expr(&expr, &mut context)
                .expect("eval should succeed")
                .0;
            check_scalar_close(99.0, &value).assert();

            let warnings = context.take_expression_warnings();
            assert_eq!(warnings.len(), 1);
            assert!(
                matches!(
                    &warnings[0],
                    output::EvalWarning::UsedFallback {
                        function_name,
                        message,
                        ..
                    } if function_name.as_str() == "fail" && message == "boom"
                ),
                "expected UsedFallback warning, got {:?}",
                warnings[0]
            );
        }

        #[test]
        fn eval_function_call_collects_arg_errors() {
            let expr = builtin_call(
                "abs",
                vec![
                    unary(ir::UnaryOp::Not, lit_number(1.0)),
                    unary(ir::UnaryOp::Not, lit_number(2.0)),
                ],
            );
            let errors = eval(&expr).expect_err("eval should fail");
            assert_eq!(errors.len(), 2);
            check_invalid_type(&errors[0], &ExpectedType::Boolean, &scalar_number_type()).assert();
            check_invalid_type(&errors[1], &ExpectedType::Boolean, &scalar_number_type()).assert();
        }
    }

    mod nested {
        use super::*;

        #[test]
        fn eval_nested_arithmetic() {
            // (2 + 3) * 4
            let expr = binary(
                ir::BinaryOp::Mul,
                binary(ir::BinaryOp::Add, lit_number(2.0), lit_number(3.0)),
                lit_number(4.0),
            );
            let value = eval(&expr).expect("eval should succeed");
            check_scalar_close(20.0, &value).assert();
        }

        #[test]
        fn eval_comparison_of_arithmetic() {
            // (1 + 2) == 3
            let expr = compare(
                ir::ComparisonOp::Eq,
                binary(ir::BinaryOp::Add, lit_number(1.0), lit_number(2.0)),
                lit_number(3.0),
            );
            let value = eval(&expr).expect("eval should succeed");
            check_boolean(true, &value).assert();
        }
    }
}
