use indexmap::IndexMap;

use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    error::EvalError,
    eval_expr, eval_unit,
    value::{MeasuredNumber, Number, Unit, Value},
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
) -> Result<Value, Vec<EvalError>> {
    // TODO: this is about where we would use `trace_level`, but I'm not yet sure
    //       how to handle it.

    // evaluate the value and the unit
    let (value, expr_span, unit_ir) = match parameter.value() {
        ir::ParameterValue::Simple(expr, unit) => {
            let (value, expr_span) = eval_expr(expr, context)?;
            (value, expr_span, unit)
        }
        ir::ParameterValue::Piecewise(piecewise, unit) => {
            let param_ident = parameter.name().as_str();
            let param_ident_span = parameter.name_span();
            let (value, expr_span) =
                get_piecewise_result(piecewise, param_ident, param_ident_span, context)?;
            (value, expr_span, unit)
        }
    };

    let unit = unit_ir
        .as_ref()
        .map(|unit_ir| eval_unit(unit_ir, context))
        .transpose()?
        .flatten();

    // typecheck the value against the unit
    let value = match (value, unit) {
        (Value::Boolean(value), None) => Value::Boolean(value),
        (Value::String(value), None) => Value::String(value),
        (Value::Boolean(_), Some((_, unit_span))) => {
            return Err(vec![EvalError::BooleanCannotHaveUnit {
                expr_span: *expr_span,
                unit_span,
            }]);
        }
        (Value::String(_), Some((_, unit_span))) => {
            return Err(vec![EvalError::StringCannotHaveUnit {
                expr_span: *expr_span,
                unit_span,
            }]);
        }
        (Value::Number(value), None) => Value::Number(value),
        (Value::Number(number), Some((unit, _unit_span))) => {
            let number = MeasuredNumber::from_number_and_unit(number, unit);
            Value::MeasuredNumber(number)
        }
        (Value::MeasuredNumber(number), None) if number.unit().is_unitless() => {
            // if the unit is unitless, then we can just return the measured number
            // even if there is no explicit unit
            Value::MeasuredNumber(number)
        }
        (Value::MeasuredNumber(number), None) => {
            return Err(vec![EvalError::ParameterMissingUnitAnnotation {
                param_expr_span: *expr_span,
                param_value_unit: number.unit().display_unit.clone(),
            }]);
        }
        (Value::MeasuredNumber(number), Some((unit, unit_span)))
            if !number.unit().dimensionally_eq(&unit) =>
        {
            return Err(vec![EvalError::ParameterUnitMismatch {
                param_expr_span: *expr_span,
                param_value_unit: number.unit().display_unit.clone(),
                param_unit_span: unit_span,
                param_unit: unit.display_unit,
            }]);
        }
        (Value::MeasuredNumber(number), Some((unit, _unit_span))) => {
            Value::MeasuredNumber(number.with_unit(unit))
        }
    };

    // check that the value is within the provided limits
    let limits = eval_limits(parameter.limits(), context)?;
    verify_value_is_within_limits(&value, expr_span, limits)?;

    Ok(value)
}

fn get_piecewise_result<'a, F: BuiltinFunction>(
    piecewise: &'a [ir::PiecewiseExpr],
    param_ident: &str,
    param_ident_span: Span,
    context: &EvalContext<F>,
) -> Result<(Value, &'a Span), Vec<EvalError>> {
    // evaluate each of the conditions and their bodies
    let results = piecewise.iter().map(|piecewise_expr| {
        let (if_result, if_expr_span) = eval_expr(piecewise_expr.if_expr(), context)?;
        let (branch_result, branch_expr_span) = eval_expr(piecewise_expr.expr(), context)?;

        match if_result {
            Value::Boolean(true) => Ok(Some((branch_result, branch_expr_span, if_expr_span))),
            Value::Boolean(false) => Ok(None),
            Value::String(_) | Value::Number(_) | Value::MeasuredNumber(_) => {
                Err(vec![EvalError::InvalidIfExpressionType {
                    expr_span: *if_expr_span,
                    found_value: if_result,
                }])
            }
        }
    });

    // find the branch that matches the condition
    // as well as any errors that occurred
    let mut matching_branches = Vec::new();
    let mut errors = Vec::new();

    for branch_result in results {
        match branch_result {
            Ok(maybe_branch_result) => {
                let Some((branch_result, branch_expr_span, if_expr_span)) = maybe_branch_result
                else {
                    continue;
                };

                matching_branches.push((branch_result, branch_expr_span, if_expr_span));
            }
            Err(e) => errors.extend(e),
        }
    }

    // first, check if any errors occurred
    if !errors.is_empty() {
        return Err(errors);
    }

    // then, check if there are multiple matching branches
    if matching_branches.len() > 1 {
        let matching_branche_spans = matching_branches
            .into_iter()
            .map(|(_, _, if_expr_span)| *if_expr_span)
            .collect();

        return Err(vec![EvalError::MultiplePiecewiseBranchesMatch {
            param_ident: param_ident.to_string(),
            param_ident_span,
            matching_branche_spans,
        }]);
    }

    // finally, return the matching branch result and expression span
    // or an error if there are no matching branches
    let Some((matching_branch_result, matching_branch_expr_span, _)) = matching_branches.pop()
    else {
        return Err(vec![EvalError::NoPiecewiseBranchMatch {
            param_ident: param_ident.to_string(),
            param_ident_span,
        }]);
    };

    Ok((matching_branch_result, matching_branch_expr_span))
}

