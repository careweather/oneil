use std::iter;

use oneil_ir as ir;

use crate::{
    context::EvalContext,
    error::EvalError,
    unit::{ComplexDimension, Unit},
    value::{NumberValue, Value},
};

const FLOAT_COMP_DELTA: f64 = 1e-10;

pub fn floats_are_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (b >= a - epsilon) && (b <= a + epsilon)
}

#[allow(clippy::too_many_lines)]
pub fn eval_expr(expr: &ir::Expr, context: &EvalContext) -> Result<Value, Vec<EvalError>> {
    match expr {
        ir::Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } => {
            // evaluate all sub-expressions
            let (left_result, rest_results) =
                eval_comparison_subexpressions(*op, left, right, rest_chained, context)?;

            // typecheck all results
            let typecheck_errors = typecheck_comparison_results(&left_result, &rest_results);

            if !typecheck_errors.is_empty() {
                return Err(typecheck_errors);
            }

            match &left_result {
                Value::Boolean(left_result) => {
                    let comparison_eval_result = eval_bool_comparison(*left_result, &rest_results);
                    Ok(Value::Boolean(comparison_eval_result))
                }
                Value::String(left_result) => {
                    let comparison_eval_result = eval_string_comparison(left_result, &rest_results);
                    Ok(Value::Boolean(comparison_eval_result))
                }
                Value::Number {
                    value: left_result,
                    unit: left_unit,
                } => {
                    let comparison_eval_result =
                        eval_number_comparison(left_result, left_unit, &rest_results);

                    Ok(Value::Boolean(comparison_eval_result))
                }
            }
        }
        ir::Expr::BinaryOp { op, left, right } => {
            // evaluate sub-expressions
            let (left_result, right_result) = eval_binary_op_subexpressions(left, right, context)?;

            // typecheck results
            let typecheck_errors = typecheck_binary_op_results(*op, &left_result, &right_result);

            if let Some(typecheck_error) = typecheck_errors {
                return Err(vec![typecheck_error]);
            }

            match *op {
                ir::BinaryOp::Add => {
                    let Value::Number {
                        value: left_value,
                        unit: left_unit,
                    } = left_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };
                    let Value::Number {
                        value: right_value,
                        unit: right_unit,
                    } = right_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let magnitude_cast = left_unit.magnitude() / right_unit.magnitude();
                    let result_value = left_value + (right_value * magnitude_cast);

                    let result_unit = left_unit;

                    Ok(Value::Number {
                        value: result_value,
                        unit: result_unit,
                    })
                }
                ir::BinaryOp::Sub => {
                    let Value::Number {
                        value: left_value,
                        unit: left_unit,
                    } = left_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let Value::Number {
                        value: right_value,
                        unit: right_unit,
                    } = right_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let magnitude_cast = left_unit.magnitude() / right_unit.magnitude();
                    let result_value = left_value - (right_value * magnitude_cast);

                    let result_unit = left_unit;

                    Ok(Value::Number {
                        value: result_value,
                        unit: result_unit,
                    })
                }
                ir::BinaryOp::TrueSub => todo!(),
                ir::BinaryOp::Mul => {
                    let Value::Number {
                        value: left_value,
                        unit: left_unit,
                    } = left_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let Value::Number {
                        value: right_value,
                        unit: right_unit,
                    } = right_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let result_value = left_value * right_value;
                    let result_unit = left_unit * right_unit;

                    Ok(Value::Number {
                        value: result_value,
                        unit: result_unit,
                    })
                }
                ir::BinaryOp::Div => {
                    let Value::Number {
                        value: left_value,
                        unit: left_unit,
                    } = left_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let Value::Number {
                        value: right_value,
                        unit: right_unit,
                    } = right_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let result_value = left_value / right_value;
                    let result_unit = left_unit / right_unit;

                    Ok(Value::Number {
                        value: result_value,
                        unit: result_unit,
                    })
                }
                ir::BinaryOp::TrueDiv => todo!(),
                ir::BinaryOp::Mod => todo!(),
                ir::BinaryOp::Pow => todo!(),
                ir::BinaryOp::And => {
                    let Value::Boolean(left_result) = left_result else {
                        unreachable!("this should be caught by the typecheck");
                    };
                    let Value::Boolean(right_result) = right_result else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let result_value = left_result && right_result;
                    Ok(Value::Boolean(result_value))
                }
                ir::BinaryOp::Or => {
                    let Value::Boolean(left_result) = left_result else {
                        unreachable!("this should be caught by the typecheck");
                    };
                    let Value::Boolean(right_result) = right_result else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let result_value = left_result || right_result;
                    Ok(Value::Boolean(result_value))
                }
                ir::BinaryOp::MinMax => {
                    let Value::Number {
                        value: left_value,
                        unit: left_unit,
                    } = left_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };
                    let Value::Number {
                        value: right_value,
                        unit: right_unit,
                    } = right_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    // TODO: should we include this check or not?
                    //       including it means that the left value *must*
                    //       be less than the right value, which might be
                    //       hard for the programmer to ensure when it comes
                    //       to performing the operation on intervals that have
                    //       gone through many calculations
                    //
                    if left_value > right_value {
                        return Err(vec![EvalError::InvalidInterval]);
                    }

                    let result_value = left_value.tightest_enclosing_interval(&right_value);

                    Ok(Value::Number {
                        value: result_value,
                        unit: left_unit,
                    })
                }
            }
        }
        ir::Expr::UnaryOp { op, expr } => {
            let expr_result = eval_expr(expr, context)?;
            match op {
                ir::UnaryOp::Neg => match expr_result {
                    Value::Number { value, unit } => Ok(Value::Number {
                        value: -value,
                        unit,
                    }),

                    Value::Boolean(_) | Value::String(_) => Err(vec![EvalError::InvalidType]),
                },
                ir::UnaryOp::Not => match expr_result {
                    Value::Boolean(value) => Ok(Value::Boolean(!value)),
                    Value::String(_) | Value::Number { .. } => Err(vec![EvalError::InvalidType]),
                },
            }
        }
        ir::Expr::FunctionCall { name, args } => eval_function_call(name, args, context),
        ir::Expr::Variable(variable) => Ok(eval_variable(variable, context)),
        ir::Expr::Literal { value } => match value {
            ir::Literal::Number(number) => {
                let unit = Unit::new(ComplexDimension::unitless(), 1.0);
                let number_value = NumberValue::new_scalar(*number);
                Ok(Value::Number {
                    value: number_value,
                    unit,
                })
            }
            ir::Literal::String(string) => Ok(Value::String(string.to_owned())),
            ir::Literal::Boolean(boolean) => Ok(Value::Boolean(*boolean)),
        },
    }
}

