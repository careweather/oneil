use std::collections::HashSet;

use oneil_ir as ir;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    error::EvalError,
    eval_expr, eval_unit,
    value::{MeasuredNumber, Number, SizedUnit, Unit, Value},
};

/// Evaluates a parameter and returns the resulting value.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter value is invalid.
/// - The parameter value does not match the given unit, if there is one.
/// - The parameter value is outside the limits.
/// - The parameter unit does not match the limit.
pub fn eval_parameter<F: BuiltinFunction>(
    parameter: &ir::Parameter,
    context: &EvalContext<F>,
) -> Result<(Value, bool), Vec<EvalError>> {
    // TODO: this is about where we would use `trace_level`, but I'm not yet sure
    //       how to handle it.

    // evaluate the value and the unit
    let (value, unit_ir) = match parameter.value() {
        ir::ParameterValue::Simple(expr, unit) => {
            let value = eval_expr(expr, context)?;
            (value, unit)
        }
        ir::ParameterValue::Piecewise(piecewise, unit) => {
            let value = get_piecewise_result(piecewise, context)?;
            (value, unit)
        }
    };

    // typecheck the value
    let (value, is_db) = typecheck_expr_value(value, unit_ir.as_ref(), context)?;

    // check that the value is within the provided limits
    let limits = eval_limits(parameter.limits(), context)?;
    verify_value_is_within_limits(&value, limits)?;

    Ok((value, is_db))
}