#[derive(Debug, Clone)]
enum Limits {
    AnyStringOrBooleanOrPositiveNumber,
    NumberRange {
        min: Number,
        min_expr_span: Span,
        max: Number,
        max_expr_span: Span,
        unit: Option<Unit>,
        limit_expr_span: Span,
    },
    NumberDiscrete {
        values: Vec<Number>,
        unit: Option<Unit>,
        limit_expr_span: Span,
    },
    StringDiscrete {
        // This is assumed to be small enough that a vector isn't a performance issue
        values: Vec<String>,
        limit_expr_span: Span,
    },
}

fn eval_limits<F: BuiltinFunction>(
    limits: &ir::Limits,
    context: &EvalContext<F>,
) -> Result<Limits, Vec<EvalError>> {
    match limits {
        ir::Limits::Default => Ok(Limits::AnyStringOrBooleanOrPositiveNumber),
        ir::Limits::Continuous {
            min,
            max,
            limit_expr_span,
        } => eval_continuous_limits(min, max, limit_expr_span, context),
        ir::Limits::Discrete {
            values,
            limit_expr_span,
        } => eval_discrete_limits(values, limit_expr_span, context),
    }
}

fn eval_continuous_limits<F: BuiltinFunction>(
    min: &oneil_ir::Expr,
    max: &oneil_ir::Expr,
    limit_expr_span: &Span,
    context: &EvalContext<F>,
) -> Result<Limits, Vec<EvalError>> {
    let min = eval_expr(min, context).and_then(|(value, expr_span)| match value {
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.into_number_and_unit();
            Ok((number, *expr_span, Some(unit)))
        }
        Value::Number(number) => Ok((number, *expr_span, None)),
        Value::Boolean(_) | Value::String(_) => {
            Err(vec![EvalError::InvalidContinuousLimitMinType {
                expr_span: *expr_span,
                found_value: value,
            }])
        }
    });

    let max = eval_expr(max, context).and_then(|(value, expr_span)| match value {
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.into_number_and_unit();
            Ok((number, *expr_span, Some(unit)))
        }
        Value::Number(number) => Ok((number, *expr_span, None)),
        Value::Boolean(_) | Value::String(_) => {
            Err(vec![EvalError::InvalidContinuousLimitMaxType {
                expr_span: *expr_span,
                found_value: value,
            }])
        }
    });

    let (min, min_expr_span, min_unit, max, max_expr_span, max_unit) = match (min, max) {
        (Ok((min, min_expr_span, min_unit)), Ok((max, max_expr_span, max_unit))) => {
            (min, min_expr_span, min_unit, max, max_expr_span, max_unit)
        }
        (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => return Err(errors),
        (Err(errors), Err(errors2)) => {
            let mut errors = errors;
            errors.extend(errors2);
            return Err(errors);
        }
    };

    let unit = match (min_unit, max_unit) {
        (Some(min_unit), Some(max_unit)) => {
            if !min_unit.dimensionally_eq(&max_unit) {
                return Err(vec![EvalError::MaxUnitDoesNotMatchMinUnit {
                    max_unit: max_unit.display_unit,
                    max_unit_span: max_expr_span,
                    min_unit: min_unit.display_unit,
                    min_unit_span: min_expr_span,
                }]);
            }

            Some(min_unit)
        }
        (Some(unit), None) | (None, Some(unit)) => Some(unit),
        (None, None) => None,
    };

    Ok(Limits::NumberRange {
        min,
        min_expr_span,
        max,
        max_expr_span,
        unit,
        limit_expr_span: *limit_expr_span,
    })
}