pub type ComparisonOkResult = (Value, Vec<(ir::ComparisonOp, Value)>);
fn eval_comparison_subexpressions(
    op: ir::ComparisonOp,
    left: &ir::Expr,
    right: &ir::Expr,
    rest_chained: &[(ir::ComparisonOp, ir::Expr)],
    context: &EvalContext,
) -> Result<ComparisonOkResult, Vec<EvalError>> {
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
    Ok((left_result, rest_results))
}

fn typecheck_comparison_results(
    left_result: &Value,
    rest_results: &[(ir::ComparisonOp, Value)],
) -> Vec<EvalError> {
    match left_result {
        Value::Boolean(left_result) => {
            let mut errors = vec![];
            for (op, right_result) in rest_results {
                if !matches!(right_result, Value::Boolean(_)) {
                    errors.push(EvalError::InvalidType);
                }

                if !matches!(op, ir::ComparisonOp::Eq | ir::ComparisonOp::NotEq) {
                    errors.push(EvalError::InvalidOperation);
                }
            }
            errors
        }

        Value::String(left_result) => {
            let mut errors = vec![];
            for (op, right_result) in rest_results {
                if !matches!(right_result, Value::String(_)) {
                    errors.push(EvalError::InvalidType);
                }

                if !matches!(op, ir::ComparisonOp::Eq | ir::ComparisonOp::NotEq) {
                    errors.push(EvalError::InvalidOperation);
                }
            }
            errors
        }

        Value::Number {
            value: left_result,
            unit: left_unit,
        } => {
            let mut errors = vec![];
            for (op, right_result) in rest_results {
                let Value::Number {
                    unit: right_unit, ..
                } = right_result
                else {
                    errors.push(EvalError::InvalidType);
                    continue;
                };

                // TODO: this is checking for f64 equality directly, figure out how to handle f64 comparison
                if left_unit.dimensions() != right_unit.dimensions() {
                    errors.push(EvalError::InvalidUnit);
                }
            }
            errors
        }
    }
}

fn eval_bool_comparison(left_result: bool, rest_results: &[(ir::ComparisonOp, Value)]) -> bool {
    let mut comparison_eval_result = true;
    let mut left_result = left_result;

    for (op, right_result) in rest_results {
        let Value::Boolean(right_result) = right_result else {
            unreachable!("this should be caught by the typecheck");
        };

        let op_result = match op {
            ir::ComparisonOp::Eq => left_result == *right_result,
            ir::ComparisonOp::NotEq => left_result != *right_result,
            ir::ComparisonOp::LessThan
            | ir::ComparisonOp::LessThanEq
            | ir::ComparisonOp::GreaterThan
            | ir::ComparisonOp::GreaterThanEq => {
                unreachable!("this should be caught by the typecheck");
            }
        };

        comparison_eval_result = comparison_eval_result && op_result;
        left_result = *right_result;
    }

    comparison_eval_result
}

fn eval_string_comparison(left_result: &str, rest_results: &[(ir::ComparisonOp, Value)]) -> bool {
    let mut left_result = left_result;
    let mut comparison_eval_result = true;

    for (op, right_result) in rest_results {
        let Value::String(right_result) = right_result else {
            unreachable!("this should be caught by the typecheck");
        };

        let op_result = match op {
            ir::ComparisonOp::Eq => left_result == right_result,
            ir::ComparisonOp::NotEq => left_result != right_result,
            ir::ComparisonOp::LessThan
            | ir::ComparisonOp::LessThanEq
            | ir::ComparisonOp::GreaterThan
            | ir::ComparisonOp::GreaterThanEq => {
                unreachable!("this should be caught by the typecheck");
            }
        };

        comparison_eval_result = comparison_eval_result && op_result;
        left_result = right_result;
    }

    comparison_eval_result
}