fn get_piecewise_result<F: BuiltinFunction>(
    piecewise: &[ir::PiecewiseExpr],
    context: &EvalContext<F>,
) -> Result<Value, Vec<EvalError>> {
    // evaluate each of the conditions and their bodies
    let results = piecewise.iter().map(|piecewise_expr| {
        let if_result = eval_expr(piecewise_expr.if_expr(), context)?;
        let branch_result = eval_expr(piecewise_expr.expr(), context)?;

        match if_result {
            Value::Boolean(true) => Ok(Some(branch_result)),
            Value::Boolean(false) => Ok(None),
            Value::String(_) | Value::Number(_) => Err(vec![EvalError::InvalidIfExpressionType]),
        }
    });

    // find the branch that matches the condition
    // as well as any errors that occurred
    let mut result = None;
    let mut errors = Vec::new();

    for branch_result in results {
        match branch_result {
            Ok(maybe_branch_result) => {
                let Some(branch_result) = maybe_branch_result else {
                    continue;
                };

                if result.is_some() {
                    errors.push(EvalError::MultiplePiecewiseBranchesMatch);
                }

                result = Some(branch_result);
            }
            Err(e) => errors.extend(e),
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else if let Some(result) = result {
        Ok(result)
    } else {
        Err(vec![EvalError::NoPiecewiseBranchMatch])
    }
}

/// Typechecks the value of an expression against a unit.
///
/// If the parameter has a unit and the value is unitless,
/// the value is multiplied by the unit's magnitude.
///
/// In addition, if the value is a unitless number and the unit is a dB unit,
/// the value is converted from a logarithmic unit to a linear unit.
fn typecheck_expr_value<F: BuiltinFunction>(
    value: Value,
    unit_ir: Option<&ir::CompositeUnit>,
    context: &EvalContext<F>,
) -> Result<(Value, bool), Vec<EvalError>> {
    match value {
        Value::Boolean(_) => {
            if unit_ir.is_some() {
                Err(vec![EvalError::BooleanCannotHaveUnit])
            } else {
                Ok((value, false))
            }
        }
        Value::String(_) => {
            if unit_ir.is_some() {
                Err(vec![EvalError::StringCannotHaveUnit])
            } else {
                Ok((value, false))
            }
        }
        Value::Number(number) => {
            // evaluate the unit if it exists,
            // otherwise use the unitless unit
            let (sized_unit, is_db) = unit_ir
                .as_ref()
                .map(|unit| eval_unit(unit, context))
                .transpose()?
                .unwrap_or((SizedUnit::unitless(), false));

            // if the value is unitless, assign it the given unit
            // otherwise, typecheck the value's unit against the given unit
            if number.unit.is_unitless() {
                let number_value = number.value * sized_unit.magnitude;
                let number_unit = sized_unit.unit;

                // handle dB units
                let number_value = if is_db {
                    Number::Scalar(10.0).pow(number_value / Number::Scalar(10.0))
                } else {
                    number_value
                };

                let number = MeasuredNumber {
                    value: number_value,
                    unit: number_unit,
                };

                Ok((Value::Number(number), is_db))
            } else {
                // TODO: is there anything that we need to do about the magnitude here?
                //       or is that only for displaying the value?
                if number.unit == sized_unit.unit {
                    Ok((Value::Number(number), is_db))
                } else {
                    Err(vec![EvalError::ParameterUnitMismatch])
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    AnyStringOrBooleanOrPositiveNumber,
    NumberRange {
        min: Number,
        max: Number,
        unit: Unit,
    },
    NumberDiscrete {
        values: Vec<Number>,
        unit: Unit,
    },
    StringDiscrete {
        values: HashSet<String>,
    },
}

fn eval_limits<F: BuiltinFunction>(
    limits: &ir::Limits,
    context: &EvalContext<F>,
) -> Result<Limits, Vec<EvalError>> {
    match limits {
        ir::Limits::Default => Ok(Limits::AnyStringOrBooleanOrPositiveNumber),
        ir::Limits::Continuous { min, max } => eval_continuous_limits(min, max, context),
        ir::Limits::Discrete { values } => eval_discrete_limits(values, context),
    }
}

fn eval_continuous_limits<F: BuiltinFunction>(
    min: &oneil_ir::Expr,
    max: &oneil_ir::Expr,
    context: &EvalContext<F>,
) -> Result<Limits, Vec<EvalError>> {
    let min = eval_expr(min, context).and_then(|value| match value {
        Value::Number(number) => Ok(number),
        Value::Boolean(_) | Value::String(_) => Err(vec![EvalError::InvalidContinuousLimitMinType]),
    });

    let max = eval_expr(max, context).and_then(|value| match value {
        Value::Number(number) => Ok(number),
        Value::Boolean(_) | Value::String(_) => Err(vec![EvalError::InvalidContinuousLimitMaxType]),
    });

    match (min, max) {
        (Ok(min), Ok(max)) => {
            if min.unit != max.unit {
                return Err(vec![EvalError::InvalidUnit]);
            }

            Ok(Limits::NumberRange {
                min: min.value,
                max: max.value,
                unit: min.unit,
            })
        }
        (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => Err(errors),
        (Err(errors), Err(errors2)) => {
            let mut errors = errors;
            errors.extend(errors2);
            Err(errors)
        }
    }
}

#[expect(
    clippy::panic_in_result_fn,
    reason = "enforcing an invariant that should always hold"
)]
fn eval_discrete_limits<F: BuiltinFunction>(
    values: &[ir::Expr],
    context: &EvalContext<F>,
) -> Result<Limits, Vec<EvalError>> {
    let values = values.iter().map(|value| eval_expr(value, context));

    let mut errors = Vec::new();
    let mut results = Vec::new();

    for value in values {
        match value {
            Ok(value) => results.push(value),
            Err(e) => errors.extend(e),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    assert!(
        !results.is_empty(),
        "must have at least one discrete limit value"
    );

    let first_value = results.remove(0);

    match first_value {
        Value::String(first_value) => {
            let mut errors = Vec::new();
            let mut strings = HashSet::new();

            strings.insert(first_value);

            for result in results {
                match result {
                    Value::String(string) => {
                        if strings.contains(&string) {
                            errors.push(EvalError::DuplicateStringLimit);
                        } else {
                            strings.insert(string);
                        }
                    }
                    Value::Number(_) | Value::Boolean(_) => {
                        errors.push(EvalError::ExpectedStringLimit);
                    }
                }
            }

            if errors.is_empty() {
                Ok(Limits::StringDiscrete { values: strings })
            } else {
                Err(errors)
            }
        }
        Value::Number(first_value) => {
            let mut errors = Vec::new();
            let mut numbers = Vec::new();
            let limit_unit = first_value.unit;

            for result in results {
                match result {
                    Value::Number(number_result) => {
                        if number_result.unit == limit_unit {
                            numbers.push(number_result.value);
                        } else {
                            errors.push(EvalError::DiscreteLimitUnitMismatch);
                        }
                    }
                    Value::Boolean(_) | Value::String(_) => {
                        errors.push(EvalError::ExpectedNumberLimit);
                    }
                }
            }

            numbers.insert(0, first_value.value);

            if errors.is_empty() {
                Ok(Limits::NumberDiscrete {
                    values: numbers,
                    unit: limit_unit,
                })
            } else {
                Err(errors)
            }
        }
        Value::Boolean(_) => Err(vec![EvalError::LimitCannotBeBoolean]),
    }
}

fn verify_value_is_within_limits(value: &Value, limits: Limits) -> Result<(), Vec<EvalError>> {
    match limits {
        Limits::AnyStringOrBooleanOrPositiveNumber => match value {
            Value::Number(number) if number.value.min() < 0.0 => {
                Err(vec![EvalError::ParameterValueOutsideLimits])
            }
            Value::Boolean(_) | Value::String(_) | Value::Number(_) => Ok(()),
        },
        Limits::NumberRange { min, max, unit } => {
            if let Value::Number(number) = value {
                if number.unit != unit {
                    Err(vec![EvalError::ParameterUnitDoesNotMatchLimit])
                } else if number.value.min() < min.min() || number.value.max() > max.max() {
                    Err(vec![EvalError::ParameterValueOutsideLimits])
                } else {
                    Ok(())
                }
            } else {
                Err(vec![EvalError::ParameterUnitDoesNotMatchLimit])
            }
        }
        Limits::NumberDiscrete { values, unit } => {
            if let Value::Number(number) = value {
                // the number must have the same unit as the limit unit,
                // unless the limit unit is unitless
                if number.unit != unit && !unit.is_unitless() {
                    return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
                }

                let mut is_inside_limits = false;
                for limit_value in values {
                    if number.value.inside(limit_value) {
                        is_inside_limits = true;
                        break;
                    }
                }

                if is_inside_limits {
                    Ok(())
                } else {
                    Err(vec![EvalError::ParameterValueOutsideLimits])
                }
            } else {
                Err(vec![EvalError::ParameterUnitDoesNotMatchLimit])
            }
        }
        Limits::StringDiscrete { values } => match value {
            Value::String(string) if !values.contains(string) => {
                Err(vec![EvalError::ParameterValueOutsideLimits])
            }
            Value::String(_) => Ok(()),
            Value::Boolean(_) | Value::Number(_) => {
                Err(vec![EvalError::ParameterUnitDoesNotMatchLimit])
            }
        },
    }
}
