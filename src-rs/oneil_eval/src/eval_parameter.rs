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

#[derive(Debug)]
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
    parameter: &ir::Parameter,
    context: &mut EvalContext<'_, E>,
) -> Result<EvalParameterResult, Vec<EvalError>> {
    // Overlay RHSes have already been applied to `parameter.value()` by
    // the design composition step, and any anchor-scope handling is
    // expressed through [`ir::DesignProvenance::anchor_path`] which the
    // caller has already pushed onto `context`'s scope stack. Eval
    // therefore just runs the parameter's value as-is — no overlay
    // lookup needed here.
    eval_parameter_from_resolved_value(parameter.value(), parameter, context)
}

/// Evaluates a parameter using an explicit resolved [`ir::ParameterValue`] (IR default or overlay).
pub fn eval_parameter_from_resolved_value<E: ExternalEvaluationContext>(
    value_source: &ir::ParameterValue,
    parameter: &ir::Parameter,
    context: &mut EvalContext<'_, E>,
) -> Result<EvalParameterResult, Vec<EvalError>> {
    context.begin_expression_evaluation();

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
        let matching_branch_spans = matching_branches
            .into_iter()
            .map(|(_, _, if_expr_span)| if_expr_span.clone())
            .collect();

        return Err(vec![EvalError::MultiplePiecewiseBranchesMatch {
            param_ident,
            param_ident_span,
            matching_branch_spans,
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
            // Instance key is looked up from the live eval context since it is no
            // longer stored in `ir::Variable::External`.
            let instance_key = context.lookup_external_instance_key(reference_name)?;
            Some(ExternalDependency {
                instance_key,
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
    use oneil_ir::{
        self as ir,
        test_helpers::{
            expr::{lit_bool, lit_number, lit_string, param_var},
            parameter::{
                build_binary_parameter, build_exponent_parameter, build_literal_parameter,
                build_parameter_from_expr, build_piecewise_parameter, build_simple_parameter,
                continuous_limits, discrete_limits,
            },
            unit::{UnitSpec, build_resolved_units},
        },
    };
    use oneil_output::{Dimension, EvalError, Number, Value, util::is_close};
    use oneil_shared::EvalInstanceKey;

    use crate::{
        check_is_close, check_param_measured_scalar, check_param_scalar_close,
        context::EvalContext,
        test_context::{TestExternalContext, test_model_path},
        test_fixtures::{eval_parameter_simple, setup_context_with_parameters},
    };

    use super::*;

    #[test]
    #[expect(clippy::type_complexity)]
    fn eval_simple_parameters_table() {
        let cases: &[(
            &str,
            f64,
            &[UnitSpec],
            Option<(f64, &[(Dimension, f64)], f64, bool)>,
        )] = &[
            ("no_unit", 1.0, &[], None),
            (
                "m",
                1.0,
                &[UnitSpec::new(None, Some("m"), false, 1.0)],
                Some((1.0, &[(Dimension::Distance, 1.0)], 1.0, false)),
            ),
            (
                "km",
                1.0,
                &[UnitSpec::new(Some("k"), Some("m"), false, 1.0)],
                Some((1000.0, &[(Dimension::Distance, 1.0)], 1000.0, false)),
            ),
            (
                "km_per_hr",
                1.0,
                &[
                    UnitSpec::new(Some("k"), Some("m"), false, 1.0),
                    UnitSpec::new(None, Some("hr"), false, -1.0),
                ],
                Some((
                    1000.0 / 3600.0,
                    &[(Dimension::Distance, 1.0), (Dimension::Time, -1.0)],
                    1000.0 / 3600.0,
                    false,
                )),
            ),
            (
                "db",
                1.0,
                &[UnitSpec::new(None, None, true, 1.0)],
                Some((10.0_f64.powf(0.1), &[], 1.0, true)),
            ),
            (
                "dbw",
                1.0,
                &[UnitSpec::new(None, Some("W"), true, 1.0)],
                Some((
                    10.0_f64.powf(0.1),
                    &[
                        (Dimension::Mass, 1.0),
                        (Dimension::Distance, 2.0),
                        (Dimension::Time, -3.0),
                    ],
                    1.0,
                    true,
                )),
            ),
        ];

        for (name, value, units, measured) in cases {
            let parameter = build_simple_parameter("x", *value, units.iter().copied());
            let result = eval_parameter_simple(&parameter).unwrap_or_else(|errors| {
                panic!("{name}: eval should succeed, got {errors:?}");
            });
            match measured {
                None => check_param_scalar_close(&result, *value).assert_with_name(name),
                Some((normalized, dims, magnitude, is_db)) => {
                    check_param_measured_scalar(&result, *normalized, dims, *magnitude, *is_db)
                        .assert_with_name(name);
                }
            }
        }
    }

    #[test]
    fn eval_add_parameters_with_different_units() {
        // setup context with x = 1.0 m and y = 1.0 km
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 1.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
                (
                    "y",
                    1.0,
                    vec![UnitSpec::new(Some("k"), Some("m"), false, 1.0)],
                ),
            ],
        );

        // setup parameter z = x + y with unit km
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::Add,
            "x",
            "y",
            [UnitSpec::new(Some("k"), Some("m"), false, 1.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // x + y = 1.0 m + 1000.0 m = 1001.0 m
        // The value is stored in base units (meters)
        check_param_measured_scalar(
            &parameter_value,
            1001.0,
            &[(Dimension::Distance, 1.0)],
            1000.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_add_parameters_kg_m_per_s2_and_n() {
        // setup context with x = 1.0 kg*m/s^2 and y = 1.0 N
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                (
                    "x",
                    1.0,
                    vec![
                        UnitSpec::new(Some("k"), Some("g"), false, 1.0),
                        UnitSpec::new(None, Some("m"), false, 1.0),
                        UnitSpec::new(None, Some("s"), false, -2.0),
                    ],
                ),
                ("y", 1.0, vec![UnitSpec::new(None, Some("N"), false, 1.0)]),
            ],
        );

        // setup parameter z = x + y with unit N
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::Add,
            "x",
            "y",
            [UnitSpec::new(None, Some("N"), false, 1.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // x + y = 1.0 N + 1.0 N = 2.0 N
        // The value is stored in base units
        check_param_measured_scalar(
            &parameter_value,
            2.0,
            &[
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 1.0),
                (Dimension::Time, -2.0),
            ],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_add_parameters_dbw_and_w() {
        // setup context with x = 1.0 dBW and y = 1.0 W
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 1.0, vec![UnitSpec::new(None, Some("W"), true, 1.0)]),
                ("y", 1.0, vec![UnitSpec::new(None, Some("W"), false, 1.0)]),
            ],
        );

        // setup parameter z = x + y with unit W
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::Add,
            "x",
            "y",
            [UnitSpec::new(None, Some("W"), false, 1.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // x = 1.0 dBW = 10^(1.0/10.0) = 10^0.1 = 1.258925... W
        // y = 1.0 W
        // x + y = 1.258925... W + 1.0 W = 2.258925... W
        check_param_measured_scalar(
            &parameter_value,
            10.0_f64.powf(0.1) + 1.0,
            &[
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
            ],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_exponent_parameter_w_squared() {
        // setup context with x = 1.0 W
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [("x", 1.0, vec![UnitSpec::new(None, Some("W"), false, 1.0)])],
        );

        // setup parameter y = x^2 with unit W^2
        let parameter =
            build_exponent_parameter("y", "x", 2.0, [UnitSpec::new(None, Some("W"), false, 2.0)]);

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // y = x^2 = (1.0 W)^2 = 1.0 W^2
        check_param_measured_scalar(
            &parameter_value,
            1.0,
            &[
                (Dimension::Mass, 2.0),
                (Dimension::Distance, 4.0),
                (Dimension::Time, -6.0),
            ],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_mul_function() {
        // setup context with x = 3.0 m and y = 2.0 m
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 3.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
                ("y", 2.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
            ],
        );

        // setup parameter z = x * y with unit m^2
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::Mul,
            "x",
            "y",
            [UnitSpec::new(None, Some("m"), false, 2.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // z = x * y = 3.0 m * 2.0 m = 6.0 m^2
        check_param_measured_scalar(
            &parameter_value,
            6.0,
            &[(Dimension::Distance, 2.0)],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_div_function() {
        // setup context with x = 6.0 m^2 and y = 2.0 m
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 6.0, vec![UnitSpec::new(None, Some("m"), false, 2.0)]),
                ("y", 2.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
            ],
        );

        // setup parameter z = x / y with unit m
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::Div,
            "x",
            "y",
            [UnitSpec::new(None, Some("m"), false, 1.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // z = x / y = 6.0 m^2 / 2.0 m = 3.0 m
        check_param_measured_scalar(
            &parameter_value,
            3.0,
            &[(Dimension::Distance, 1.0)],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_escaped_div_function() {
        // setup context with x = 6.0 m and y = 2.0 m
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 6.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
                ("y", 2.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
            ],
        );

        // setup parameter z = x // y with unit m
        // Escaped division requires matching units
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::EscapedDiv,
            "x",
            "y",
            [
                UnitSpec::new(None, Some("m"), false, 1.0),
                UnitSpec::new(None, Some("m"), false, -1.0),
            ],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // z = x // y = 6.0 m // 2.0 m = 3.0
        // For scalars, escaped division behaves the same as regular division
        check_param_measured_scalar(&parameter_value, 3.0, &[], 1.0, false).assert();
    }

    #[test]
    fn eval_escaped_sub_function() {
        // setup context with x = 6.0 m and y = 2.0 m
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 6.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
                ("y", 2.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
            ],
        );

        // setup parameter z = x -- y with unit m
        // Escaped subtraction requires matching units
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::EscapedSub,
            "x",
            "y",
            [UnitSpec::new(None, Some("m"), false, 1.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // z = x -- y = 6.0 m -- 2.0 m = 4.0 m
        // For scalars, escaped subtraction behaves the same as regular subtraction
        check_param_measured_scalar(
            &parameter_value,
            4.0,
            &[(Dimension::Distance, 1.0)],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_mod_function() {
        // setup context with x = 7.0 m and y = 3.0 m
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [
                ("x", 7.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
                ("y", 3.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)]),
            ],
        );

        // setup parameter z = x % y with unit m
        let parameter = build_binary_parameter(
            "z",
            ir::BinaryOp::Mod,
            "x",
            "y",
            [UnitSpec::new(None, Some("m"), false, 1.0)],
        );

        let parameter_value =
            eval_parameter(&parameter, &mut context).expect("eval should succeed");

        // z = x % y = 7.0 m % 3.0 m = 1.0 m
        check_param_measured_scalar(
            &parameter_value,
            1.0,
            &[(Dimension::Distance, 1.0)],
            1.0,
            false,
        )
        .assert();
    }

    #[test]
    fn eval_boolean_literal() {
        let parameter = build_literal_parameter(
            "flag",
            ir::Literal::boolean(true),
            [],
            ir::Limits::default(),
        );
        let result = eval_parameter_simple(&parameter).expect("eval should succeed");
        assert_eq!(result.value, Value::Boolean(true));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn eval_string_literal() {
        let parameter = build_literal_parameter(
            "name",
            ir::Literal::string("alpha".to_string()),
            [],
            ir::Limits::default(),
        );
        let result = eval_parameter_simple(&parameter).expect("eval should succeed");
        assert_eq!(result.value, Value::String("alpha".to_string()));
    }

    #[test]
    fn eval_boolean_cannot_have_unit() {
        let parameter = build_literal_parameter(
            "flag",
            ir::Literal::boolean(true),
            [UnitSpec::new(None, Some("m"), false, 1.0)],
            ir::Limits::default(),
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], EvalError::BooleanCannotHaveUnit { .. }),
            "expected BooleanCannotHaveUnit, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_string_cannot_have_unit() {
        let parameter = build_literal_parameter(
            "name",
            ir::Literal::string("alpha".to_string()),
            [UnitSpec::new(None, Some("m"), false, 1.0)],
            ir::Limits::default(),
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], EvalError::StringCannotHaveUnit { .. }),
            "expected StringCannotHaveUnit, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_rejects_negative_under_default_limits() {
        let parameter = build_simple_parameter("x", -1.0, []);
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::ParameterValueBelowDefaultLimits {
                    param_value: Value::Number(Number::Scalar(v)),
                    ..
                } if (*v - -1.0).abs() < f64::EPSILON
            ),
            "expected ParameterValueBelowDefaultLimits for -1, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_within_continuous_limits() {
        let parameter = build_literal_parameter(
            "x",
            ir::Literal::number(5.0),
            [],
            continuous_limits(lit_number(0.0), lit_number(10.0)),
        );
        let result = eval_parameter_simple(&parameter).expect("eval should succeed");
        let Value::Number(Number::Scalar(v)) = result.value else {
            panic!("expected scalar, got {:?}", result.value);
        };
        check_is_close(5.0, v).assert();
    }

    #[test]
    fn eval_below_continuous_limits() {
        let parameter = build_literal_parameter(
            "x",
            ir::Literal::number(0.0),
            [],
            continuous_limits(lit_number(1.0), lit_number(10.0)),
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::ParameterValueBelowContinuousLimits { .. }
            ),
            "expected ParameterValueBelowContinuousLimits, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_above_continuous_limits() {
        let parameter = build_literal_parameter(
            "x",
            ir::Literal::number(11.0),
            [],
            continuous_limits(lit_number(1.0), lit_number(10.0)),
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::ParameterValueAboveContinuousLimits { .. }
            ),
            "expected ParameterValueAboveContinuousLimits, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_within_number_discrete_limits() {
        let parameter = build_literal_parameter(
            "x",
            ir::Literal::number(2.0),
            [],
            discrete_limits(vec![lit_number(1.0), lit_number(2.0), lit_number(3.0)]),
        );
        let result = eval_parameter_simple(&parameter).expect("eval should succeed");
        assert_eq!(result.value, Value::Number(Number::Scalar(2.0)));
    }

    #[test]
    fn eval_not_in_number_discrete_limits() {
        let parameter = build_literal_parameter(
            "x",
            ir::Literal::number(4.0),
            [],
            discrete_limits(vec![lit_number(1.0), lit_number(2.0), lit_number(3.0)]),
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::ParameterValueNotInDiscreteLimits { .. }
            ),
            "expected ParameterValueNotInDiscreteLimits, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_within_string_discrete_limits() {
        let parameter = build_literal_parameter(
            "mode",
            ir::Literal::string("b".to_string()),
            [],
            discrete_limits(vec![lit_string("a"), lit_string("b"), lit_string("c")]),
        );
        let result = eval_parameter_simple(&parameter).expect("eval should succeed");
        assert_eq!(result.value, Value::String("b".to_string()));
    }

    #[test]
    fn eval_not_in_string_discrete_limits() {
        let parameter = build_literal_parameter(
            "mode",
            ir::Literal::string("z".to_string()),
            [],
            discrete_limits(vec![lit_string("a"), lit_string("b"), lit_string("c")]),
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::ParameterValueNotInDiscreteLimits { .. }
            ),
            "expected ParameterValueNotInDiscreteLimits, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_piecewise_selects_matching_branch() {
        let parameter = build_piecewise_parameter(
            "x",
            [
                (lit_number(1.0), lit_bool(false)),
                (lit_number(2.0), lit_bool(true)),
                (lit_number(3.0), lit_bool(false)),
            ],
            [],
        );
        let result = eval_parameter_simple(&parameter).expect("eval should succeed");
        assert_eq!(result.value, Value::Number(Number::Scalar(2.0)));
    }

    #[test]
    fn eval_piecewise_no_matching_branch() {
        let parameter = build_piecewise_parameter(
            "x",
            [
                (lit_number(1.0), lit_bool(false)),
                (lit_number(2.0), lit_bool(false)),
            ],
            [],
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::NoPiecewiseBranchMatch {
                    param_ident,
                    ..
                } if param_ident.as_str() == "x"
            ),
            "expected NoPiecewiseBranchMatch for x, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_piecewise_multiple_matching_branches() {
        let parameter = build_piecewise_parameter(
            "x",
            [
                (lit_number(1.0), lit_bool(true)),
                (lit_number(2.0), lit_bool(true)),
            ],
            [],
        );
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::MultiplePiecewiseBranchesMatch {
                    param_ident,
                    matching_branch_spans,
                    ..
                } if param_ident.as_str() == "x" && matching_branch_spans.len() == 2
            ),
            "expected MultiplePiecewiseBranchesMatch with 2 spans, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_piecewise_invalid_if_type() {
        let parameter = build_piecewise_parameter("x", [(lit_number(1.0), lit_number(0.0))], []);
        let errors = eval_parameter_simple(&parameter).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::InvalidIfExpressionType {
                    found_value: Value::Number(Number::Scalar(v)),
                    ..
                } if is_close(*v, 0.0)
            ),
            "expected InvalidIfExpressionType, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_measured_value_missing_unit_annotation() {
        // Seed a measured parameter, then reference it without a unit annotation.
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [("src", 1.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)])],
        );

        let parameter =
            build_parameter_from_expr("dst", param_var("src"), None, ir::Limits::default());

        let errors = eval_parameter(&parameter, &mut context).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], EvalError::ParameterMissingUnitAnnotation { .. }),
            "expected ParameterMissingUnitAnnotation, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_parameter_unit_mismatch() {
        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        setup_context_with_parameters(
            &mut context,
            [("src", 1.0, vec![UnitSpec::new(None, Some("m"), false, 1.0)])],
        );

        // Annotate as seconds while the value is in meters.
        let parameter = build_parameter_from_expr(
            "dst",
            param_var("src"),
            build_resolved_units([UnitSpec::new(None, Some("s"), false, 1.0)]),
            ir::Limits::default(),
        );

        let errors = eval_parameter(&parameter, &mut context).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], EvalError::ParameterUnitMismatch { .. }),
            "expected ParameterUnitMismatch, got {:?}",
            errors[0]
        );
    }
}
