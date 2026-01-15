use std::iter;

use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    error::EvalError,
    value::{Number, Value},
};

/// Evaluates an expression and returns the resulting value.
///
/// # Errors
///
/// Returns an error if the expression is invalid.
pub fn eval_expr<'a, F: BuiltinFunction>(
    expr: &'a ir::Expr,
    context: &EvalContext<F>,
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
            eval_unary_op(*op, expr_result, *expr_result_span).map(|result| (result, span))
        }
        ir::Expr::FunctionCall {
            name,
            args,
            span,
            name_span: _,
        } => {
            let args_results = eval_function_call_args(args, context)?;
            eval_function_call(name, args_results, context).map(|result| (result, span))
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

fn eval_comparison_subexpressions<F: BuiltinFunction>(
    left: &ir::Expr,
    op: ir::ComparisonOp,
    right: &ir::Expr,
    rest_chained: &[(ir::ComparisonOp, ir::Expr)],
    context: &EvalContext<F>,
) -> Result<ComparisonSubexpressionsResult, Vec<EvalError>> {
    let left_result = eval_expr(left, context);
    let rest_results = iter::once((op, right))
        .chain(
            rest_chained
                .iter()
                // convert from `&(_, _)` to `(&_, &_)`
                .map(|(op, right_operand)| (*op, right_operand)),
        )
        .map(|(op, right_operand)| {
            eval_expr(right_operand, context).map(|(result, span)| (op, (result, span)))
        });

    let (left_result, left_result_span, rest_results) = match left_result {
        Err(left_errors) => {
            // find all evaluation errors that occurred and return them
            let errors = left_errors
                .into_iter()
                .chain(rest_results.filter_map(Result::err).flatten())
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
                        ok_rest_results.push((op, (right_operand, *right_operand_span)));
                    }
                    Err(mut errors) => err_rest_results.append(&mut errors),
                }
            }

            // if any evaluation errors occurred, return them
            if !err_rest_results.is_empty() {
                return Err(err_rest_results);
            }

            // otherwise, everything was okay
            (left_result, *left_result_span, ok_rest_results)
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
        last_successful_lhs: (Value, Span),
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
                let comparison_result = eval_comparison_op(&lhs, lhs_span, op, &rhs, rhs_span);

                comparison_result
                    .map(|comparison_result| ComparisonSuccess {
                        result: result && comparison_result,
                        next_lhs: (rhs, rhs_span),
                    })
                    .map_err(|error| ComparisonFailure {
                        errors: vec![*error],
                        last_successful_lhs: (lhs, lhs_span),
                    })
            }

            Err(ComparisonFailure {
                errors,
                last_successful_lhs: (last_successful_lhs, last_successful_lhs_span),
            }) => {
                let result = eval_comparison_op(
                    &last_successful_lhs,
                    last_successful_lhs_span,
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
                    last_successful_lhs: (last_successful_lhs, last_successful_lhs_span),
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

    result.map_err(|error| Box::new(error.into_eval_error(lhs_span, rhs_span)))
}

struct BinaryOpSubexpressionsResult {
    left_result: Value,
    left_result_span: Span,
    right_result: Value,
    right_result_span: Span,
}

fn eval_binary_op_subexpressions<F: BuiltinFunction>(
    left: &ir::Expr,
    right: &ir::Expr,
    context: &EvalContext<F>,
) -> Result<BinaryOpSubexpressionsResult, Vec<EvalError>> {
    let left_result = eval_expr(left, context);
    let right_result = eval_expr(right, context);

    match (left_result, right_result) {
        (Ok((left_result, left_result_span)), Ok((right_result, right_result_span))) => {
            Ok(BinaryOpSubexpressionsResult {
                left_result,
                left_result_span: *left_result_span,
                right_result,
                right_result_span: *right_result_span,
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

    result.map_err(|error| vec![error.into_eval_error(left_result_span, right_result_span)])
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

    result.map_err(|error| vec![error.into_eval_error(expr_result_span)])
}

fn eval_function_call_args<F: BuiltinFunction>(
    args: &[ir::Expr],
    context: &EvalContext<F>,
) -> Result<Vec<(Value, Span)>, Vec<EvalError>> {
    let args_results = args.iter().map(|arg| eval_expr(arg, context));

    let mut args = vec![];
    let mut errors = vec![];

    for result in args_results {
        match result {
            Ok((value, value_span)) => args.push((value, *value_span)),
            Err(arg_errors) => errors.extend(arg_errors),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(args)
}

fn eval_function_call<F: BuiltinFunction>(
    name: &ir::FunctionName,
    args: Vec<(Value, Span)>,
    context: &EvalContext<F>,
) -> Result<Value, Vec<EvalError>> {
    match name {
        ir::FunctionName::Builtin(fn_identifier, fn_identifier_span) => {
            context.evaluate_builtin_function(fn_identifier, *fn_identifier_span, args)
        }
        ir::FunctionName::Imported(fn_identifier, fn_identifier_span) => {
            context.evaluate_imported_function(fn_identifier, *fn_identifier_span, args)
        }
    }
}

fn eval_variable<F: BuiltinFunction>(
    variable: &ir::Variable,
    context: &EvalContext<F>,
) -> Result<Value, Vec<EvalError>> {
    match variable {
        ir::Variable::Builtin {
            ident,
            ident_span: _,
        } => Ok(context.lookup_builtin_variable(ident)),
        ir::Variable::Parameter {
            parameter_name,
            parameter_span,
        } => context.lookup_parameter_value(parameter_name, *parameter_span),
        ir::Variable::External {
            model_path: model,
            parameter_name,
            reference_span: _,
            parameter_span,
        } => context.lookup_model_parameter_value(model, parameter_name, *parameter_span),
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
