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
