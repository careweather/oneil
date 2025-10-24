use std::iter;

use oneil_ir as ir;

use crate::{
    error::EvalError,
    unit::{ComplexDimension, Unit},
    value::{NumberValue, Value},
};

const FLOAT_COMP_DELTA: f64 = 1e-10;

pub fn floats_are_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (b >= a - epsilon) && (b <= a + epsilon)
}

#[allow(clippy::too_many_lines)]
pub fn eval_expr(expr: &ir::Expr) -> Result<Value, Vec<EvalError>> {
    match expr {
        ir::Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } => {
            // evaluate all sub-expressions
            let left_result = eval_expr(left);

            let rest_results = iter::once((op, &**right))
                .chain(
                    rest_chained
                        .iter()
                        // convert from `&(_, _)` to `(&_, &_)`
                        .map(|(op, right_operand)| (op, right_operand)),
                )
                .map(|(op, right_operand)| {
                    eval_expr(right_operand).map(|right_result| (op, right_result))
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

            // typecheck all results
            let errors = match &left_result {
                Value::Boolean(left_result) => {
                    let mut errors = vec![];
                    for (op, right_result) in &rest_results {
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
                    for (op, right_result) in &rest_results {
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
                    for (op, right_result) in &rest_results {
                        let Value::Number {
                            unit: right_unit, ..
                        } = right_result
                        else {
                            errors.push(EvalError::InvalidType);
                            continue;
                        };

                        if left_unit.dimensions() != right_unit.dimensions() {
                            errors.push(EvalError::InvalidUnit);
                        }
                    }
                    errors
                }
            };

            if !errors.is_empty() {
                return Err(errors);
            }

            match &left_result {
                Value::Boolean(left_result) => {
                    let mut comparison_eval_result = true;
                    let mut left_result = left_result;

                    for (op, right_result) in &rest_results {
                        let Value::Boolean(right_result) = right_result else {
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

                    if errors.is_empty() {
                        Ok(Value::Boolean(comparison_eval_result))
                    } else {
                        Err(errors)
                    }
                }
                Value::String(left_result) => {
                    let mut left_result = left_result;
                    let mut comparison_eval_result = true;

                    for (op, right_result) in &rest_results {
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

                    if errors.is_empty() {
                        Ok(Value::Boolean(comparison_eval_result))
                    } else {
                        Err(errors)
                    }
                }
                Value::Number {
                    value: left_result,
                    unit: left_unit,
                } => {
                    let expected_dimensions = left_unit.dimensions();

                    let mut errors = vec![];
                    let mut left_result = left_result;
                    let mut comparison_eval_result = true;

                    for (op, right_result) in &rest_results {
                        let Value::Number {
                            value: right_result,
                            unit: right_unit,
                        } = right_result
                        else {
                            unreachable!("this should be caught by the typecheck");
                        };

                        // TODO: this is checking for f64 equality directly
                        if left_unit.dimensions() != right_unit.dimensions() {
                            errors.push(EvalError::InvalidUnit);
                            continue;
                        }

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

                    if errors.is_empty() {
                        Ok(Value::Boolean(comparison_eval_result))
                    } else {
                        Err(errors)
                    }
                }
            }
        }
        ir::Expr::BinaryOp { op, left, right } => todo!(),
        ir::Expr::UnaryOp { op, expr } => {
            let expr_result = eval_expr(expr)?;
            match op {
                ir::UnaryOp::Neg => match expr_result {
                    Value::Number { value, unit } => {
                        let neg_value = match value {
                            NumberValue::Scalar(value) => NumberValue::Scalar(-value),
                            NumberValue::Interval { min, max } => NumberValue::Interval {
                                // Note that min and max are reversed because of
                                // negative sign
                                min: -max,
                                max: -min,
                            },
                        };

                        Ok(Value::Number {
                            value: neg_value,
                            unit,
                        })
                    }

                    Value::Boolean(_) | Value::String(_) => Err(vec![EvalError::InvalidType]),
                },
                ir::UnaryOp::Not => match expr_result {
                    Value::Boolean(value) => Ok(Value::Boolean(!value)),
                    Value::String(_) | Value::Number { .. } => Err(vec![EvalError::InvalidType]),
                },
            }
        }
        ir::Expr::FunctionCall { name, args } => todo!(),
        ir::Expr::Variable(variable) => todo!(),
        ir::Expr::Literal { value } => match value {
            ir::Literal::Number(number) => {
                let unit = Unit::new(ComplexDimension::unitless(), 1.0);
                let number_value = NumberValue::Scalar(*number);
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