#[expect(
    clippy::panic_in_result_fn,
    reason = "enforcing an invariant that should always hold"
)]
fn eval_discrete_limits<F: BuiltinFunction>(
    values: &[ir::Expr],
    limit_expr_span: &Span,
    context: &EvalContext<F>,
) -> Result<Limits, Vec<EvalError>> {
    let values = values.iter().map(|value| eval_expr(value, context));

    let mut errors = Vec::new();
    let mut results = Vec::new();

    for value in values {
        match value {
            Ok((value, expr_span)) => results.push((value, expr_span)),
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

    let (first_value, first_expr_span) = results.remove(0);

    match first_value {
        Value::String(first_value) => {
            eval_string_discrete_limits(first_value, first_expr_span, results, limit_expr_span)
        }
        Value::Number(first_value) => {
            eval_number_discrete_limits(first_value, None, results, limit_expr_span)
        }
        Value::MeasuredNumber(first_value) => {
            let (first_value, limit_unit) = first_value.into_number_and_unit();

            eval_number_discrete_limits(
                first_value,
                Some((limit_unit, *first_expr_span)),
                results,
                limit_expr_span,
            )
        }
        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotBeDiscreteLimitValue {
            expr_span: *first_expr_span,
        }]),
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "passing by ref makes the types more complex for the rest of the function"
)]
fn eval_string_discrete_limits(
    first_value: String,
    first_expr_span: &Span,
    results: Vec<(Value, &Span)>,
    limit_expr_span: &Span,
) -> Result<Limits, Vec<EvalError>> {
    let mut seen_strings = IndexMap::new();
    let mut errors = Vec::new();

    // this is a vector of strings since for errors,
    // we want to retain the order of the strings in the limit
    let mut string_values = Vec::new();

    string_values.push(&first_value);
    seen_strings.insert(&first_value, *first_expr_span);

    for (value, expr_span) in &results {
        match value {
            Value::String(string) => {
                if let Some(original_expr_span) = seen_strings.get(string) {
                    errors.push(EvalError::DuplicateStringLimit {
                        expr_span: **expr_span,
                        original_expr_span: *original_expr_span,
                        string_value: string.clone(),
                    });
                } else {
                    string_values.push(string);
                    seen_strings.insert(string, **expr_span);
                }
            }
            Value::Number(_) | Value::MeasuredNumber(_) | Value::Boolean(_) => {
                errors.push(EvalError::ExpectedStringLimit {
                    expr_span: **expr_span,
                    found_value: value.clone(),
                });
            }
        }
    }

    if errors.is_empty() {
        let strings = string_values.into_iter().cloned().collect();
        Ok(Limits::StringDiscrete {
            values: strings,
            limit_expr_span: *limit_expr_span,
        })
    } else {
        Err(errors)
    }
}

fn eval_number_discrete_limits(
    first_value: Number,
    limit_unit: Option<(Unit, Span)>,
    results: Vec<(Value, &Span)>,
    limit_expr_span: &Span,
) -> Result<Limits, Vec<EvalError>> {
    let mut errors = Vec::new();
    let mut numbers = Vec::new();
    let mut limit_unit = limit_unit;

    numbers.push(first_value);

    for (value, expr_span) in results {
        match value {
            Value::MeasuredNumber(number_result) => {
                let (number_result, number_result_unit) = number_result.into_number_and_unit();

                match &limit_unit {
                    Some((limit_unit, _)) if number_result_unit.dimensionally_eq(limit_unit) => {
                        numbers.push(number_result);
                    }
                    Some((limit_unit, limit_expr_span)) => {
                        errors.push(EvalError::DiscreteLimitUnitMismatch {
                            limit_unit: limit_unit.display_unit.clone(),
                            limit_span: *limit_expr_span,
                            value_unit: number_result_unit.display_unit.clone(),
                            value_unit_span: *expr_span,
                        });
                    }
                    None => {
                        limit_unit = Some((number_result_unit, *expr_span));
                        numbers.push(number_result);
                    }
                }
            }
            Value::Number(number_result) => {
                numbers.push(number_result);
            }
            Value::Boolean(_) | Value::String(_) => {
                errors.push(EvalError::ExpectedNumberLimit {
                    expr_span: *expr_span,
                    found_value: value,
                });
            }
        }
    }

    let limit_unit = limit_unit.map(|(unit, _)| unit);

    if errors.is_empty() {
        Ok(Limits::NumberDiscrete {
            values: numbers,
            unit: limit_unit,
            limit_expr_span: *limit_expr_span,
        })
    } else {
        Err(errors)
    }
}

fn verify_value_is_within_limits(
    value: &Value,
    param_expr_span: &Span,
    limits: Limits,
) -> Result<(), Vec<EvalError>> {
    match limits {
        Limits::AnyStringOrBooleanOrPositiveNumber => {
            verify_value_is_within_default_limits(value, param_expr_span)
        }
        Limits::NumberRange {
            min,
            min_expr_span,
            max,
            max_expr_span,
            unit,
            limit_expr_span,
        } => verify_value_is_within_number_range(
            value,
            param_expr_span,
            min,
            min_expr_span,
            max,
            max_expr_span,
            unit,
            limit_expr_span,
        ),
        Limits::NumberDiscrete {
            values,
            unit,
            limit_expr_span,
        } => verify_value_is_within_number_discrete_limit(
            value,
            param_expr_span,
            values,
            unit,
            limit_expr_span,
        ),
        Limits::StringDiscrete {
            values,
            limit_expr_span,
        } => verify_value_is_within_string_discrete_limit(
            value,
            param_expr_span,
            values,
            limit_expr_span,
        ),
    }
}

