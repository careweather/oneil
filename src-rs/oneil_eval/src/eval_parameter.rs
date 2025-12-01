use std::collections::HashSet;

use oneil_ir as ir;

use crate::{
    context::EvalContext,
    error::EvalError,
    eval_expr, eval_unit,
    value::{MeasuredNumber, Number, SizedUnit, Value},
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
pub fn eval_parameter(
    parameter: &ir::Parameter,
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
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

    let value = match value {
        Value::Boolean(_) => {
            if unit_ir.is_some() {
                return Err(vec![EvalError::BooleanCannotHaveUnit]);
            }

            value
        }
        Value::String(_) => {
            if unit_ir.is_some() {
                return Err(vec![EvalError::StringCannotHaveUnit]);
            }

            value
        }
        Value::Number(number) => {
            // evaluate the unit if it exists,
            // otherwise use the unitless unit
            let sized_unit = unit_ir
                .as_ref()
                .map(|unit| eval_unit(unit, context))
                .transpose()?
                .unwrap_or(SizedUnit::unitless());

            // if the value is unitless, assign it the given unit
            // otherwise, typecheck the value's unit against the given unit
            if number.unit.is_unitless() {
                let number_value = number.value * sized_unit.magnitude;
                let number_unit = sized_unit.unit;

                let number = MeasuredNumber {
                    value: number_value,
                    unit: number_unit,
                };

                Value::Number(number)
            } else {
                // TODO: is there anything that we need to do about the magnitude here?
                //       or is that only for displaying the value?
                if number.unit == sized_unit.unit {
                    Value::Number(number)
                } else {
                    return Err(vec![EvalError::ParameterUnitMismatch]);
                }
            }
        }
    };

    let limits = eval_limits(parameter.limits(), context)?;

    // TODO: spend more time reasoning about this. Because the limits may
    //       contain intervals, we need to consider how those intervals
    //       interact with the value.
    match limits {
        Limits::AnyStringOrBooleanOrPositiveNumber => match &value {
            Value::Number(number) if number.value < Number::Scalar(0.0) => {
                return Err(vec![EvalError::ParameterValueOutsideLimits]);
            }
            Value::Boolean(_) | Value::String(_) | Value::Number(_) => (),
        },
        Limits::NumberRange { min, max } => {
            if let Value::Number(number) = &value {
                if number.unit != min.unit || number.unit != max.unit {
                    return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
                }

                if number.value < min.value || number.value > max.value {
                    return Err(vec![EvalError::ParameterValueOutsideLimits]);
                }
            } else {
                return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
            }
        }
        Limits::NumberDiscrete { values } => {
            if let Value::Number(number) = &value {
                let mut is_inside_limits = false;
                for limit_value in values {
                    if number.unit != limit_value.unit {
                        return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
                    }

                    if number.value.inside(limit_value.value) {
                        is_inside_limits = true;
                        break;
                    }
                }
                if !is_inside_limits {
                    return Err(vec![EvalError::ParameterValueOutsideLimits]);
                }
            } else {
                return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
            }
        }
        Limits::StringDiscrete { values } => match &value {
            Value::String(string) if !values.contains(string) => {
                return Err(vec![EvalError::ParameterValueOutsideLimits]);
            }
            Value::String(_) => (),
            Value::Boolean(_) | Value::Number(_) => {
                return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
            }
        },
    }

    Ok(value)
}

fn get_piecewise_result(
    piecewise: &[ir::PiecewiseExpr],
    context: &EvalContext,
) -> Result<Value, Vec<EvalError>> {
    // evaluate each of the conditions and their bodies
    // TODO: this is slow, but we do it so that every branch is
    //       "typechecked". is there a better way to do this? Do we
    //       need to do better?
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

#[derive(Debug, Clone, PartialEq)]
pub enum Limits {
    AnyStringOrBooleanOrPositiveNumber,
    NumberRange {
        min: MeasuredNumber,
        max: MeasuredNumber,
    },
    NumberDiscrete {
        values: Vec<MeasuredNumber>,
    },
    StringDiscrete {
        values: HashSet<String>,
    },
}

#[expect(
    clippy::panic_in_result_fn,
    reason = "enforcing an invariant that should always hold"
)]
fn eval_limits(limits: &ir::Limits, context: &EvalContext) -> Result<Limits, Vec<EvalError>> {
    match limits {
        ir::Limits::Default => Ok(Limits::AnyStringOrBooleanOrPositiveNumber),
        ir::Limits::Continuous { min, max } => {
            let min = eval_expr(min, context).and_then(|value| match value {
                Value::Number(number) => Ok(number),
                Value::Boolean(_) | Value::String(_) => {
                    Err(vec![EvalError::InvalidContinuousLimitMinType])
                }
            });

            let max = eval_expr(max, context).and_then(|value| match value {
                Value::Number(number) => Ok(number),
                Value::Boolean(_) | Value::String(_) => {
                    Err(vec![EvalError::InvalidContinuousLimitMaxType])
                }
            });

            match (min, max) {
                (Ok(min), Ok(max)) => Ok(Limits::NumberRange { min, max }),
                (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => Err(errors),
                (Err(errors), Err(errors2)) => {
                    let mut errors = errors;
                    errors.extend(errors2);
                    Err(errors)
                }
            }
        }
        ir::Limits::Discrete { values } => {
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

                    for result in results {
                        match result {
                            Value::Number(number_result) => {
                                if number_result.unit == first_value.unit {
                                    numbers.push(number_result);
                                } else {
                                    errors.push(EvalError::DiscreteLimitUnitMismatch);
                                }
                            }
                            Value::Boolean(_) | Value::String(_) => {
                                errors.push(EvalError::ExpectedNumberLimit);
                            }
                        }
                    }

                    numbers.insert(0, first_value);

                    if errors.is_empty() {
                        Ok(Limits::NumberDiscrete { values: numbers })
                    } else {
                        Err(errors)
                    }
                }
                Value::Boolean(_) => Err(vec![EvalError::LimitCannotBeBoolean]),
            }
        }
    }
}
