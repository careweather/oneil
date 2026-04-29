use indexmap::{IndexMap, IndexSet};

use oneil_ir as ir;
use oneil_shared::{
    span::Span,
    symbols::{BuiltinValueName, ParameterName, ReferenceName},
};

use oneil_output::{
    self as output, BuiltinDependency, DependencySet, EvalError, EvalWarning, ExternalDependency,
    MeasuredNumber, Number, ParameterDependency, Unit, Value,
};

use crate::{
    context::{EvalContext, ExternalEvaluationContext},
    eval_expr, eval_unit,
};

pub struct EvalParameterResult {
    pub value: Value,
    pub expr_span: Span,
    pub warnings: Vec<EvalWarning>,
}

/// Evaluates a parameter and returns the resulting value.
///
/// # Errors
///
/// Returns an error if:
/// - The parameter value is invalid.
/// - The parameter value does not match the given unit, if there is one.
/// - The parameter value is outside the limits.
/// - The parameter unit does not match the limit.
pub fn eval_parameter<E: ExternalEvaluationContext>(
    parameter_name: ParameterName,
    parameter: &ir::Parameter,
    context: &mut EvalContext<'_, E>,
) -> Result<EvalParameterResult, Vec<EvalError>> {
    // Overlay RHSes have already been applied to `parameter.value()` by
    // the design composition step, and any anchor-scope handling is
    // expressed through [`ir::DesignProvenance::anchor_path`] which the
    // caller has already pushed onto `context`'s scope stack. Eval
    // therefore just runs the parameter's value as-is — no overlay
    // lookup needed here.
    eval_parameter_from_resolved_value(parameter_name, parameter.value(), parameter, context)
}