fn verify_value_is_within_default_limits(
    value: &Value,
    param_expr_span: &Span,
) -> Result<(), Vec<EvalError>> {
    match value {
        Value::MeasuredNumber(number) if number.normalized_value().min() < 0.0 => {
            Err(vec![EvalError::ParameterValueBelowDefaultLimits {
                param_expr_span: *param_expr_span,
                param_value: value.clone(),
            }])
        }
        Value::Number(number) if number.min() < 0.0 => {
            Err(vec![EvalError::ParameterValueBelowDefaultLimits {
                param_expr_span: *param_expr_span,
                param_value: value.clone(),
            }])
        }
        Value::Boolean(_) | Value::String(_) | Value::Number(_) | Value::MeasuredNumber(_) => {
            Ok(())
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "each argument has an associated span"
)]
fn verify_value_is_within_number_range(
    value: &Value,
    param_expr_span: &Span,
    min: Number,
    min_expr_span: Span,
    max: Number,
    max_expr_span: Span,
    unit: Option<Unit>,
    limit_expr_span: Span,
) -> Result<(), Vec<EvalError>> {
    match value {
        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotHaveALimit {
            expr_span: *param_expr_span,
            limit_span: limit_expr_span,
        }]),
        Value::String(_) => Err(vec![EvalError::StringCannotHaveNumberLimit {
            param_expr_span: *param_expr_span,
            param_value: value.clone(),
            limit_span: limit_expr_span,
        }]),
        Value::Number(number) => {
            if let Some(limit_unit) = unit {
                Err(vec![EvalError::UnitlessNumberCannotHaveLimitWithUnit {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    limit_span: limit_expr_span,
                    limit_unit: limit_unit.display_unit,
                }])
            } else if number.min() < min.min() {
                Err(vec![EvalError::ParameterValueBelowContinuousLimits {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    min_expr_span,
                    min_value: Value::Number(min),
                }])
            } else if number.max() > max.max() {
                Err(vec![EvalError::ParameterValueAboveContinuousLimits {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    max_expr_span,
                    max_value: Value::Number(max),
                }])
            } else {
                Ok(())
            }
        }
        Value::MeasuredNumber(number) => {
            let limit_unit = match unit {
                Some(unit) if number.unit().dimensionally_eq(&unit) => unit,
                Some(unit) => {
                    return Err(vec![EvalError::LimitUnitDoesNotMatchParameterUnit {
                        param_unit: number.unit().display_unit.clone(),
                        limit_span: limit_expr_span,
                        limit_unit: unit.display_unit,
                    }]);
                }
                None => number.unit().clone(),
            };

            // the min and the max must be converted to the same unit as the number
            let adjusted_min = MeasuredNumber::from_number_and_unit(min, limit_unit.clone());
            let adjusted_max = MeasuredNumber::from_number_and_unit(max, limit_unit);

            if number.normalized_value().min() < adjusted_min.normalized_value().min() {
                Err(vec![EvalError::ParameterValueBelowContinuousLimits {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    min_expr_span,
                    min_value: Value::Number(min),
                }])
            } else if number.normalized_value().max() > adjusted_max.normalized_value().max() {
                Err(vec![EvalError::ParameterValueAboveContinuousLimits {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    max_expr_span,
                    max_value: Value::Number(max),
                }])
            } else {
                Ok(())
            }
        }
    }
}

fn verify_value_is_within_number_discrete_limit(
    value: &Value,
    param_expr_span: &Span,
    values: Vec<Number>,
    unit: Option<Unit>,
    limit_expr_span: Span,
) -> Result<(), Vec<EvalError>> {
    match value {
        Value::MeasuredNumber(number) => {
            let limit_unit = match unit {
                Some(limit_unit) if number.unit().dimensionally_eq(&limit_unit) => limit_unit,
                Some(limit_unit) => {
                    return Err(vec![EvalError::LimitUnitDoesNotMatchParameterUnit {
                        param_unit: number.unit().display_unit.clone(),
                        limit_span: limit_expr_span,
                        limit_unit: limit_unit.display_unit,
                    }]);
                }
                None => number.unit().clone(),
            };

            let is_inside_limits = values.iter().any(|limit_value| {
                let adjusted_limit_value =
                    MeasuredNumber::from_number_and_unit(*limit_value, limit_unit.clone());
                adjusted_limit_value
                    .normalized_value()
                    .contains(number.normalized_value())
            });

            if is_inside_limits {
                Ok(())
            } else {
                let values: Vec<Value> = values
                    .into_iter()
                    .map(|value| {
                        let measured_number =
                            MeasuredNumber::from_number_and_unit(value, number.unit().clone());
                        Value::MeasuredNumber(measured_number)
                    })
                    .collect();

                Err(vec![EvalError::ParameterValueNotInDiscreteLimits {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    limit_expr_span,
                    limit_values: values,
                }])
            }
        }

        Value::Number(number) => {
            if let Some(limit_unit) = unit {
                return Err(vec![EvalError::UnitlessNumberCannotHaveLimitWithUnit {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    limit_span: limit_expr_span,
                    limit_unit: limit_unit.display_unit,
                }]);
            }

            let is_inside_limits = values
                .iter()
                .any(|limit_value| limit_value.contains(number));

            if is_inside_limits {
                Ok(())
            } else {
                let values: Vec<Value> = values.into_iter().map(Value::Number).collect();
                Err(vec![EvalError::ParameterValueNotInDiscreteLimits {
                    param_expr_span: *param_expr_span,
                    param_value: value.clone(),
                    limit_expr_span,
                    limit_values: values,
                }])
            }
        }

        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotHaveALimit {
            expr_span: *param_expr_span,
            limit_span: limit_expr_span,
        }]),

        Value::String(_) => Err(vec![EvalError::StringCannotHaveNumberLimit {
            param_expr_span: *param_expr_span,
            param_value: value.clone(),
            limit_span: limit_expr_span,
        }]),
    }
}

