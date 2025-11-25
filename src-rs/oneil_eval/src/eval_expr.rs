use std::iter;

use oneil_ir as ir;

use crate::{
    context::EvalContext,
    error::EvalError,
    unit::{ComplexDimension, Unit},
    value::{NumberValue, Value},
};

#[allow(clippy::too_many_lines)]
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
        ir::Expr::UnaryOp { op, expr } => todo!(),
        ir::Expr::FunctionCall { name, args } => todo!(),
        ir::Expr::Variable(variable) => todo!(),
        ir::Expr::Literal { value } => todo!(),
    }
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

                        match result {
                            Ok(result) => Ok(ComparisonSuccess {
                                result,
                                next_lhs: rhs,
                            }),
                            Err(error) => Err(ComparisonFailure {
                                errors: vec![error],
                                last_successful_lhs: lhs,
                            }),
                        }
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

fn eval_comparison_op(lhs: &Value, op: ir::ComparisonOp, rhs: &Value) -> Result<bool, EvalError> {
    match (lhs, rhs) {
        (Value::Boolean(lhs_bool), Value::Boolean(rhs_bool)) => match op {
            ir::ComparisonOp::Eq => Ok(lhs_bool == rhs_bool),
            ir::ComparisonOp::NotEq => Ok(lhs_bool != rhs_bool),
            ir::ComparisonOp::LessThan
            | ir::ComparisonOp::LessThanEq
            | ir::ComparisonOp::GreaterThan
            | ir::ComparisonOp::GreaterThanEq => Err(EvalError::InvalidOperation),
        },

        (Value::String(lhs_string), Value::String(rhs_string)) => match op {
            ir::ComparisonOp::Eq => Ok(lhs_string == rhs_string),
            ir::ComparisonOp::NotEq => Ok(lhs_string != rhs_string),
            ir::ComparisonOp::LessThan
            | ir::ComparisonOp::LessThanEq
            | ir::ComparisonOp::GreaterThan
            | ir::ComparisonOp::GreaterThanEq => Err(EvalError::InvalidOperation),
        },

        (
            Value::Number {
                value: lhs_number,
                unit: lhs_unit,
            },
            Value::Number {
                value: rhs_number,
                unit: rhs_unit,
            },
        ) => {
            let lhs_adjusted_number = *lhs_number * lhs_unit.magnitude();
            let rhs_adjusted_number = *rhs_number * rhs_unit.magnitude();

            match op {
                _ if lhs_unit.dimensions() != rhs_unit.dimensions() => Err(EvalError::InvalidUnit),
                ir::ComparisonOp::Eq => Ok(lhs_adjusted_number == rhs_adjusted_number),
                ir::ComparisonOp::NotEq => Ok(lhs_adjusted_number != rhs_adjusted_number),
                ir::ComparisonOp::LessThan => Ok(lhs_adjusted_number < rhs_adjusted_number),
                ir::ComparisonOp::LessThanEq => Ok(lhs_adjusted_number <= rhs_adjusted_number),
                ir::ComparisonOp::GreaterThan => Ok(lhs_adjusted_number > rhs_adjusted_number),
                ir::ComparisonOp::GreaterThanEq => Ok(lhs_adjusted_number >= rhs_adjusted_number),
            }
        }

        (lhs, _rhs) => Err(EvalError::InvalidType),
    }
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
    todo!()
}

#[allow(clippy::too_many_lines)]
pub fn eval_expr_(expr: &ir::Expr, context: &EvalContext) -> Result<Value, Vec<EvalError>> {
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
                ir::BinaryOp::TrueSub => panic!(
                    "this operation is no longer supported - use regular subtraction instead"
                ),
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
                ir::BinaryOp::TrueDiv => {
                    panic!("this operation is no longer supported - use regular division instead")
                }
                ir::BinaryOp::Mod => {
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

                    let result_value = left_value % right_value;
                    let result_unit = left_unit;

                    Ok(Value::Number {
                        value: result_value,
                        unit: result_unit,
                    })
                }
                ir::BinaryOp::Pow => {
                    let Value::Number {
                        value: base_value,
                        unit: base_unit,
                    } = left_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };
                    let Value::Number {
                        value: exponent_value,
                        unit: _exponent_unit,
                    } = right_result
                    else {
                        unreachable!("this should be caught by the typecheck");
                    };

                    let result_value = base_value.pow(&exponent_value);
                    let result_unit = match exponent_value {
                        NumberValue::Scalar(exponent) => base_unit.pow(exponent),
                        NumberValue::Interval(exponent) if base_unit.dimensions().is_unitless() => {
                            base_unit
                        }
                        NumberValue::Interval(_) => {
                            unreachable!("this should be caught by the typecheck")
                        }
                    };

                    Ok(Value::Number {
                        value: result_value,
                        unit: result_unit,
                    })
                }
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

fn typecheck_binary_op_results(
    op: ir::BinaryOp,
    left_result: &Value,
    right_result: &Value,
) -> Option<EvalError> {
    match op {
        // TODO: fix this typechecking for pow, mul, etc.
        ir::BinaryOp::Add
        | ir::BinaryOp::Sub
        | ir::BinaryOp::TrueSub
        | ir::BinaryOp::Mod
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
                if left_unit.dimensions() == right_unit.dimensions() {
                    None
                } else {
                    Some(EvalError::InvalidUnit)
                }
            }
            _ => Some(EvalError::InvalidType),
        },
        ir::BinaryOp::Mul | ir::BinaryOp::Div | ir::BinaryOp::TrueDiv => {
            match (left_result, right_result) {
                (Value::Number { .. }, Value::Number { .. }) => None,
                _ => Some(EvalError::InvalidType),
            }
        }
        ir::BinaryOp::Pow => match (left_result, right_result) {
            (
                Value::Number {
                    unit: base_unit, ..
                },
                Value::Number {
                    unit: exponent_unit,
                    value: exponent_value,
                },
            ) => {
                // the exponent must be unitless
                if !exponent_unit.dimensions().is_unitless() {
                    return Some(EvalError::HasExponentWithUnits);
                }

                // the exponent cannot be an interval
                match exponent_value {
                    NumberValue::Interval(_) => Some(EvalError::HasIntervalExponent),
                    NumberValue::Scalar(_) => None,
                }
            }
            _ => Some(EvalError::InvalidType),
        },
        ir::BinaryOp::And | ir::BinaryOp::Or => match (left_result, right_result) {
            (Value::Boolean(_), Value::Boolean(_)) => None,
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
