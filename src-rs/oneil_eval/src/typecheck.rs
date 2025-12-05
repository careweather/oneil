use std::iter;

use oneil_ir as ir;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    value::{NumberType, TypeError, Unit, ValueType},
};

pub fn typecheck_expr<F: BuiltinFunction>(
    expr: &ir::Expr,
    context: &EvalContext<F>,
) -> Result<ValueType, Vec<TypeError>> {
    match expr {
        ir::Expr::ComparisonOp {
            left,
            op,
            right,
            rest_chained,
        } => typecheck_comparison_chain(left, *op, right, rest_chained, context),
        ir::Expr::BinaryOp { left, op, right } => typecheck_binary_op(left, *op, right, context),
        ir::Expr::UnaryOp { expr, op } => typecheck_unary_op(expr, *op, context),
        ir::Expr::FunctionCall { name, args } => typecheck_function_call(name, args, context),
        ir::Expr::Variable(variable) => typecheck_variable(variable, context),
        ir::Expr::Literal { value } => Ok(typecheck_literal(value)),
    }
}

fn typecheck_comparison_chain<F: BuiltinFunction>(
    left: &ir::Expr,
    op: ir::ComparisonOp,
    right: &ir::Expr,
    rest_chained: &[(ir::ComparisonOp, ir::Expr)],
    context: &EvalContext<F>,
) -> Result<ValueType, Vec<TypeError>> {
    let left_type = typecheck_expr(left, context);

    let rest = iter::once((op, right)).chain(rest_chained.iter().map(|(op, right)| (*op, right)));

    let mut errors = vec![];
    let mut lhs_type = match left_type {
        Ok(lhs_type) => Some(lhs_type),
        Err(error) => {
            errors.extend(error);
            None
        }
    };

    for (op, right) in rest {
        match typecheck_expr(right, context) {
            Ok(rhs_type) => {
                if let Some(lhs_type) = lhs_type {
                    let result = typecheck_comparison_op(&lhs_type, op, &rhs_type);
                    if let Err(error) = result {
                        errors.push(error);
                    }
                }

                lhs_type = Some(rhs_type);
            }

            Err(error) => {
                errors.extend(error);
                lhs_type = None;
            }
        }
    }

    if errors.is_empty() {
        Ok(ValueType::Boolean)
    } else {
        Err(errors)
    }
}

fn typecheck_comparison_op(
    lhs_type: &ValueType,
    op: ir::ComparisonOp,
    rhs_type: &ValueType,
) -> Result<ValueType, TypeError> {
    match op {
        ir::ComparisonOp::Eq | ir::ComparisonOp::NotEq => {
            if lhs_type == rhs_type {
                Ok(ValueType::Boolean)
            } else {
                Err(TypeError::InvalidType)
            }
        }
        ir::ComparisonOp::LessThan
        | ir::ComparisonOp::LessThanEq
        | ir::ComparisonOp::GreaterThan
        | ir::ComparisonOp::GreaterThanEq => match (lhs_type, rhs_type) {
            (
                ValueType::Number { unit: lhs_unit, .. },
                ValueType::Number { unit: rhs_unit, .. },
            ) => {
                if lhs_unit == rhs_unit {
                    Ok(ValueType::Boolean)
                } else {
                    Err(TypeError::InvalidUnit)
                }
            }
            _ => Err(TypeError::InvalidType),
        },
    }
}