/// Evaluates a parameter using an explicit resolved [`ir::ParameterValue`] (IR default or overlay).
pub fn eval_parameter_from_resolved_value<E: ExternalEvaluationContext>(
    parameter_name: ParameterName,
    value_source: &ir::ParameterValue,
    parameter: &ir::Parameter,
    context: &mut EvalContext<'_, E>,
) -> Result<EvalParameterResult, Vec<EvalError>> {
    context.begin_parameter_evaluation(parameter_name);

    // evaluate the value and the unit
    let (value, expr_span, unit_ir) = match value_source {
        ir::ParameterValue::Simple(expr, unit) => {
            let (value, expr_span) = eval_expr::eval_expr(expr, context)?;
            (value, expr_span, unit)
        }
        ir::ParameterValue::Piecewise(piecewise, unit) => {
            let param_ident = parameter.name().clone();
            let param_ident_span = parameter.name_span().clone();
            let (value, expr_span) =
                get_piecewise_result(piecewise, param_ident, param_ident_span, context)?;
            (value, expr_span, unit)
        }
    };

    let unit = unit_ir
        .as_ref()
        .map(|unit_ir| eval_unit::eval_unit(unit_ir, context));

    // typecheck the value against the unit
    let value = match (value, unit) {
        (Value::Boolean(value), None) => Value::Boolean(value),
        (Value::String(value), None) => Value::String(value),
        (Value::Boolean(_), Some((_, unit_span))) => {
            return Err(vec![EvalError::BooleanCannotHaveUnit {
                expr_span: expr_span.clone(),
                unit_span,
            }]);
        }
        (Value::String(_), Some((_, unit_span))) => {
            return Err(vec![EvalError::StringCannotHaveUnit {
                expr_span: expr_span.clone(),
                unit_span,
            }]);
        }
        (Value::Number(value), None) => Value::Number(value),
        (Value::Number(number), Some((unit, _unit_span))) => {
            let number = MeasuredNumber::from_number_and_unit(number, unit);
            Value::MeasuredNumber(number)
        }
        (Value::MeasuredNumber(number), None) if number.is_dimensionless() => {
            Value::MeasuredNumber(number.with_unit(Unit::one()))
        }
        (Value::MeasuredNumber(number), None) => {
            return Err(vec![EvalError::ParameterMissingUnitAnnotation {
                param_expr_span: expr_span.clone(),
                param_value_unit: number.unit().display_unit.clone(),
                is_dimensionless: number.unit().is_dimensionless(),
            }]);
        }
        (Value::MeasuredNumber(number), Some((unit, unit_span)))
            if !number.unit().dimensionally_eq(&unit) =>
        {
            return Err(vec![EvalError::ParameterUnitMismatch {
                param_expr_span: expr_span.clone(),
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

    let warnings = context.take_expression_warnings();

    Ok(EvalParameterResult {
        value,
        expr_span: expr_span.clone(),
        warnings,
    })
}

fn get_piecewise_result<'a, E: ExternalEvaluationContext>(
    piecewise: &'a [ir::PiecewiseExpr],
    param_ident: ParameterName,
    param_ident_span: Span,
    context: &mut EvalContext<'_, E>,
) -> Result<(Value, &'a Span), Vec<EvalError>> {
    // evaluate each of the conditions and their bodies
    let results = piecewise.iter().map(|piecewise_expr| {
        let (if_result, if_expr_span) = eval_expr::eval_expr(piecewise_expr.if_expr(), context)?;
        let (branch_result, branch_expr_span) =
            eval_expr::eval_expr(piecewise_expr.expr(), context)?;

        match if_result {
            Value::Boolean(true) => Ok(Some((branch_result, branch_expr_span, if_expr_span))),
            Value::Boolean(false) => Ok(None),
            Value::String(_) | Value::Number(_) | Value::MeasuredNumber(_) => {
                Err(vec![EvalError::InvalidIfExpressionType {
                    expr_span: if_expr_span.clone(),
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
            .map(|(_, _, if_expr_span)| if_expr_span.clone())
            .collect();

        return Err(vec![EvalError::MultiplePiecewiseBranchesMatch {
            param_ident,
            param_ident_span,
            matching_branche_spans,
        }]);
    }

    // finally, return the matching branch result and expression span
    // or an error if there are no matching branches
    let Some((matching_branch_result, matching_branch_expr_span, _)) = matching_branches.pop()
    else {
        return Err(vec![EvalError::NoPiecewiseBranchMatch {
            param_ident,
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

fn eval_limits<E: ExternalEvaluationContext>(
    limits: &ir::Limits,
    context: &mut EvalContext<'_, E>,
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

fn eval_continuous_limits<E: ExternalEvaluationContext>(
    min: &oneil_ir::Expr,
    max: &oneil_ir::Expr,
    limit_expr_span: &Span,
    context: &mut EvalContext<'_, E>,
) -> Result<Limits, Vec<EvalError>> {
    let min = eval_expr::eval_expr(min, context).and_then(|(value, expr_span)| match value {
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.into_number_and_unit();
            Ok((number, expr_span.clone(), Some(unit)))
        }
        Value::Number(number) => Ok((number, expr_span.clone(), None)),
        Value::Boolean(_) | Value::String(_) => {
            Err(vec![EvalError::InvalidContinuousLimitMinType {
                expr_span: expr_span.clone(),
                found_value: value,
            }])
        }
    });

    let max = eval_expr::eval_expr(max, context).and_then(|(value, expr_span)| match value {
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.into_number_and_unit();
            Ok((number, expr_span.clone(), Some(unit)))
        }
        Value::Number(number) => Ok((number, expr_span.clone(), None)),
        Value::Boolean(_) | Value::String(_) => {
            Err(vec![EvalError::InvalidContinuousLimitMaxType {
                expr_span: expr_span.clone(),
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
        limit_expr_span: limit_expr_span.clone(),
    })
}

#[expect(
    clippy::panic_in_result_fn,
    reason = "enforcing an invariant that should always hold"
)]
fn eval_discrete_limits<E: ExternalEvaluationContext>(
    values: &[ir::Expr],
    limit_expr_span: &Span,
    context: &mut EvalContext<'_, E>,
) -> Result<Limits, Vec<EvalError>> {
    let mut errors = Vec::new();
    let mut results: Vec<(Value, &Span)> = Vec::new();

    for value in values {
        match eval_expr::eval_expr(value, context) {
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
                Some((limit_unit, first_expr_span.clone())),
                results,
                limit_expr_span,
            )
        }
        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotBeDiscreteLimitValue {
            expr_span: first_expr_span.clone(),
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
    seen_strings.insert(&first_value, first_expr_span.clone());

    for (value, expr_span) in &results {
        match value {
            Value::String(string) => {
                if let Some(original_expr_span) = seen_strings.get(string) {
                    errors.push(EvalError::DuplicateStringLimit {
                        expr_span: (*expr_span).clone(),
                        original_expr_span: original_expr_span.clone(),
                        string_value: string.clone(),
                    });
                } else {
                    string_values.push(string);
                    seen_strings.insert(string, (*expr_span).clone());
                }
            }
            Value::Number(_) | Value::MeasuredNumber(_) | Value::Boolean(_) => {
                errors.push(EvalError::ExpectedStringLimit {
                    expr_span: (*expr_span).clone(),
                    found_value: value.clone(),
                });
            }
        }
    }

    if errors.is_empty() {
        let strings = string_values.into_iter().cloned().collect();
        Ok(Limits::StringDiscrete {
            values: strings,
            limit_expr_span: limit_expr_span.clone(),
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
                            limit_span: limit_expr_span.clone(),
                            value_unit: number_result_unit.display_unit.clone(),
                            value_unit_span: expr_span.clone(),
                        });
                    }
                    None => {
                        limit_unit = Some((number_result_unit, expr_span.clone()));
                        numbers.push(number_result);
                    }
                }
            }
            Value::Number(number_result) => {
                numbers.push(number_result);
            }
            Value::Boolean(_) | Value::String(_) => {
                errors.push(EvalError::ExpectedNumberLimit {
                    expr_span: expr_span.clone(),
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
            limit_expr_span: limit_expr_span.clone(),
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
                param_expr_span: param_expr_span.clone(),
                param_value: value.clone(),
            }])
        }
        Value::Number(number) if number.min() < 0.0 => {
            Err(vec![EvalError::ParameterValueBelowDefaultLimits {
                param_expr_span: param_expr_span.clone(),
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
            expr_span: param_expr_span.clone(),
            limit_span: limit_expr_span,
        }]),
        Value::String(_) => Err(vec![EvalError::StringCannotHaveNumberLimit {
            param_expr_span: param_expr_span.clone(),
            param_value: value.clone(),
            limit_span: limit_expr_span,
        }]),
        Value::Number(number) => {
            if let Some(limit_unit) = unit {
                Err(vec![EvalError::UnitlessNumberCannotHaveLimitWithUnit {
                    param_expr_span: param_expr_span.clone(),
                    param_value: value.clone(),
                    limit_span: limit_expr_span,
                    limit_unit: limit_unit.display_unit,
                }])
            } else if number.min() < min.min() {
                Err(vec![EvalError::ParameterValueBelowContinuousLimits {
                    param_expr_span: param_expr_span.clone(),
                    param_value: value.clone(),
                    min_expr_span,
                    min_value: Value::Number(min),
                }])
            } else if number.max() > max.max() {
                Err(vec![EvalError::ParameterValueAboveContinuousLimits {
                    param_expr_span: param_expr_span.clone(),
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
                    param_expr_span: param_expr_span.clone(),
                    param_value: value.clone(),
                    min_expr_span,
                    min_value: Value::Number(min),
                }])
            } else if number.normalized_value().max() > adjusted_max.normalized_value().max() {
                Err(vec![EvalError::ParameterValueAboveContinuousLimits {
                    param_expr_span: param_expr_span.clone(),
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
                    param_expr_span: param_expr_span.clone(),
                    param_value: value.clone(),
                    limit_expr_span,
                    limit_values: values,
                }])
            }
        }

        Value::Number(number) => {
            if let Some(limit_unit) = unit {
                return Err(vec![EvalError::UnitlessNumberCannotHaveLimitWithUnit {
                    param_expr_span: param_expr_span.clone(),
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
                    param_expr_span: param_expr_span.clone(),
                    param_value: value.clone(),
                    limit_expr_span,
                    limit_values: values,
                }])
            }
        }

        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotHaveALimit {
            expr_span: param_expr_span.clone(),
            limit_span: limit_expr_span,
        }]),

        Value::String(_) => Err(vec![EvalError::StringCannotHaveNumberLimit {
            param_expr_span: param_expr_span.clone(),
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
                param_expr_span: param_expr_span.clone(),
                param_value: value.clone(),
                limit_expr_span,
                limit_values: values,
            }])
        }
        Value::String(_) => Ok(()),
        Value::Boolean(_) => Err(vec![EvalError::BooleanCannotHaveALimit {
            expr_span: param_expr_span.clone(),
            limit_span: limit_expr_span,
        }]),
        Value::Number(_) | Value::MeasuredNumber(_) => {
            Err(vec![EvalError::NumberCannotHaveStringLimit {
                param_expr_span: param_expr_span.clone(),
                param_value: value.clone(),
                limit_span: limit_expr_span,
            }])
        }
    }
}

/// Builds an [`output::Parameter`] from a successfully evaluated value plus the IR metadata.
///
/// Handles trace-level gating and (for Debug levels) collects current dependency values via
/// `get_*_dependency_values` helpers.
pub fn build_output_parameter<E: ExternalEvaluationContext>(
    value: Value,
    expr_span: Span,
    warnings: Vec<EvalWarning>,
    parameter: &ir::Parameter,
    context: &mut EvalContext<'_, E>,
) -> output::Parameter {
    let (print_level, debug_info) = match parameter.trace_level() {
        ir::TraceLevel::Debug if parameter.is_performance() => {
            let builtin_dependency_values =
                get_builtin_dependency_values(parameter.dependencies().builtin(), context);
            let parameter_dependency_values =
                get_parameter_dependency_values(parameter.dependencies().parameter(), context);
            let external_dependency_values =
                get_external_dependency_values(parameter.dependencies().external(), context);
            (
                output::PrintLevel::Performance,
                Some(output::DebugInfo {
                    builtin_dependency_values,
                    parameter_dependency_values,
                    external_dependency_values,
                }),
            )
        }
        ir::TraceLevel::Trace | ir::TraceLevel::None if parameter.is_performance() => {
            (output::PrintLevel::Performance, None)
        }
        ir::TraceLevel::Debug => {
            let builtin_dependency_values =
                get_builtin_dependency_values(parameter.dependencies().builtin(), context);
            let parameter_dependency_values =
                get_parameter_dependency_values(parameter.dependencies().parameter(), context);
            let external_dependency_values =
                get_external_dependency_values(parameter.dependencies().external(), context);
            (
                output::PrintLevel::Trace,
                Some(output::DebugInfo {
                    builtin_dependency_values,
                    parameter_dependency_values,
                    external_dependency_values,
                }),
            )
        }
        ir::TraceLevel::Trace => (output::PrintLevel::Trace, None),
        ir::TraceLevel::None => (output::PrintLevel::None, None),
    };

    let builtin_dependencies = parameter
        .dependencies()
        .builtin()
        .keys()
        .map(|builtin_name| BuiltinDependency {
            name: builtin_name.clone(),
        })
        .collect::<IndexSet<_>>();

    let parameter_dependencies = parameter
        .dependencies()
        .parameter()
        .keys()
        .map(|parameter_name| ParameterDependency {
            parameter_name: parameter_name.clone(),
        })
        .collect::<IndexSet<_>>();

    let external_dependencies = parameter
        .dependencies()
        .external()
        .keys()
        .filter_map(|(reference_name, parameter_name)| {
            // Model path is looked up from the live eval context since it is no
            // longer stored in `ir::Variable::External`.
            let model_path = context.lookup_external_model_path(reference_name)?;
            Some(ExternalDependency {
                model_path,
                reference_name: reference_name.clone(),
                parameter_name: parameter_name.clone(),
            })
        })
        .collect::<IndexSet<_>>();

    let dependencies = DependencySet {
        builtin_dependencies,
        parameter_dependencies,
        external_dependencies,
    };

    output::Parameter {
        ident: parameter.name().clone(),
        label: parameter.label().clone(),
        value,
        print_level,
        debug_info,
        dependencies,
        expr_span,
        warnings,
    }
}

/// Looks up current values of builtin dependencies for debug reporting.
pub fn get_builtin_dependency_values<E: ExternalEvaluationContext>(
    dependencies: &IndexMap<BuiltinValueName, Span>,
    context: &EvalContext<'_, E>,
) -> IndexMap<BuiltinValueName, Value> {
    dependencies
        .keys()
        .map(|dependency| {
            let value = context.lookup_builtin_variable(dependency);
            (dependency.clone(), value)
        })
        .collect::<IndexMap<_, _>>()
}

/// Looks up current values of parameter dependencies for debug reporting.
///
/// Must only be called after the referenced parameters have been evaluated; the lazy memo
/// table must already contain `Done` slots for them. If a dependency is still unevaluated,
/// `force_parameter` will evaluate it now.
///
/// # Panics
///
/// Panics if any dependency is not defined in scope (a resolver invariant violation).
pub fn get_parameter_dependency_values<E: ExternalEvaluationContext>(
    dependencies: &IndexMap<ParameterName, Span>,
    context: &mut EvalContext<'_, E>,
) -> IndexMap<ParameterName, Value> {
    let mut out = IndexMap::new();
    for (dependency, dependency_span) in dependencies {
        let value = context
            .lookup_parameter_value(dependency, dependency_span.clone())
            .expect("dependency should be found because the expression evaluated successfully");
        out.insert(dependency.clone(), value);
    }
    out
}

/// Looks up current values of external (cross-reference) dependencies for debug reporting.
///
/// # Panics
///
/// Panics if any dependency is not defined in scope.
pub fn get_external_dependency_values<E: ExternalEvaluationContext>(
    dependencies: &IndexMap<(ReferenceName, ParameterName), Span>,
    context: &mut EvalContext<'_, E>,
) -> IndexMap<(ReferenceName, ParameterName), Value> {
    let mut out = IndexMap::new();
    for ((reference_name, parameter_name), dependency_span) in dependencies {
        let value = context
            .lookup_external_parameter_value(
                reference_name,
                parameter_name,
                dependency_span.clone(),
            )
            .expect("dependency should be found because the expression evaluated successfully");
        out.insert((reference_name.clone(), parameter_name.clone()), value);
    }
    out
}

#[cfg(test)]
mod tests {
    use oneil_output::Dimension;
    use oneil_shared::EvalInstanceKey;

    use crate::{
        assert_is_close, assert_units_dimensionally_eq,
        context::EvalContext,
        test_context::{TestExternalContext, test_model_path},
    };

    use super::*;

    #[test]
    fn eval_no_unit() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, []);
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        // check the parameter value
        let Value::Number(number) = parameter_value.value else {
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
        let parameter = helper::build_simple_parameter(
            "x",
            1.0,
            [helper::UnitSpec::new(Some("m"), None, false, 1.0)],
        );
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let parameter = helper::build_simple_parameter(
            "x",
            1.0,
            [helper::UnitSpec::new(Some("m"), Some("k"), false, 1.0)],
        );
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let parameter = helper::build_simple_parameter(
            "x",
            1.0,
            [
                helper::UnitSpec::new(Some("m"), Some("k"), false, 1.0),
                helper::UnitSpec::new(Some("hr"), None, false, -1.0),
            ],
        );
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0), (Dimension::Time, -1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let parameter = helper::build_simple_parameter(
            "x",
            1.0,
            [helper::UnitSpec::new(None, None, true, 1.0)],
        );
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let parameter = helper::build_simple_parameter(
            "x",
            1.0,
            [helper::UnitSpec::new(Some("W"), None, true, 1.0)],
        );
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    1.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
                (
                    "y",
                    1.0,
                    vec![helper::UnitSpec::new(Some("m"), Some("k"), false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x + y with unit km
        let parameter = helper::build_add_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("m"), Some("k"), false, 1.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    1.0,
                    vec![
                        helper::UnitSpec::new(Some("g"), Some("k"), false, 1.0),
                        helper::UnitSpec::new(Some("m"), None, false, 1.0),
                        helper::UnitSpec::new(Some("s"), None, false, -2.0),
                    ],
                ),
                (
                    "y",
                    1.0,
                    vec![helper::UnitSpec::new(Some("N"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x + y with unit N
        let parameter = helper::build_add_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("N"), None, false, 1.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 1.0),
            (Dimension::Time, -2.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    1.0,
                    vec![helper::UnitSpec::new(Some("W"), None, true, 1.0)],
                ),
                (
                    "y",
                    1.0,
                    vec![helper::UnitSpec::new(Some("W"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x + y with unit W
        let parameter = helper::build_add_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("W"), None, false, 1.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [(
                "x",
                1.0,
                vec![helper::UnitSpec::new(Some("W"), None, false, 1.0)],
            )],
        );

        // setup parameter y = x^2 with unit W^2
        let parameter = helper::build_exponent_parameter(
            "y",
            "x",
            2.0,
            [helper::UnitSpec::new(Some("W"), None, false, 2.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [
            (Dimension::Mass, 2.0),
            (Dimension::Distance, 4.0),
            (Dimension::Time, -6.0),
        ];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    3.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
                (
                    "y",
                    2.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x * y with unit m^2
        let parameter = helper::build_mul_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("m"), None, false, 2.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 2.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    6.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 2.0)],
                ),
                (
                    "y",
                    2.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x / y with unit m
        let parameter = helper::build_div_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("m"), None, false, 1.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    6.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
                (
                    "y",
                    2.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x // y with unit m
        // Escaped division requires matching units
        let parameter = helper::build_escaped_div_parameter(
            "z",
            "x",
            "y",
            [
                helper::UnitSpec::new(Some("m"), None, false, 1.0),
                helper::UnitSpec::new(Some("m"), None, false, -1.0),
            ],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    6.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
                (
                    "y",
                    2.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x -- y with unit m
        // Escaped subtraction requires matching units
        let parameter = helper::build_escaped_sub_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("m"), None, false, 1.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        helper::setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    7.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
                (
                    "y",
                    3.0,
                    vec![helper::UnitSpec::new(Some("m"), None, false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x % y with unit m
        let parameter = helper::build_mod_parameter(
            "z",
            "x",
            "y",
            [helper::UnitSpec::new(Some("m"), None, false, 1.0)],
        );

        let parameter_value = eval_parameter(parameter.name().clone(), &parameter, &mut context)
            .expect("eval should succeed");

        let expected_dimensions = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::MeasuredNumber(number) = parameter_value.value else {
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

    mod helper {
        use super::*;

        use crate::context::EvalContext;
        use crate::test_context::TestExternalContext;
        use oneil_output as output;

        use oneil_ir::DisplayCompositeUnit;
        use oneil_shared::labels::ParameterLabel;

        use oneil_shared::span::Span;
        use oneil_shared::symbols::{ParameterName, UnitBaseName, UnitName, UnitPrefix};

        /// Returns a dummy span for use in test parameters.
        ///
        /// This function creates a span with all fields set to zero.
        /// It is not intended to be directly tested, but rather used
        /// as a placeholder when constructing IR nodes for testing.
        fn random_span() -> Span {
            Span::synthetic()
        }

        /// Returns a display composite unit that isn't intended to be tested.
        fn unimportant_display_composite_unit() -> DisplayCompositeUnit {
            DisplayCompositeUnit::BaseUnit(ir::DisplayUnit::new("unimportant".to_string(), 1.0))
        }

        /// Specification for a unit in tests.
        #[derive(Debug, Clone, Copy)]
        pub struct UnitSpec {
            /// The base unit name (e.g., "m", "s", "W"). Use `None` for pure dB.
            pub base_name: Option<&'static str>,
            /// Optional SI prefix (e.g., "k" for kilo, "m" for milli).
            pub prefix: Option<&'static str>,
            /// Whether this is a decibel unit.
            pub is_db: bool,
            /// The exponent of the unit.
            pub exponent: f64,
        }

        impl UnitSpec {
            pub const fn new(
                base_name: Option<&'static str>,
                prefix: Option<&'static str>,
                is_db: bool,
                exponent: f64,
            ) -> Self {
                Self {
                    base_name,
                    prefix,
                    is_db,
                    exponent,
                }
            }
        }

        fn build_full_name(base_name: Option<&str>, prefix: Option<&str>, is_db: bool) -> UnitName {
            UnitName::new(format!(
                "{}{}{}",
                if is_db { "dB" } else { "" },
                prefix.unwrap_or(""),
                base_name.unwrap_or("")
            ))
        }

        fn build_unit_info(
            base_name: Option<&str>,
            prefix: Option<&str>,
            is_db: bool,
        ) -> ir::UnitInfo {
            if is_db {
                ir::UnitInfo::Db {
                    prefix: prefix.map(|s| UnitPrefix::new(s.to_string())),
                    base_name: base_name.map(|s| UnitBaseName::new(s.to_string())),
                }
            } else {
                ir::UnitInfo::Standard {
                    prefix: prefix.map(|s| UnitPrefix::new(s.to_string())),
                    base_name: UnitBaseName::new(
                        base_name.expect("base name should be provided").to_string(),
                    ),
                }
            }
        }

        fn build_resolved_units(
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> Option<ir::CompositeUnit> {
            let units: Vec<_> = units
                .into_iter()
                .map(|spec| {
                    let full_name = build_full_name(spec.base_name, spec.prefix, spec.is_db);
                    let info = build_unit_info(spec.base_name, spec.prefix, spec.is_db);
                    ir::Unit::new(
                        random_span(),
                        full_name,
                        random_span(),
                        spec.exponent,
                        None,
                        info,
                    )
                })
                .collect();

            if units.is_empty() {
                None
            } else {
                Some(ir::CompositeUnit::new(
                    units,
                    unimportant_display_composite_unit(),
                    random_span(),
                    oneil_output::DimensionMap::dimensionless(),
                ))
            }
        }

        /// Builds a simple parameter with a literal numeric value.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value` - The numeric value of the parameter
        /// * `units` - An iterator of `UnitSpec` values
        ///
        /// # Returns
        ///
        /// A parameter with a literal number expression and the specified units.
        pub fn build_simple_parameter(
            name: &str,
            value: f64,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr = ir::Expr::Literal {
                span: random_span(),
                value: ir::Literal::Number(value),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
            )
        }

        /// Builds a parameter with an addition expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the first parameter to add
        /// * `value_b` - The name of the second parameter to add
        /// * `units` - An iterator of `UnitSpec` values
        ///
        /// # Returns
        ///
        /// A parameter with an addition binary operation: `value_a + value_b`.
        pub fn build_add_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_a), random_span()),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_b), random_span()),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Add,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
            )
        }

        /// Builds a parameter with a multiplication expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the first parameter to multiply
        /// * `value_b` - The name of the second parameter to multiply
        /// * `units` - An iterator of `UnitSpec` values
        ///
        /// # Returns
        ///
        /// A parameter with a multiplication binary operation: `value_a * value_b`.
        pub fn build_mul_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_a), random_span()),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_b), random_span()),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Mul,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
            )
        }

        /// Builds a parameter with a division expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the dividend parameter
        /// * `value_b` - The name of the divisor parameter
        /// * `units` - An iterator of `UnitSpec` values
        ///
        /// # Returns
        ///
        /// A parameter with a division binary operation: `value_a / value_b`.
        pub fn build_div_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_a), random_span()),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_b), random_span()),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Div,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
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
        /// * `units` - An iterator of unit specs (must match the units of both operands)
        ///
        /// # Returns
        ///
        /// A parameter with an escaped division binary operation: `value_a // value_b`.
        pub fn build_escaped_div_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_a), random_span()),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_b), random_span()),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::EscapedDiv,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
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
        /// * `units` - An iterator of unit specs (must match the units of both operands)
        ///
        /// # Returns
        ///
        /// A parameter with an escaped subtraction binary operation: `value_a -- value_b`.
        pub fn build_escaped_sub_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_a), random_span()),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_b), random_span()),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::EscapedSub,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
            )
        }

        /// Builds a parameter with a modulo expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `value_a` - The name of the dividend parameter
        /// * `value_b` - The name of the divisor parameter
        /// * `units` - An iterator of `UnitSpec` values
        ///
        /// # Returns
        ///
        /// A parameter with a modulo binary operation: `value_a % value_b`.
        pub fn build_mod_parameter(
            name: &str,
            value_a: &str,
            value_b: &str,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_a = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_a), random_span()),
            };

            let expr_b = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(value_b), random_span()),
            };

            let expr = ir::Expr::BinaryOp {
                span: random_span(),
                op: ir::BinaryOp::Mod,
                left: Box::new(expr_a),
                right: Box::new(expr_b),
            };

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
            )
        }

        /// Builds a parameter with an exponentiation expression.
        ///
        /// # Arguments
        ///
        /// * `name` - The name of the parameter
        /// * `base` - The name of the base parameter
        /// * `exponent` - The exponent value (a literal number)
        /// * `units` - An iterator of `UnitSpec` values
        ///
        /// # Returns
        ///
        /// A parameter with an exponentiation binary operation: `base ^ exponent`.
        pub fn build_exponent_parameter(
            name: &str,
            base: &str,
            exponent: f64,
            units: impl IntoIterator<Item = UnitSpec>,
        ) -> ir::Parameter {
            let expr_base = ir::Expr::Variable {
                span: random_span(),
                variable: ir::Variable::parameter(ParameterName::from(base), random_span()),
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

            let units = build_resolved_units(units);

            ir::Parameter::new(
                ir::Dependencies::new(),
                ParameterName::from(name),
                random_span(),
                random_span(),
                ParameterLabel::from(name),
                None,
                ir::ParameterValue::simple(expr, units),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
                None,
            )
        }

        /// Loads pre-defined parameters into an existing evaluation context.
        ///
        /// The context must already have an active model pushed (e.g. via
        /// `context.push_active_model(EvalInstanceKey::root(test_model_path("test")))`).
        ///
        /// # Arguments
        ///
        /// * `context` - The evaluation context to add parameter results to
        /// * `previous_parameters` - An iterator of tuples containing:
        ///   - Parameter name
        ///   - Parameter value (a literal number)
        ///   - Units as a vector of unit specs
        pub fn setup_context_with_parameters(
            context: &mut EvalContext<'_, TestExternalContext>,
            previous_parameters: impl IntoIterator<Item = (&'static str, f64, Vec<UnitSpec>)>,
        ) {
            for (name, value, units) in previous_parameters {
                let parameter = build_simple_parameter(name, value, units);

                let parameter_value = eval_parameter(parameter.name().clone(), &parameter, context)
                    .expect("eval should succeed");
                let parameter_result = build_parameter_result(name, parameter_value.value);
                context.add_parameter_result(ParameterName::from(name), Ok(parameter_result));
            }
        }

        pub fn build_parameter_result(name: &str, value: Value) -> output::Parameter {
            output::Parameter {
                value,
                ident: ParameterName::from(name),
                label: ParameterLabel::from(name),
                print_level: output::PrintLevel::None,
                debug_info: None,
                dependencies: output::DependencySet::default(),
                expr_span: random_span(),
                warnings: Vec::new(),
            }
        }
    }
}