fn eval_number_comparison(
    left_result: &NumberValue,
    left_unit: &Unit,
    rest_results: &[(ir::ComparisonOp, Value)],
) -> bool {
    let expected_dimensions = left_unit.dimensions();

    let mut left_result = left_result;
    let mut comparison_eval_result = true;

    for (op, right_result) in rest_results {
        let Value::Number {
            value: right_result,
            unit: right_unit,
        } = right_result
        else {
            unreachable!("this should be caught by the typecheck");
        };

        // TODO: this is checking for f64 equality directly, figure out how to handle f64 comparison
        // this is an expensive check, so we only perform it in debug mode
        debug_assert_eq!(
            left_unit.dimensions(),
            right_unit.dimensions(),
            "this should be caught by the typecheck"
        );

        let op_result = match op {
            ir::ComparisonOp::Eq => left_result == right_result,
            ir::ComparisonOp::NotEq => left_result != right_result,
            ir::ComparisonOp::LessThan => left_result < right_result,
            ir::ComparisonOp::GreaterThan => left_result > right_result,
            ir::ComparisonOp::LessThanEq => left_result <= right_result,
            ir::ComparisonOp::GreaterThanEq => left_result >= right_result,
        };

        comparison_eval_result = comparison_eval_result && op_result;
        left_result = right_result;
    }
    comparison_eval_result
}

fn eval_binary_op_subexpressions(
    left: &ir::Expr,
    right: &ir::Expr,
    context: &EvalContext,
) -> Result<(Value, Value), Vec<EvalError>> {
    let left_result = eval_expr(left, context);
    let right_result = eval_expr(right, context);

    match (left_result, right_result) {
        (Ok(left_result), Ok(right_result)) => Ok((left_result, right_result)),
        (Err(left_errors), Ok(_)) => Err(left_errors),
        (Ok(_), Err(right_errors)) => Err(right_errors),
        (Err(left_errors), Err(right_errors)) => {
            Err(left_errors.into_iter().chain(right_errors).collect())
        }
    }
}

fn typecheck_binary_op_results(
    op: ir::BinaryOp,
    left_result: &Value,
    right_result: &Value,
) -> Option<EvalError> {
    match op {
        ir::BinaryOp::Add
        | ir::BinaryOp::Sub
        | ir::BinaryOp::TrueSub
        | ir::BinaryOp::Mul
        | ir::BinaryOp::Div
        | ir::BinaryOp::TrueDiv
        | ir::BinaryOp::Mod
        | ir::BinaryOp::Pow
        | ir::BinaryOp::MinMax => match (left_result, right_result) {
            (
                Value::Number {
                    unit: left_unit, ..
                },
                Value::Number {
                    unit: right_unit, ..
                },
            ) => {
                // TODO: this is checking for f64 equality directly, figure out how to handle f64 comparison
                if left_unit.dimensions() != right_unit.dimensions() {
                    Some(EvalError::InvalidUnit)
                } else {
                    None
                }
            }
            (Value::Number { .. }, _) => Some(EvalError::InvalidType),
            (_, Value::Number { .. }) => Some(EvalError::InvalidType),
            _ => Some(EvalError::InvalidType),
        },
        ir::BinaryOp::And | ir::BinaryOp::Or => match (left_result, right_result) {
            (Value::Boolean(_), Value::Boolean(_)) => None,
            (Value::Boolean(_), _) => Some(EvalError::InvalidType),
            (_, Value::Boolean(_)) => Some(EvalError::InvalidType),
            _ => Some(EvalError::InvalidType),
        },
    }
}

fn eval_function_call(
    name: &ir::FunctionName,
    args: &[ir::Expr],
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    let args_results = args.iter().map(|arg| eval_expr(arg, context));

    let mut args = vec![];
    let mut arg_errors = vec![];

    for result in args_results {
        match result {
            Ok(value) => args.push(value),
            Err(errors) => arg_errors.extend(errors),
        }
    }

    if !arg_errors.is_empty() {
        return Err(arg_errors);
    }

    match name {
        ir::FunctionName::Builtin(fn_identifier) => {
            Ok(context.evaluate_builtin_function(fn_identifier, &args))
        }
        ir::FunctionName::Imported(fn_identifier) => {
            Ok(context.evaluate_imported_function(fn_identifier, &args))
        }
    }
}

fn eval_variable(variable: &ir::Variable, context: &EvalContext) -> Value {
    match variable {
        ir::Variable::Builtin(identifier) => context.lookup_builtin_variable(identifier),
        ir::Variable::Parameter(parameter_name) => context.lookup_parameter(parameter_name),
        ir::Variable::External {
            model,
            parameter_name,
        } => context.lookup_model_parameter(model, parameter_name),
    }
}
