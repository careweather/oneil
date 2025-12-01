use std::iter;

use oneil_ir as ir;

use crate::{
    context::EvalContext,
    error::EvalError,
    value::{MeasuredNumber, Number, Unit, Value},
};

/// Evaluates an expression and returns the resulting value.
///
/// # Errors
///
/// Returns an error if the expression is invalid.
pub fn eval_expr(expr: &ir::Expr, context: &EvalContext) -> Result<Value, Vec<EvalError>> {
    match expr {
        ir::Expr::ComparisonOp {
            left,
            op,
            right,
            rest_chained,
        } => {
            let ComparisonSubexpressionsResult {
                left_result,
                rest_results,
            } = eval_comparison_subexpressions(left, *op, right, rest_chained, context)?;
            eval_comparison_chain(left_result, rest_results, context)
        }
        ir::Expr::BinaryOp { op, left, right } => {
            let BinaryOpSubexpressionsResult {
                left_result,
                right_result,
            } = eval_binary_op_subexpressions(left, right, context)?;
            eval_binary_op(left_result, *op, right_result, context)
        }
        ir::Expr::UnaryOp { op, expr } => {
            let expr_result = eval_expr(expr, context)?;
            eval_unary_op(*op, expr_result, context)
        }
        ir::Expr::FunctionCall { name, args } => {
            let args_results = eval_function_call_args(args, context)?;
            eval_function_call(name, args_results, context)
        }
        ir::Expr::Variable(variable) => eval_variable(variable, context),
        ir::Expr::Literal { value } => {
            let literal_result = eval_literal(value, context);
            Ok(literal_result)
        }
    }
}

struct ComparisonSubexpressionsResult {
    left_result: Value,
    rest_results: Vec<(ir::ComparisonOp, Value)>,
}

fn eval_comparison_subexpressions(
    left: &ir::Expr,
    op: ir::ComparisonOp,
    right: &ir::Expr,
    rest_chained: &[(ir::ComparisonOp, ir::Expr)],
    context: &EvalContext,
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
            eval_expr(right_operand, context).map(|right_result| (op, right_result))
        });

    let (left_result, rest_results) = match left_result {
        Err(left_errors) => {
            // find all evaluation errors that occurred and return them
            let errors = left_errors
                .into_iter()
                .chain(rest_results.filter_map(Result::err).flatten())
                .collect();

            return Err(errors);
        }

        Ok(left_result) => {
            let mut ok_rest_results = vec![];
            let mut err_rest_results = vec![];

            // check for evaluation errors
            for result in rest_results {
                match result {
                    Ok((op, right_operand)) => ok_rest_results.push((op, right_operand)),
                    Err(mut errors) => err_rest_results.append(&mut errors),
                }
            }

            // if any evaluation errors occurred, return them
            if !err_rest_results.is_empty() {
                return Err(err_rest_results);
            }

            // otherwise, everything was okay
            (left_result, ok_rest_results)
        }
    };
    Ok(ComparisonSubexpressionsResult {
        left_result,
        rest_results,
    })
}

fn eval_comparison_chain(
    left_result: Value,
    rest_results: Vec<(ir::ComparisonOp, Value)>,
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    // structs only used internally in this function
    struct ComparisonSuccess {
        result: bool,
        next_lhs: Value,
    }

    struct ComparisonFailure {
        errors: Vec<EvalError>,
        last_successful_lhs: Value,
    }

    let initial_result = Ok(ComparisonSuccess {
        result: true,
        next_lhs: left_result,
    });

    let comparison_result =
        rest_results
            .into_iter()
            .fold(
                initial_result,
                |comparison_result, (op, rhs)| match comparison_result {
                    Ok(ComparisonSuccess {
                        next_lhs: lhs,
                        result,
                    }) => {
                        let result = eval_comparison_op(&lhs, op, &rhs);

                        result
                            .map(|result| ComparisonSuccess {
                                result,
                                next_lhs: rhs,
                            })
                            .map_err(|error| ComparisonFailure {
                                errors: vec![error],
                                last_successful_lhs: lhs,
                            })
                    }

                    Err(ComparisonFailure {
                        errors,
                        last_successful_lhs,
                    }) => {
                        let result = eval_comparison_op(&last_successful_lhs, op, &rhs);

                        let errors = if let Err(error) = result {
                            let mut comparison_errors = errors;
                            comparison_errors.push(error);
                            comparison_errors
                        } else {
                            errors
                        };

                        Err(ComparisonFailure {
                            errors,
                            last_successful_lhs,
                        })
                    }
                },
            );

    comparison_result
        .map(|comparison_success| Value::Boolean(comparison_success.result))
        .map_err(|comparison_failure| comparison_failure.errors)
}