fn typecheck_binary_op<F: BuiltinFunction>(
    left: &ir::Expr,
    op: ir::BinaryOp,
    right: &ir::Expr,
    context: &EvalContext<F>,
) -> Result<ValueType, Vec<TypeError>> {
    let left_type = typecheck_expr(left, context);
    let right_type = typecheck_expr(right, context);

    let (left_type, right_type) = match (left_type, right_type) {
        (Ok(left_type), Ok(right_type)) => (left_type, right_type),
        (Err(left_errors), Ok(_)) => return Err(left_errors),
        (Ok(_), Err(right_errors)) => return Err(right_errors),
        (Err(left_errors), Err(right_errors)) => {
            return Err(left_errors.into_iter().chain(right_errors).collect());
        }
    };

    match op {
        ir::BinaryOp::Add | ir::BinaryOp::Sub | ir::BinaryOp::Mod => {
            match (left_type, right_type) {
                (
                    ValueType::Number {
                        unit: lhs_unit,
                        number_type: lhs_number_type,
                    },
                    ValueType::Number {
                        unit: rhs_unit,
                        number_type: rhs_number_type,
                    },
                ) => {
                    // if the units don't match, return an error
                    if lhs_unit != rhs_unit {
                        return Err(vec![TypeError::InvalidUnit]);
                    }

                    let number_type = match (lhs_number_type, rhs_number_type) {
                        // if both numbers are scalar, the result is scalar
                        (NumberType::Scalar, NumberType::Scalar) => NumberType::Scalar,
                        // otherwise, the result is an interval
                        (_, _) => NumberType::Interval,
                    };

                    Ok(ValueType::Number {
                        unit: lhs_unit,
                        number_type,
                    })
                }
                (_, _) => Err(vec![TypeError::InvalidType]),
            }
        }
        ir::BinaryOp::Mul | ir::BinaryOp::Div => match (left_type, right_type) {
            (
                ValueType::Number {
                    unit: lhs_unit,
                    number_type: lhs_number_type,
                },
                ValueType::Number {
                    unit: rhs_unit,
                    number_type: rhs_number_type,
                },
            ) => {
                let unit = match op {
                    ir::BinaryOp::Mul => lhs_unit * rhs_unit,
                    ir::BinaryOp::Div => lhs_unit / rhs_unit,
                    ir::BinaryOp::Add
                    | ir::BinaryOp::Sub
                    | ir::BinaryOp::Mod
                    | ir::BinaryOp::Pow
                    | ir::BinaryOp::And
                    | ir::BinaryOp::Or
                    | ir::BinaryOp::MinMax => {
                        unreachable!("this branch should only handle multiplication and division")
                    }
                };

                let number_type = match (lhs_number_type, rhs_number_type) {
                    (NumberType::Scalar, NumberType::Scalar) => NumberType::Scalar,
                    (_, _) => NumberType::Interval,
                };

                Ok(ValueType::Number { unit, number_type })
            }
            (_, _) => Err(vec![TypeError::InvalidType]),
        },
        ir::BinaryOp::Pow => match (left_type, right_type) {
            (
                ValueType::Number {
                    unit: base_unit,
                    number_type: base_number_type,
                },
                ValueType::Number {
                    unit: exponent_unit,
                    number_type: exponent_number_type,
                },
            ) => {
                if !exponent_unit.is_unitless() {
                    return Err(vec![TypeError::InvalidType]);
                }

                if exponent_number_type != NumberType::Scalar {
                    return Err(vec![TypeError::InvalidNumberType]);
                }

                let unit = todo!("we can't know what this is...");

                Ok(ValueType::Number {
                    unit,
                    number_type: exponent_number_type,
                })
            }
            (_, _) => Err(vec![TypeError::InvalidType]),
        },
        ir::BinaryOp::MinMax => match (left_type, right_type) {
            (
                ValueType::Number {
                    unit: lhs_unit,
                    number_type: _,
                },
                ValueType::Number {
                    unit: rhs_unit,
                    number_type: _,
                },
            ) => {
                if lhs_unit != rhs_unit {
                    return Err(vec![TypeError::InvalidUnit]);
                }

                Ok(ValueType::Number {
                    unit: lhs_unit,
                    number_type: NumberType::Interval,
                })
            }
            (_, _) => Err(vec![TypeError::InvalidType]),
        },
        ir::BinaryOp::And | ir::BinaryOp::Or => match (left_type, right_type) {
            (ValueType::Boolean, ValueType::Boolean) => Ok(ValueType::Boolean),
            (_, _) => Err(vec![TypeError::InvalidType]),
        },
    }
}

fn typecheck_unary_op<F: BuiltinFunction>(
    expr: &ir::Expr,
    op: ir::UnaryOp,
    context: &EvalContext<F>,
) -> Result<ValueType, Vec<TypeError>> {
    let expr_type = typecheck_expr(expr, context)?;
    match op {
        ir::UnaryOp::Not => match expr_type {
            ValueType::Boolean => Ok(ValueType::Boolean),
            _ => Err(vec![TypeError::InvalidType])
        },
        ir::UnaryOp::Neg => match expr_type {
            ValueType::Number { unit, number_type } => Ok(ValueType::Number { unit, number_type })
            _ => Err(vec![TypeError::InvalidType])
        } 
    }
}

fn typecheck_function_call<F: BuiltinFunction>(
    name: &ir::FunctionName,
    args: &[ir::Expr],
    context: &EvalContext<F>,
) -> Result<ValueType, Vec<TypeError>> {
    todo!()
}

fn typecheck_variable<F: BuiltinFunction>(
    variable: &ir::Variable,
    context: &EvalContext<F>,
) -> Result<ValueType, Vec<TypeError>> {
    todo!()
}

fn typecheck_literal(
    literal: &ir::Literal,
) -> ValueType {
    match literal {
        ir::Literal::Number(_) => ValueType::Number { unit: Unit::unitless(), number_type: NumberType::Scalar },
        ir::Literal::String(_) => ValueType::String,
        ir::Literal::Boolean(_) => ValueType::Boolean,
    }
}