fn verify_value_is_within_string_discrete_limit(
    value: &Value,
    param_expr_span: &Span,
    values: Vec<String>,
    limit_expr_span: Span,
) -> Result<(), Vec<EvalError>> {
    match value {
        Value::String(string) if !values.contains(string) => {
            let values: Vec<Value> = values.into_iter().map(Value::String).collect();
            Err(vec![EvalError::ParameterValueNotInDiscreteLimits {
                param_expr_span: *param_expr_span,
                param_value: value.clone(),
                limit_expr_span,
                limit_values: values,
            }])
        }
        Value::String(_) => Ok(()),
        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotHaveALimit {
            expr_span: *param_expr_span,
            limit_span: limit_expr_span,
        }]),
        Value::Number(_) | Value::MeasuredNumber(_) => {
            Err(vec![EvalError::NumberCannotHaveStringLimit {
                param_expr_span: *param_expr_span,
                param_value: value.clone(),
                limit_span: limit_expr_span,
            }])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_is_close, assert_units_dimensionally_eq,
        builtin::{self},
        value::Dimension,
    };

    use super::*;

    #[test]
    fn eval_no_unit() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, []);
        let context = helper::create_eval_context([]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number else {
            panic!("expected scalar");
        };

        // check the value
        assert_is_close!(1.0, value);
    }

    #[test]
    fn eval_with_unit_m() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("m", 1.0)]);
        let context = helper::create_eval_context([]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // check the value
        assert_is_close!(1.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_with_unit_km() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("km", 1.0)]);
        let context = helper::create_eval_context([]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // check the value
        // 1.0 km = 1000.0 m
        assert_is_close!(1000.0, value);
        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1000.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_with_unit_km_per_hr() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("km", 1.0), ("hr", -1.0)]);
        let context = helper::create_eval_context([]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0), (Dimension::Time, -1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // check the value
        // 1.0 km/hr = 1000.0 m / 3600.0 s = 0.277777... m/s
        assert_is_close!(1000.0 / 3600.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1000.0 / 3600.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_with_unit_db() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("dB", 1.0)]);
        let context = helper::create_eval_context([]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // check the value
        // 1.0 dB = 10^(1.0/10.0) = 10^0.1 = 1.258925...
        assert_is_close!(10.0_f64.powf(0.1), value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(unit.is_db);
    }

    #[test]
    fn eval_with_unit_dbw() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("dBW", 1.0)]);
        let context = helper::create_eval_context([]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // 1.0 dBW = 10^(1.0/10.0) = 10^0.1 = 1.258925...
        assert_is_close!(10.0_f64.powf(0.1), value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(unit.is_db);
    }

    #[test]
    fn eval_add_parameters_with_different_units() {
        // setup context with x = 1.0 m and y = 1.0 km
        let context = helper::create_eval_context([
            ("x", 1.0, vec![("m", 1.0)]),
            ("y", 1.0, vec![("km", 1.0)]),
        ]);

        // setup parameter z = x + y with unit km
        let parameter = helper::build_add_parameter("z", "x", "y", [("km", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // x + y = 1.0 m + 1000.0 m = 1001.0 m
        // The value is stored in base units (meters)
        assert_is_close!(1001.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1000.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_add_parameters_kg_m_per_s2_and_n() {
        // setup context with x = 1.0 kg*m/s^2 and y = 1.0 N
        let context = helper::create_eval_context([
            ("x", 1.0, vec![("kg", 1.0), ("m", 1.0), ("s", -2.0)]),
            ("y", 1.0, vec![("N", 1.0)]),
        ]);

        // setup parameter z = x + y with unit N
        let parameter = helper::build_add_parameter("z", "x", "y", [("N", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 1.0),
            (Dimension::Time, -2.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // x + y = 1.0 N + 1.0 N = 2.0 N
        // The value is stored in base units
        assert_is_close!(2.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_add_parameters_dbw_and_w() {
        // setup context with x = 1.0 dBW and y = 1.0 W
        let context = helper::create_eval_context([
            ("x", 1.0, vec![("dBW", 1.0)]),
            ("y", 1.0, vec![("W", 1.0)]),
        ]);

        // setup parameter z = x + y with unit W
        let parameter = helper::build_add_parameter("z", "x", "y", [("W", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // x = 1.0 dBW = 10^(1.0/10.0) = 10^0.1 = 1.258925... W
        // y = 1.0 W
        // x + y = 1.258925... W + 1.0 W = 2.258925... W
        assert_is_close!(10.0_f64.powf(0.1) + 1.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_exponent_parameter_w_squared() {
        // setup context with x = 1.0 W
        let context = helper::create_eval_context([("x", 1.0, vec![("W", 1.0)])]);

        // setup parameter y = x^2 with unit W^2
        let parameter = helper::build_exponent_parameter("y", "x", 2.0, [("W", 2.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 2.0),
            (Dimension::Distance, 4.0),
            (Dimension::Time, -6.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // y = x^2 = (1.0 W)^2 = 1.0 W^2
        assert_is_close!(1.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_mul_function() {
        // setup context with x = 3.0 m and y = 2.0 m
        let context = helper::create_eval_context([
            ("x", 3.0, vec![("m", 1.0)]),
            ("y", 2.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = x * y with unit m^2
        let parameter = helper::build_mul_parameter("z", "x", "y", [("m", 2.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 2.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = x * y = 3.0 m * 2.0 m = 6.0 m^2
        assert_is_close!(6.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_div_function() {
        // setup context with x = 6.0 m^2 and y = 2.0 m
        let context = helper::create_eval_context([
            ("x", 6.0, vec![("m", 2.0)]),
            ("y", 2.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = x / y with unit m
        let parameter = helper::build_div_parameter("z", "x", "y", [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = x / y = 6.0 m^2 / 2.0 m = 3.0 m
        assert_is_close!(3.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_escaped_div_function() {
        // setup context with x = 6.0 m and y = 2.0 m
        let context = helper::create_eval_context([
            ("x", 6.0, vec![("m", 1.0)]),
            ("y", 2.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = x // y with unit m
        // Escaped division requires matching units
        let parameter = helper::build_escaped_div_parameter("z", "x", "y", []);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = x // y = 6.0 m // 2.0 m = 3.0
        // For scalars, escaped division behaves the same as regular division
        assert_is_close!(3.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_escaped_sub_function() {
        // setup context with x = 6.0 m and y = 2.0 m
        let context = helper::create_eval_context([
            ("x", 6.0, vec![("m", 1.0)]),
            ("y", 2.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = x -- y with unit m
        // Escaped subtraction requires matching units
        let parameter = helper::build_escaped_sub_parameter("z", "x", "y", [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = x -- y = 6.0 m -- 2.0 m = 4.0 m
        // For scalars, escaped subtraction behaves the same as regular subtraction
        assert_is_close!(4.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_mod_function() {
        // setup context with x = 7.0 m and y = 3.0 m
        let context = helper::create_eval_context([
            ("x", 7.0, vec![("m", 1.0)]),
            ("y", 3.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = x % y with unit m
        let parameter = helper::build_mod_parameter("z", "x", "y", [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = x % y = 7.0 m % 3.0 m = 1.0 m
        assert_is_close!(1.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_sqrt_function() {
        // setup context with x = 4.0 m^2
        let context = helper::create_eval_context([("x", 4.0, vec![("m", 2.0)])]);

        // setup parameter y = sqrt(x) with unit m
        let parameter = helper::build_function_call_parameter("y", "sqrt", ["x"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // y = sqrt(x) = sqrt(4.0 m^2) = 2.0 m
        assert_is_close!(2.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_min_function() {
        // setup context with x = 3.0 m and y = 5.0 m
        let context = helper::create_eval_context([
            ("x", 3.0, vec![("m", 1.0)]),
            ("y", 5.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = min(x, y) with unit m
        let parameter = helper::build_function_call_parameter("z", "min", ["x", "y"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = min(x, y) = min(3.0 m, 5.0 m) = 3.0 m
        assert_is_close!(3.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_min_function_with_interval() {
        // setup context
        let mut context = helper::create_eval_context([]);

        // add x as an interval parameter [2.0, 4.0] m
        let x_parameter = helper::build_interval_parameter("x", 2.0, 4.0, [("m", 1.0)]);
        let x_value = eval_parameter(&x_parameter, &context).expect("eval should succeed");
        let parameter_result = helper::build_parameter_result("x", x_value);
        context.add_parameter_result("x".to_string(), Ok(parameter_result));

        // setup parameter z = min(x) with unit m
        let parameter = helper::build_function_call_parameter("z", "min", ["x"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // x = [2.0, 4.0] m
        // min(x) = min(2.0, 4.0) = 2.0 m
        assert_is_close!(2.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_max_function() {
        // setup context with x = 3.0 m and y = 5.0 m
        let context = helper::create_eval_context([
            ("x", 3.0, vec![("m", 1.0)]),
            ("y", 5.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = max(x, y) with unit m
        let parameter = helper::build_function_call_parameter("z", "max", ["x", "y"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = max(x, y) = max(3.0 m, 5.0 m) = 5.0 m
        assert_is_close!(5.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_max_function_with_interval() {
        // setup context
        let mut context = helper::create_eval_context([]);

        // add x as an interval parameter [2.0, 4.0] m
        let x_parameter = helper::build_interval_parameter("x", 2.0, 4.0, [("m", 1.0)]);
        let x_value = eval_parameter(&x_parameter, &context).expect("eval should succeed");
        let parameter_result = helper::build_parameter_result("x", x_value);
        context.add_parameter_result("x".to_string(), Ok(parameter_result));

        // setup parameter z = max(x) with unit m
        let parameter = helper::build_function_call_parameter("z", "max", ["x"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // x = [2.0, 4.0] m
        // max(x) = max(2.0, 4.0) = 4.0 m
        assert_is_close!(4.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_range_function() {
        // setup context
        let mut context = helper::create_eval_context([]);

        // add x as an interval parameter [2.0, 4.0] m
        let x_parameter = helper::build_interval_parameter("x", 2.0, 4.0, [("m", 1.0)]);
        let x_value = eval_parameter(&x_parameter, &context).expect("eval should succeed");
        let parameter_result = helper::build_parameter_result("x", x_value);
        context.add_parameter_result("x".to_string(), Ok(parameter_result));

        // setup parameter z = range(x) with unit m
        let parameter = helper::build_function_call_parameter("z", "range", ["x"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // x = [2.0, 4.0] m
        // range(x) = max(2.0, 4.0) - min(2.0, 4.0) = 4.0 - 2.0 = 2.0 m
        assert_is_close!(2.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    #[test]
    fn eval_mid_function() {
        // setup context with x = 2.0 m and y = 4.0 m
        let context = helper::create_eval_context([
            ("x", 2.0, vec![("m", 1.0)]),
            ("y", 4.0, vec![("m", 1.0)]),
        ]);

        // setup parameter z = mid(x, y) with unit m
        let parameter = helper::build_function_call_parameter("z", "mid", ["x", "y"], [("m", 1.0)]);
        let parameter_value = eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value else {
            panic!("expected number");
        };

        let normalized_value = number.normalized_value();
        let unit = number.unit();

        let Number::Scalar(value) = *normalized_value.as_number() else {
            panic!("expected scalar");
        };

        // z = mid(x, y) = (x + y) / 2 = (2.0 m + 4.0 m) / 2 = 3.0 m
        assert_is_close!(3.0, value);

        // check the unit
        assert_units_dimensionally_eq!(expected_dimensions, unit);
        assert_is_close!(1.0, unit.magnitude);
        assert!(!unit.is_db);
    }

    mod helper {
        use super::*;

        use std::path::PathBuf;

        use crate::builtin::BuiltinMap;

        use crate::builtin::std::StdBuiltinFunction;
        use crate::context::EvalContext;
        use crate::output::eval_result;

        use oneil_ir::DisplayCompositeUnit;
        use oneil_shared::span::SourceLocation;

        use oneil_shared::span::Span;

        /// Returns a dummy span for use in test parameters.
        ///
        /// This function creates a span with all fields set to zero.
        /// It is not intended to be directly tested, but rather used
        /// as a placeholder when constructing IR nodes for testing.
        fn random_span() -> Span {
            let start = SourceLocation {
                offset: 0,
                line: 0,
                column: 0,
            };
            let end = SourceLocation {
                offset: 0,
                line: 0,
                column: 0,
            };
            Span::new(start, end)
        }

        /// Returns a display composite unit that isn't intended to be tested.
        fn unimportant_display_composite_unit() -> DisplayCompositeUnit {
            DisplayCompositeUnit::BaseUnit(ir::DisplayUnit::new("unimportant".to_string(), 1.0))
        }

        /// Builds a simple parameter with a literal numeric value.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value` - The numeric value of the parameter
        /// * `units` - An iterator of unit names and their exponents (e.g., `[("m", 1.0), ("s", -1.0)]` for m/s)
        ///
        /// # Returns
        ///
        /// A parameter with a literal number expression and the specified units.
        pub fn build_simple_parameter(
            name: &str,
            value: f64,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr = ir::Expr::Literal {
                span: random_span(),
                value: ir::Literal::Number(value),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with an interval value (min-max expression).
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The minimum value of the interval
        /// * `value_b` - The maximum value of the interval
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with a min-max binary operation that creates an interval value.
        pub fn build_interval_parameter(
            name: &str,
            value_a: f64,
            value_b: f64,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Literal {
                span: random_span(),
                value: ir::Literal::Number(value_a),
            };

            let expr_b = ir::Expr::Literal {
                span: random_span(),
                value: ir::Literal::Number(value_b),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::MinMax,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with an addition expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the first parameter to add
        /// * `value_b` - The name of the second parameter to add
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with an addition binary operation: `value_a + value_b`.
        pub fn build_add_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_a.to_string()),
                    random_span(),
                ),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_b.to_string()),
                    random_span(),
                ),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Add,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with a multiplication expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the first parameter to multiply
        /// * `value_b` - The name of the second parameter to multiply
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with a multiplication binary operation: `value_a * value_b`.
        pub fn build_mul_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_a.to_string()),
                    random_span(),
                ),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_b.to_string()),
                    random_span(),
                ),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Mul,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with a division expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the dividend parameter
        /// * `value_b` - The name of the divisor parameter
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with a division binary operation: `value_a / value_b`.
        pub fn build_div_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_a.to_string()),
                    random_span(),
                ),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_b.to_string()),
                    random_span(),
                ),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Div,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with an escaped division expression.
        ///
        /// Escaped division (`//`) requires matching units and uses non-standard
        /// interval arithmetic (divides min by min and max by max for intervals).
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the dividend parameter
        /// * `value_b` - The name of the divisor parameter
        /// * `units` - An iterator of unit names and their exponents (must match the units of both operands)
        ///
        /// # Returns
        ///
        /// A parameter with an escaped division binary operation: `value_a // value_b`.
        pub fn build_escaped_div_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_a.to_string()),
                    random_span(),
                ),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_b.to_string()),
                    random_span(),
                ),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::EscapedDiv,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with an escaped subtraction expression.
        ///
        /// Escaped subtraction (`--`) requires matching units and uses non-standard
        /// interval arithmetic (subtracts min from min and max from max for intervals).
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the minuend parameter
        /// * `value_b` - The name of the subtrahend parameter
        /// * `units` - An iterator of unit names and their exponents (must match the units of both operands)
        ///
        /// # Returns
        ///
        /// A parameter with an escaped subtraction binary operation: `value_a -- value_b`.
        pub fn build_escaped_sub_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_a.to_string()),
                    random_span(),
                ),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_b.to_string()),
                    random_span(),
                ),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::EscapedSub,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with a modulo expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the dividend parameter
        /// * `value_b` - The name of the divisor parameter
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with a modulo binary operation: `value_a % value_b`.
        pub fn build_mod_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_a.to_string()),
                    random_span(),
                ),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(value_b.to_string()),
                    random_span(),
                ),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Mod,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with an exponentiation expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `base` - The name of the base parameter
        /// * `exponent` - The exponent value (a literal number)
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with an exponentiation binary operation: `base ^ exponent`.
        pub fn build_exponent_parameter(
            name: &str,
            base: &str,
            exponent: f64,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let expr_base = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(
                    ir::ParameterName::new(base.to_string()),
                    random_span(),
                ),
            };

            let expr_exponent = ir::Expr::Literal {
                span: random_span(),
                value: ir::Literal::Number(exponent),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Pow,
                left: Box::new(expr_base),
                right: Box::new(expr_exponent),
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Builds a parameter with a builtin function call expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `function` - The name of the builtin function to call
        /// * `args` - An iterator of parameter names to pass as arguments
        /// * `units` - An iterator of unit names and their exponents
        ///
        /// # Returns
        ///
        /// A parameter with a function call expression: `function(arg1, arg2, ...)`.
        pub fn build_function_call_parameter(
            name: &str,
            function: &str,
            args: impl IntoIterator<Item = &'static str>,
            units: impl IntoIterator<Item = (&'static str, f64)>,
        ) -> ir::Parameter {
            let args = args
                .into_iter()
                .map(|arg| ir::Expr::Variable {
                    span: random_span(),
                    variable: ir::Variable::parameter(
                        ir::ParameterName::new(arg.to_string()),
                        random_span(),
                    ),
                })
                .collect();

            let expr = ir::Expr::FunctionCall {
                span: random_span(),
                name_span: random_span(),
                name: ir::FunctionName::Builtin(
                    ir::Identifier::new(function.to_string()),
                    random_span(),
                ),
                args,
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| {
                    ir::Unit::new(
                        random_span(),
                        unit.to_string(),
                        random_span(),
                        exponent,
                        None,
                    )
                })
                .collect();
            let units =
                ir::CompositeUnit::new(units, unimportant_display_composite_unit(), random_span());

            ir::Parameter::new(
                ir::Dependencies::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::Label::new(name.to_string()),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Creates an evaluation context with pre-defined parameters.
        ///
        /// # Arguments
        ///
        /// * `previous_parameters` - An iterator of tuples containing:
        ///   - Parameter name
        ///   - Parameter value (a literal number)
        ///   - Units as a vector of tuples (unit name, exponent)
        ///
        /// # Returns
        ///
        /// An evaluation context with the standard builtin values, functions, units, and prefixes,
        /// and with the specified parameters already evaluated and added to the context.
        pub fn create_eval_context(
            previous_parameters: impl IntoIterator<Item = (&'static str, f64, Vec<(&'static str, f64)>)>,
        ) -> EvalContext<StdBuiltinFunction> {
            let mut context = EvalContext::new(BuiltinMap::new(
                builtin::std::builtin_values(),
                builtin::std::builtin_functions(),
                builtin::std::builtin_units(),
                builtin::std::builtin_prefixes(),
            ));

            let model_path = PathBuf::from("test");
            context.set_active_model(model_path);

            for (name, value, units) in previous_parameters {
                let parameter = build_simple_parameter(name, value, units);
                let parameter_value =
                    eval_parameter(&parameter, &context).expect("eval should succeed");
                let parameter_result = build_parameter_result(name, parameter_value);
                context.add_parameter_result(name.to_string(), Ok(parameter_result));
            }

            context
        }

        pub fn build_parameter_result(name: &str, value: Value) -> eval_result::Parameter {
            eval_result::Parameter {
                value,
                ident: name.to_string(),
                label: name.to_string(),
                print_level: eval_result::PrintLevel::None,
                debug_info: None,
            }
        }
    }
}
