use std::collections::HashSet;

use oneil_ir as ir;

use crate::{
    builtin::BuiltinFunction,
    context::EvalContext,
    error::EvalError,
    eval_expr, eval_unit,
    value::{MeasuredNumber, Number, SizedUnit, Unit, Value, ValueType},
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
) -> Result<(Value, TypecheckInfo), Vec<EvalError>> {
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
    let typecheck_info = typecheck_expr_value(value.type_(), unit_ir.as_ref(), context)?;

    // transform the value based on the typecheck info
    let value = transform_value(value, &typecheck_info);

    // check that the value is within the provided limits
    let limits = eval_limits(parameter.limits(), context)?;
    let limits = transform_limits(limits, &typecheck_info);
    verify_value_is_within_limits(&value, limits)?;

    Ok((value, typecheck_info))
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

#[derive(Debug, Clone, PartialEq)]
pub enum TypecheckInfo {
    String,
    Boolean,
    Number { sized_unit: SizedUnit, is_db: bool },
}

/// Typechecks the value of an expression against a unit.
///
/// If the parameter has a unit and the value is unitless,
/// the value is multiplied by the unit's magnitude.
///
/// In addition, if the value is a unitless number and the unit is a dB unit,
/// the value is converted from a logarithmic unit to a linear unit.
fn typecheck_expr_value<F: BuiltinFunction>(
    type_: ValueType,
    unit_ir: Option<&ir::CompositeUnit>,
    context: &EvalContext<F>,
) -> Result<TypecheckInfo, Vec<EvalError>> {
    match type_ {
        ValueType::Boolean => {
            if unit_ir.is_some() {
                Err(vec![EvalError::BooleanCannotHaveUnit])
            } else {
                Ok(TypecheckInfo::Boolean)
            }
        }
        ValueType::String => {
            if unit_ir.is_some() {
                Err(vec![EvalError::StringCannotHaveUnit])
            } else {
                Ok(TypecheckInfo::String)
            }
        }
        ValueType::Number {
            unit,
            number_type: _,
        } => {
            // evaluate the unit if it exists,
            // otherwise use the unitless unit
            let (sized_unit, is_db) = unit_ir
                .as_ref()
                .map(|unit| eval_unit(unit, context))
                .transpose()?
                .unwrap_or((SizedUnit::unitless(), false));

            // if the value is unitless, assign it the given unit
            // otherwise, typecheck the value's unit against the given unit
            if unit.is_unitless() || unit == sized_unit.unit {
                Ok(TypecheckInfo::Number { sized_unit, is_db })
            } else {
                Err(vec![EvalError::ParameterUnitMismatch])
            }
        }
    }
}

fn transform_value(value: Value, typecheck_info: &TypecheckInfo) -> Value {
    match (typecheck_info, value) {
        (TypecheckInfo::Number { sized_unit, is_db }, Value::Number(number))
            if number.unit.is_unitless() =>
        {
            let number_value = number.value * sized_unit.magnitude;
            let number_unit = sized_unit.unit.clone();

            // handle dB units
            let number_value = if *is_db {
                db_to_linear(number_value)
            } else {
                number_value
            };

            let number = MeasuredNumber {
                value: number_value,
                unit: number_unit,
            };

            Value::Number(number)
        }
        (TypecheckInfo::String, value @ Value::String(_))
        | (TypecheckInfo::Boolean, value @ Value::Boolean(_))
        | (TypecheckInfo::Number { .. }, value @ Value::Number(_)) => value,
        (_, _) => unreachable!(
            "this shouldn't happen because the result of typechecking should always match the value's type"
        ),
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

fn transform_limits(limits: Limits, typecheck_info: &TypecheckInfo) -> Limits {
    match (limits, typecheck_info) {
        (Limits::NumberRange { min, max, unit }, TypecheckInfo::Number { sized_unit, is_db })
            if unit.is_unitless() =>
        {
            let min = min * Number::Scalar(sized_unit.magnitude);
            let max = max * Number::Scalar(sized_unit.magnitude);
            let unit = sized_unit.unit.clone();

            // handle dB units
            let min = if *is_db { db_to_linear(min) } else { min };
            let max = if *is_db { db_to_linear(max) } else { max };

            Limits::NumberRange { min, max, unit }
        }

        (Limits::NumberDiscrete { values, unit }, TypecheckInfo::Number { sized_unit, is_db })
            if unit.is_unitless() =>
        {
            let values = values
                .into_iter()
                .map(|value| value * Number::Scalar(sized_unit.magnitude))
                .map(|value| if *is_db { db_to_linear(value) } else { value })
                .collect();

            let unit = sized_unit.unit.clone();
            Limits::NumberDiscrete { values, unit }
        }

        (
            Limits::AnyStringOrBooleanOrPositiveNumber,
            TypecheckInfo::Number { sized_unit, is_db },
        ) if *is_db => {
            Limits::NumberRange {
                min: Number::Scalar(1.0), // 0 db == 1
                max: Number::Scalar(f64::INFINITY),
                unit: sized_unit.unit.clone(),
            }
        }

        (
            limits @ (Limits::NumberRange { .. } | Limits::NumberDiscrete { .. }),
            TypecheckInfo::Number { .. },
        )
        | (limits @ Limits::StringDiscrete { .. }, TypecheckInfo::String)
        | (limits @ Limits::AnyStringOrBooleanOrPositiveNumber, _) => limits,

        (_, _) => {
            unreachable!("this shouldn't happen because typechecking should have already occurred")
        }
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
                if !unit.is_unitless() && number.unit != unit {
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
                if !unit.is_unitless() && number.unit != unit {
                    return Err(vec![EvalError::ParameterUnitDoesNotMatchLimit]);
                }

                let is_inside_limits = values
                    .iter()
                    .any(|limit_value| limit_value.contains(number.value));

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

fn db_to_linear(value: Number) -> Number {
    Number::Scalar(10.0).pow(value / Number::Scalar(10.0))
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_is_close, assert_units_eq,
        builtin::{self},
        value::Dimension,
    };

    use super::*;

    #[test]
    fn eval_no_unit() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, []);
        let context = helper::create_eval_context([]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        assert_is_close!(1.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_with_unit_m() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("m", 1.0)]);
        let context = helper::create_eval_context([]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        assert_is_close!(1.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_with_unit_km() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("km", 1.0)]);
        let context = helper::create_eval_context([]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // 1.0 km = 1000.0 m
        assert_is_close!(1000.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1000.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_with_unit_km_per_hr() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("km", 1.0), ("hr", -1.0)]);
        let context = helper::create_eval_context([]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0), (Dimension::Time, -1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // 1.0 km/hr = 1000.0 m / 3600.0 s = 0.277777... m/s
        assert_is_close!(1000.0 / 3600.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1000.0 / 3600.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_with_unit_db() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("dB", 1.0)]);
        let context = helper::create_eval_context([]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // 1.0 dB = 10^(1.0/10.0) = 10^0.1 = 1.258925...
        assert_is_close!(10.0_f64.powf(0.1), value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(is_db);
    }

    #[test]
    fn eval_with_unit_dbw() {
        // setup parameter and context
        let parameter = helper::build_simple_parameter("x", 1.0, [("dBW", 1.0)]);
        let context = helper::create_eval_context([]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // 1.0 dBW = 10^(1.0/10.0) = 10^0.1 = 1.258925...
        assert_is_close!(10.0_f64.powf(0.1), value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // x + y = 1.0 m + 1000.0 m = 1001.0 m
        // The value is stored in base units (meters)
        assert_is_close!(1001.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1000.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 1.0),
            (Dimension::Time, -2.0),
        ];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // x + y = 1.0 N + 1.0 N = 2.0 N
        // The value is stored in base units
        assert_is_close!(2.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [
            (Dimension::Mass, 1.0),
            (Dimension::Distance, 2.0),
            (Dimension::Time, -3.0),
        ];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // x = 1.0 dBW = 10^(1.0/10.0) = 10^0.1 = 1.258925... W
        // y = 1.0 W
        // x + y = 1.258925... W + 1.0 W = 2.258925... W
        assert_is_close!(10.0_f64.powf(0.1) + 1.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_exponent_parameter_w_squared() {
        // setup context with x = 1.0 W
        let context = helper::create_eval_context([("x", 1.0, vec![("W", 1.0)])]);

        // setup parameter y = x^2 with unit W^2
        let parameter = helper::build_exponent_parameter("y", "x", 2.0, [("W", 2.0)]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [
            (Dimension::Mass, 2.0),
            (Dimension::Distance, 4.0),
            (Dimension::Time, -6.0),
        ];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // y = x^2 = (1.0 W)^2 = 1.0 W^2
        assert_is_close!(1.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 2.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // z = x * y = 3.0 m * 2.0 m = 6.0 m^2
        assert_is_close!(6.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // z = x / y = 6.0 m^2 / 2.0 m = 3.0 m
        assert_is_close!(3.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // z = x % y = 7.0 m % 3.0 m = 1.0 m
        assert_is_close!(1.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_sqrt_function() {
        // setup context with x = 4.0 m^2
        let context = helper::create_eval_context([("x", 4.0, vec![("m", 2.0)])]);

        // setup parameter y = sqrt(x) with unit m
        let parameter = helper::build_function_call_parameter("y", "sqrt", ["x"], [("m", 1.0)]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // y = sqrt(x) = sqrt(4.0 m^2) = 2.0 m
        assert_is_close!(2.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // z = min(x, y) = min(3.0 m, 5.0 m) = 3.0 m
        assert_is_close!(3.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_min_function_with_interval() {
        // setup context
        let mut context = helper::create_eval_context([]);

        // add x as an interval parameter [2.0, 4.0] m
        let x_parameter = helper::build_interval_parameter("x", 2.0, 4.0, [("m", 1.0)]);
        let (x_value, _) = eval_parameter(&x_parameter, &context).expect("eval should succeed");
        context.add_parameter_result("x".to_string(), Ok(x_value));

        // setup parameter z = min(x) with unit m
        let parameter = helper::build_function_call_parameter("z", "min", ["x"], [("m", 1.0)]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // x = [2.0, 4.0] m
        // min(x) = min(2.0, 4.0) = 2.0 m
        assert_is_close!(2.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // z = max(x, y) = max(3.0 m, 5.0 m) = 5.0 m
        assert_is_close!(5.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_max_function_with_interval() {
        // setup context
        let mut context = helper::create_eval_context([]);

        // add x as an interval parameter [2.0, 4.0] m
        let x_parameter = helper::build_interval_parameter("x", 2.0, 4.0, [("m", 1.0)]);
        let (x_value, _) = eval_parameter(&x_parameter, &context).expect("eval should succeed");
        context.add_parameter_result("x".to_string(), Ok(x_value));

        // setup parameter z = max(x) with unit m
        let parameter = helper::build_function_call_parameter("z", "max", ["x"], [("m", 1.0)]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // x = [2.0, 4.0] m
        // max(x) = max(2.0, 4.0) = 4.0 m
        assert_is_close!(4.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    #[test]
    fn eval_range_function() {
        // setup context
        let mut context = helper::create_eval_context([]);

        // add x as an interval parameter [2.0, 4.0] m
        let x_parameter = helper::build_interval_parameter("x", 2.0, 4.0, [("m", 1.0)]);
        let (x_value, _) = eval_parameter(&x_parameter, &context).expect("eval should succeed");
        context.add_parameter_result("x".to_string(), Ok(x_value));

        // setup parameter z = range(x) with unit m
        let parameter = helper::build_function_call_parameter("z", "range", ["x"], [("m", 1.0)]);
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // x = [2.0, 4.0] m
        // range(x) = max(2.0, 4.0) - min(2.0, 4.0) = 4.0 - 2.0 = 2.0 m
        assert_is_close!(2.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
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
        let (parameter_value, typecheck_info) =
            eval_parameter(&parameter, &context).expect("eval should succeed");

        let expected_units = [(Dimension::Distance, 1.0)];

        // check the parameter value
        let Value::Number(number) = parameter_value else {
            panic!("expected number");
        };

        let Number::Scalar(value) = number.value else {
            panic!("expected scalar");
        };

        // z = mid(x, y) = (x + y) / 2 = (2.0 m + 4.0 m) / 2 = 3.0 m
        assert_is_close!(3.0, value);
        assert_units_eq!(expected_units, number.unit);

        // check the typecheck info
        let TypecheckInfo::Number { sized_unit, is_db } = typecheck_info else {
            panic!("expected number typecheck info");
        };

        assert_units_eq!(expected_units, sized_unit.unit);
        assert_is_close!(1.0, sized_unit.magnitude);
        assert!(!is_db);
    }

    mod helper {
        use super::*;

        use std::path::PathBuf;

        use crate::builtin::BuiltinMap;

        use crate::context::EvalContext;

        use crate::error::EvalError;

        use crate::value::Value;

        use std::collections::HashSet;

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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
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
                name: ir::FunctionName::Builtin(ir::Identifier::new(function.to_string())),
                args,
            };

            let units = units
                .into_iter()
                .map(|(unit, exponent)| ir::Unit::new(unit.to_string(), exponent))
                .collect();
            let units = ir::CompositeUnit::new(units);

            ir::Parameter::new(
                HashSet::new(),
                ir::ParameterName::new(name.to_string()),
                random_span(),
                random_span(),
                ir::ParameterValue::simple(expr, Some(units)),
                ir::Limits::default(),
                false,
                ir::TraceLevel::None,
            )
        }

        /// Type alias for builtin functions used in tests.
        pub type BuiltinFunction = fn(Vec<Value>) -> Result<Value, Vec<EvalError>>;

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
        ) -> EvalContext<BuiltinFunction> {
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
                let (parameter_value, _) =
                    eval_parameter(&parameter, &context).expect("eval should succeed");
                context.add_parameter_result(name.to_string(), Ok(parameter_value));
            }

            context
        }
    }
}