fn eval_comparison_op(lhs: &Value, op: ir::ComparisonOp, rhs: &Value) -> Result<bool, EvalError> {
    let result = match op {
        ir::ComparisonOp::Eq => lhs.checked_eq(rhs),
        ir::ComparisonOp::NotEq => lhs.checked_ne(rhs),
        ir::ComparisonOp::LessThan => lhs.checked_lt(rhs),
        ir::ComparisonOp::LessThanEq => lhs.checked_lte(rhs),
        ir::ComparisonOp::GreaterThan => lhs.checked_gt(rhs),
        ir::ComparisonOp::GreaterThanEq => lhs.checked_gte(rhs),
    };

    result.map_err(EvalError::ValueError)
}

struct BinaryOpSubexpressionsResult {
    left_result: Value,
    right_result: Value,
}

fn eval_binary_op_subexpressions(
    left: &ir::Expr,
    right: &ir::Expr,
    context: &EvalContext,
) -> Result<BinaryOpSubexpressionsResult, Vec<EvalError>> {
    let left_result = eval_expr(left, context);
    let right_result = eval_expr(right, context);

    match (left_result, right_result) {
        (Ok(left_result), Ok(right_result)) => Ok(BinaryOpSubexpressionsResult {
            left_result,
            right_result,
        }),
        (Err(left_errors), Ok(_)) => Err(left_errors),
        (Ok(_), Err(right_errors)) => Err(right_errors),
        (Err(left_errors), Err(right_errors)) => {
            Err(left_errors.into_iter().chain(right_errors).collect())
        }
    }
}

fn eval_binary_op(
    left_result: Value,
    op: ir::BinaryOp,
    right_result: Value,
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    let result = match op {
        ir::BinaryOp::Add => left_result.checked_add(right_result),
        ir::BinaryOp::Sub => left_result.checked_sub(right_result),
        ir::BinaryOp::TrueSub => todo!("get rid of this operation"),
        ir::BinaryOp::Mul => left_result.checked_mul(right_result),
        ir::BinaryOp::Div => left_result.checked_div(right_result),
        ir::BinaryOp::TrueDiv => todo!("get rid of this operation"),
        ir::BinaryOp::Mod => left_result.checked_rem(right_result),
        ir::BinaryOp::Pow => left_result.checked_pow(right_result),
        ir::BinaryOp::And => left_result.checked_and(right_result),
        ir::BinaryOp::Or => left_result.checked_or(right_result),
        ir::BinaryOp::MinMax => left_result.checked_min_max(right_result),
    };

    result.map_err(|error| vec![EvalError::ValueError(error)])
}

fn eval_unary_op(
    op: ir::UnaryOp,
    expr_result: Value,
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    let result = match op {
        ir::UnaryOp::Neg => expr_result.checked_neg(),
        ir::UnaryOp::Not => expr_result.checked_not(),
    };

    result.map_err(|error| vec![EvalError::ValueError(error)])
}

fn eval_function_call_args(
    args: &[ir::Expr],
    context: &EvalContext,
) -> Result<Vec<Value>, Vec<EvalError>> {
    let args_results = args.iter().map(|arg| eval_expr(arg, context));

    let mut args = vec![];
    let mut errors = vec![];

    for result in args_results {
        match result {
            Ok(value) => args.push(value),
            Err(arg_errors) => errors.extend(arg_errors),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(args)
}

fn eval_function_call(
    name: &ir::FunctionName,
    args: Vec<Value>,
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    match name {
        ir::FunctionName::Builtin(fn_identifier) => {
            context.evaluate_builtin_function(fn_identifier, args)
        }
        ir::FunctionName::Imported(fn_identifier) => {
            context.evaluate_imported_function(fn_identifier, args)
        }
    }
}

fn eval_variable(variable: &ir::Variable, context: &EvalContext) -> Result<Value, Vec<EvalError>> {
    match variable {
        ir::Variable::Builtin(identifier) => context.lookup_builtin_variable(identifier),
        ir::Variable::Parameter(parameter_name) => context.lookup_parameter(parameter_name),
        ir::Variable::External {
            model,
            parameter_name,
        } => context.lookup_model_parameter(model, parameter_name),
    }
}

fn eval_literal(value: &ir::Literal, context: &EvalContext) -> Value {
    match value {
        ir::Literal::Boolean(boolean) => Value::Boolean(*boolean),
        ir::Literal::String(string) => Value::String(string.clone()),
        ir::Literal::Number(number) => {
            let number = Number::Scalar(*number);
            let unit = Unit::unitless();
            Value::Number(MeasuredNumber::new(number, unit))
        }
    }
}
