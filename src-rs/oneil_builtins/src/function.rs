//! Standard builtin functions (e.g. min, max, sqrt).

use oneil_eval::EvalError;
use oneil_output::Value;
use oneil_shared::span::Span;

/// Information about a builtin function.
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    /// The name of the function.
    pub name: &'static str,
    /// The arguments of the function.
    pub args: &'static [&'static str],
    /// The description of the function.
    pub description: &'static str,
    /// The function implementation.
    pub function: BuiltinFunctionFn,
}

/// Type alias for standard builtin function type.
pub type BuiltinFunctionFn = fn(Span, Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>>;

#[expect(clippy::too_many_lines, reason = "this is a list of builtin functions")]
/// Returns an iterator over all standard builtin functions.
pub fn builtin_functions_complete() -> impl Iterator<Item = (&'static str, BuiltinFunction)> {
    [
        BuiltinFunction {
            name: "min",
            args: &["n", "..."],
            description: fns::MIN_DESCRIPTION,
            function: fns::min as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "max",
            args: &["n", "..."],
            description: fns::MAX_DESCRIPTION,
            function: fns::max as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "sin",
            args: &["x"],
            description: fns::SIN_DESCRIPTION,
            function: fns::sin as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "cos",
            args: &["x"],
            description: fns::COS_DESCRIPTION,
            function: fns::cos as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "tan",
            args: &["x"],
            description: fns::TAN_DESCRIPTION,
            function: fns::tan as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "asin",
            args: &["x"],
            description: fns::ASIN_DESCRIPTION,
            function: fns::asin as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "acos",
            args: &["x"],
            description: fns::ACOS_DESCRIPTION,
            function: fns::acos as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "atan",
            args: &["x"],
            description: fns::ATAN_DESCRIPTION,
            function: fns::atan as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "sqrt",
            args: &["x"],
            description: fns::SQRT_DESCRIPTION,
            function: fns::sqrt as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "ln",
            args: &["x"],
            description: fns::LN_DESCRIPTION,
            function: fns::ln as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "log2",
            args: &["x"],
            description: fns::LOG2_DESCRIPTION,
            function: fns::log2 as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "log10",
            args: &["x"],
            description: fns::LOG10_DESCRIPTION,
            function: fns::log10 as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "floor",
            args: &["x"],
            description: fns::FLOOR_DESCRIPTION,
            function: fns::floor as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "ceiling",
            args: &["x"],
            description: fns::CEILING_DESCRIPTION,
            function: fns::ceiling as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "range",
            args: &["x", "y?"],
            description: fns::RANGE_DESCRIPTION,
            function: fns::range as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "abs",
            args: &["x"],
            description: fns::ABS_DESCRIPTION,
            function: fns::abs as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "sign",
            args: &["x"],
            description: fns::SIGN_DESCRIPTION,
            function: fns::sign as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "mid",
            args: &["x", "y?"],
            description: fns::MID_DESCRIPTION,
            function: fns::mid as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "strip",
            args: &["x"],
            description: fns::STRIP_DESCRIPTION,
            function: fns::strip as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: "mnmx",
            args: &["n", "..."],
            description: fns::MNMX_DESCRIPTION,
            function: fns::mnmx as BuiltinFunctionFn,
        },
    ]
    .into_iter()
    .map(|function| (function.name, function))
}

mod fns {
    use std::borrow::Cow;

    use oneil_eval::{
        EvalError,
        error::{ExpectedArgumentCount, ExpectedType, convert::binary_eval_error_to_eval_error},
    };
    use oneil_output::{MeasuredNumber, Number, NumberType, Unit, Value};
    use oneil_shared::span::Span;

    use super::helper;

    pub const MIN_DESCRIPTION: &str = "Find the minimum value of the given values.\n\nIf a value is an interval, the minimum value of the interval is used.";

    #[expect(
        clippy::needless_pass_by_value,
        reason = "matches the expected signature"
    )]
    pub fn min(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: "min".to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::AtLeast(1),
                actual_argument_count: args.len(),
            }]);
        }

        let number_list = helper::extract_homogeneous_numbers_list(&args)?;

        match number_list {
            helper::HomogeneousNumberList::Numbers(numbers) => numbers
                .into_iter()
                .filter_map(|number| match number.as_ref() {
                    Number::Scalar(value) => Some(*value),
                    Number::Interval(interval) => {
                        if interval.is_empty() {
                            None
                        } else {
                            Some(interval.min())
                        }
                    }
                })
                .reduce(f64::min)
                .map_or_else(
                    || {
                        Err(vec![EvalError::BuiltinFnCustomError {
                            error_location: identifier_span,
                            msg: "list contains no numbers or non-empty intervals".to_string(),
                        }])
                    },
                    |min| Ok(Value::Number(Number::Scalar(min))),
                ),
            helper::HomogeneousNumberList::MeasuredNumbers(numbers) => numbers
                .into_iter()
                .filter_map(|number| match number.normalized_value().as_number() {
                    Number::Scalar(_) => Some(number.min()),
                    Number::Interval(interval) => {
                        if interval.is_empty() {
                            None
                        } else {
                            Some(number.min())
                        }
                    }
                })
                .reduce(|a, b| {
                    if a.normalized_value().lt(b.normalized_value()) {
                        a
                    } else {
                        b
                    }
                })
                .map_or_else(
                    || {
                        Err(vec![EvalError::BuiltinFnCustomError {
                            error_location: identifier_span,
                            msg: "list contains no numbers or non-empty intervals".to_string(),
                        }])
                    },
                    |min| Ok(Value::MeasuredNumber(min)),
                ),
        }
    }

    pub const MAX_DESCRIPTION: &str = "Find the maximum value of the given values.\n\nIf a value is an interval, the maximum value of the interval is used.";

    #[expect(
        clippy::needless_pass_by_value,
        reason = "matches the expected signature"
    )]
    pub fn max(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: "max".to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::AtLeast(1),
                actual_argument_count: args.len(),
            }]);
        }

        let number_list = helper::extract_homogeneous_numbers_list(&args)?;

        match number_list {
            helper::HomogeneousNumberList::Numbers(numbers) => numbers
                .into_iter()
                .filter_map(|number| match number.as_ref() {
                    Number::Scalar(value) => Some(*value),
                    Number::Interval(interval) => {
                        if interval.is_empty() {
                            None
                        } else {
                            Some(interval.max())
                        }
                    }
                })
                .reduce(f64::max)
                .map_or_else(
                    || {
                        Err(vec![EvalError::BuiltinFnCustomError {
                            error_location: identifier_span,
                            msg: "list contains no numbers or non-empty intervals".to_string(),
                        }])
                    },
                    |max| Ok(Value::Number(Number::Scalar(max))),
                ),
            helper::HomogeneousNumberList::MeasuredNumbers(numbers) => numbers
                .into_iter()
                .filter_map(|number| match number.normalized_value().as_number() {
                    Number::Scalar(_) => Some(number.max()),
                    Number::Interval(interval) => {
                        if interval.is_empty() {
                            None
                        } else {
                            Some(number.max())
                        }
                    }
                })
                .reduce(|a, b| {
                    if a.normalized_value().gt(b.normalized_value()) {
                        a
                    } else {
                        b
                    }
                })
                .map_or_else(
                    || {
                        Err(vec![EvalError::BuiltinFnCustomError {
                            error_location: identifier_span,
                            msg: "list contains no numbers or non-empty intervals".to_string(),
                        }])
                    },
                    |max| Ok(Value::MeasuredNumber(max)),
                ),
        }
    }

    pub const SIN_DESCRIPTION: &str = "Compute the sine of an angle.";

    pub fn sin(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        // the base unit for angles is radians,
        // so we need to convert from dimensionless with no magnitude
        helper::unary_measured_number_fn(identifier_span, args, "sin", |m, arg_span| {
            let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
            Ok(Value::Number(number.sin()))
        })
    }

    pub const COS_DESCRIPTION: &str = "Compute the cosine of an angle.";

    pub fn cos(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        // the base unit for angles is radians,
        // so we need to convert from dimensionless with no magnitude
        helper::unary_measured_number_fn(identifier_span, args, "cos", |m, arg_span| {
            let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
            Ok(Value::Number(number.cos()))
        })
    }

    pub const TAN_DESCRIPTION: &str = "Compute the tangent of an angle.";

    pub fn tan(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        // the base unit for angles is radians,
        // so we need to convert from dimensionless with no magnitude
        helper::unary_measured_number_fn(identifier_span, args, "tan", |m, arg_span| {
            let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
            Ok(Value::Number(number.tan()))
        })
    }

    pub const ASIN_DESCRIPTION: &str =
        "Compute the arcsine (inverse sine) of a value, returning an angle.";

    pub fn asin(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        // the base unit for angles is radians,
        // so we essentially need to convert to unit `1`

        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "asin",
            |n, _arg_span| {
                let number = n.asin();
                let unit = Unit::one();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
            |m, arg_span| {
                let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
                let number = number.asin();
                let unit = Unit::one();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
        )
    }

    pub const ACOS_DESCRIPTION: &str =
        "Compute the arccosine (inverse cosine) of a value, returning an angle.";

    pub fn acos(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        // the base unit for angles is radians,
        // so we essentially need to convert to unit `1`

        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "acos",
            |n, _arg_span| {
                let number = n.acos();
                let unit = Unit::one();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
            |m, arg_span| {
                let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
                let number = number.acos();
                let unit = Unit::one();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
        )
    }

    pub const ATAN_DESCRIPTION: &str =
        "Compute the arctangent (inverse tangent) of a value, returning an angle.";

    pub fn atan(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        // the base unit for angles is radians,
        // so we essentially need to convert to unit `1`

        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "atan",
            |n, _arg_span| {
                let number = n.atan();
                let unit = Unit::one();

                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
            |m, arg_span| {
                let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
                let number = number.atan();
                let unit = Unit::one();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
        )
    }

    pub const SQRT_DESCRIPTION: &str = "Compute the square root of a value.";

    pub fn sqrt(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "sqrt",
            |n, _arg_span| Ok(Value::Number(n.sqrt())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.sqrt())),
        )
    }

    pub const LN_DESCRIPTION: &str = "Compute the natural logarithm (base e) of a value.";

    /// Returns the natural logarithm (base e) of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn ln(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "ln",
            |n, _arg_span| Ok(Value::Number(n.ln())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.ln())),
        )
    }

    pub const LOG10_DESCRIPTION: &str = "Compute the base-10 logarithm of a value.";

    /// Returns the base-10 logarithm of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn log10(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "log10",
            |n, _arg_span| Ok(Value::Number(n.log10())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.log10())),
        )
    }

    pub const LOG2_DESCRIPTION: &str = "Compute the base-2 logarithm of a value.";

    /// Returns the base-2 logarithm of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn log2(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "log2",
            |n, _arg_span| Ok(Value::Number(n.log2())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.log2())),
        )
    }

    pub const FLOOR_DESCRIPTION: &str = "Round a value down to the nearest integer.";

    /// Returns the single numerical argument rounded down to the nearest integer.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn floor(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "floor",
            |n, _arg_span| Ok(Value::Number(n.floor())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.floor())),
        )
    }

    pub const CEILING_DESCRIPTION: &str = "Round a value up to the nearest integer.";

    /// Returns the single numerical argument rounded up to the nearest integer.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn ceiling(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "ceiling",
            |n, _arg_span| Ok(Value::Number(n.ceiling())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.ceiling())),
        )
    }

    pub const RANGE_DESCRIPTION: &str = "Compute the range of values.\n\nWith one argument (an interval), returns the difference between the maximum and minimum.\n\nWith two arguments, returns the difference between them.";

    pub fn range(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        match args.len() {
            1 => {
                let mut args = args.into_iter();

                let (arg, arg_span) = args.next().expect("there should be one argument");

                let (number_value, unit) = match arg {
                    Value::MeasuredNumber(number) => {
                        let (number_value, unit) = number.into_number_and_unit();
                        (number_value, Some(unit))
                    }
                    Value::Number(number) => (number, None),
                    Value::Boolean(_) | Value::String(_) => {
                        return Err(vec![EvalError::InvalidType {
                            expected_type: ExpectedType::NumberOrMeasuredNumber,
                            found_type: arg.type_(),
                            found_span: arg_span,
                        }]);
                    }
                };

                let Number::Interval(interval) = number_value else {
                    return Err(vec![EvalError::InvalidNumberType {
                        number_type: NumberType::Interval,
                        found_number_type: number_value.type_(),
                        found_span: arg_span,
                    }]);
                };

                let result = interval.max() - interval.min();

                unit.map_or(Ok(Value::Number(Number::Scalar(result))), |unit| {
                    Ok(Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
                        Number::Scalar(result),
                        unit,
                    )))
                })
            }
            2 => {
                let mut args = args.into_iter();

                let (left, left_span) = args.next().expect("there should be two arguments");
                let (right, right_span) = args.next().expect("there should be two arguments");

                left.checked_sub(right).map_err(|error| {
                    vec![binary_eval_error_to_eval_error(
                        error, left_span, right_span,
                    )]
                })
            }
            _ => Err(vec![EvalError::InvalidArgumentCount {
                function_name: "range".to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Between(1, 2),
                actual_argument_count: args.len(),
            }]),
        }
    }

    pub const ABS_DESCRIPTION: &str = "Compute the absolute value of a number.";

    /// Returns the absolute value of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn abs(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "abs",
            |n, _arg_span| Ok(Value::Number(n.abs())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.abs())),
        )
    }

    pub const SIGN_DESCRIPTION: &str =
        "Compute the sign of a number, returning -1 for negative, 0 for zero, or 1 for positive.";

    /// Returns the sign of the single numerical argument (-1, 0, or 1).
    ///
    /// For measured numbers, the unit is dropped since sign is dimensionless.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn sign(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "sign",
            |n, _arg_span| Ok(Value::Number(n.sign())),
            |m, _arg_span| {
                let (number, _unit) = m.into_number_and_unit();
                Ok(Value::Number(number.sign()))
            },
        )
    }

    pub const MID_DESCRIPTION: &str = "Compute the midpoint.\n\nWith one argument (an interval), returns the midpoint of the interval.\n\nWith two arguments, returns the midpoint between them.";

    pub fn mid(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        match args.len() {
            1 => {
                let mut args = args.into_iter();

                let (arg, arg_span) = args.next().expect("there should be one argument");

                let (number_value, unit) = match arg {
                    Value::MeasuredNumber(number) => {
                        let (number_value, unit) = number.into_number_and_unit();
                        (number_value, Some(unit))
                    }
                    Value::Number(number) => (number, None),
                    Value::Boolean(_) | Value::String(_) => {
                        return Err(vec![EvalError::InvalidType {
                            expected_type: ExpectedType::NumberOrMeasuredNumber,
                            found_type: arg.type_(),
                            found_span: arg_span,
                        }]);
                    }
                };

                let Number::Interval(interval) = number_value else {
                    return Err(vec![EvalError::InvalidNumberType {
                        number_type: NumberType::Interval,
                        found_number_type: number_value.type_(),
                        found_span: arg_span,
                    }]);
                };

                let mid = f64::midpoint(interval.min(), interval.max());

                unit.map_or(Ok(Value::Number(Number::Scalar(mid))), |unit| {
                    Ok(Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
                        Number::Scalar(mid),
                        unit,
                    )))
                })
            }
            2 => {
                let mut args = args.into_iter();

                let (left, left_span) = args.next().expect("there should be two arguments");
                let (right, right_span) = args.next().expect("there should be two arguments");

                left.checked_add(right)
                    .and_then(|value| value.checked_div(Value::from(2.0)))
                    .map_err(|error| {
                        vec![binary_eval_error_to_eval_error(
                            error, left_span, right_span,
                        )]
                    })
            }
            _ => Err(vec![EvalError::InvalidArgumentCount {
                function_name: "mid".to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Between(1, 2),
                actual_argument_count: args.len(),
            }]),
        }
    }

    pub const STRIP_DESCRIPTION: &str =
        "Strip units from a measured number, returning just the numeric value.";

    /// Strips units from a measured number, returning the numeric value.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn strip(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        helper::unary_measured_number_fn(identifier_span, args, "strip", |m, _arg_span| {
            Ok(Value::Number(m.into_number_and_unit().0))
        })
    }

    pub const MNMX_DESCRIPTION: &str =
        "Return both the minimum and maximum values from the given values.";

    /// Returns the tightest interval containing all given values (min of mins, max of maxes).
    ///
    /// # Errors
    ///
    /// Returns `Err` if no arguments are given, or if arguments are not
    /// homogeneous numbers or measured numbers.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "matches the expected signature"
    )]
    pub fn mnmx(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: "mnmx".to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::AtLeast(1),
                actual_argument_count: args.len(),
            }]);
        }

        let number_list = helper::extract_homogeneous_numbers_list(&args)?;

        match number_list {
            helper::HomogeneousNumberList::Numbers(numbers) => {
                let result = numbers
                    .into_iter()
                    .map(Cow::into_owned)
                    .reduce(Number::tightest_enclosing_interval)
                    .expect("there should be at least one number");

                Ok(Value::Number(result))
            }
            helper::HomogeneousNumberList::MeasuredNumbers(numbers) => {
                let result = numbers
                    .into_iter()
                    .map(Cow::into_owned)
                    .reduce(|a, b| {
                        a.checked_min_max(&b)
                            .expect("homogeneous list ensures same unit")
                    })
                    .expect("there should be at least one number");

                Ok(Value::MeasuredNumber(result))
            }
        }
    }
}

mod helper {
    use std::borrow::Cow;

    use oneil_shared::span::Span;

    use oneil_eval::{
        EvalError,
        error::{ExpectedArgumentCount, ExpectedType},
    };
    use oneil_output::{DisplayUnit, MeasuredNumber, Number, Unit, Value};

    pub fn unary_number_or_measured_number_fn<F, G>(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        name: &str,
        number_op: F,
        measured_op: G,
    ) -> Result<Value, Vec<EvalError>>
    where
        F: FnOnce(Number, Span) -> Result<Value, Vec<EvalError>>,
        G: FnOnce(MeasuredNumber, Span) -> Result<Value, Vec<EvalError>>,
    {
        if args.len() != 1 {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: name.to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Exact(1),
                actual_argument_count: args.len(),
            }]);
        }

        let (arg, arg_span) = args
            .into_iter()
            .next()
            .expect("there should be one argument");
        let found_type = arg.type_();

        match arg {
            Value::Number(number) => number_op(number, arg_span),
            Value::MeasuredNumber(measured) => measured_op(measured, arg_span),
            Value::Boolean(_) | Value::String(_) => Err(vec![EvalError::InvalidType {
                expected_type: ExpectedType::NumberOrMeasuredNumber,
                found_type,
                found_span: arg_span,
            }]),
        }
    }

    pub fn unary_measured_number_fn<F>(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        name: &str,
        measured_op: F,
    ) -> Result<Value, Vec<EvalError>>
    where
        F: FnOnce(MeasuredNumber, Span) -> Result<Value, Vec<EvalError>>,
    {
        if args.len() != 1 {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: name.to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Exact(1),
                actual_argument_count: args.len(),
            }]);
        }

        let (arg, arg_span) = args
            .into_iter()
            .next()
            .expect("there should be one argument");
        let found_type = arg.type_();

        match arg {
            Value::MeasuredNumber(measured) => measured_op(measured, arg_span),
            Value::Number(_) | Value::Boolean(_) | Value::String(_) => {
                Err(vec![EvalError::InvalidType {
                    expected_type: ExpectedType::MeasuredNumber,
                    found_type,
                    found_span: arg_span,
                }])
            }
        }
    }

    /// Converts a measured number into a unitless number.
    ///
    /// Returns an error if the measured number is not dimensionless.
    pub fn dimensionless_measured_number_as_number(
        measured: MeasuredNumber,
        argument_span: Span,
    ) -> Result<Number, Vec<EvalError>> {
        if !measured.unit().is_dimensionless() {
            return Err(vec![EvalError::InvalidUnit {
                expected_unit: None,
                found_unit: Some(measured.unit().display_unit.clone()),
                found_span: argument_span,
            }]);
        }

        Ok(measured.into_number_using_unit(&Unit::one()))
    }

    // Use a Cow (Clone on Write) to avoid unnecessary cloning.
    #[derive(Debug)]
    pub enum HomogeneousNumberList<'a> {
        Numbers(Vec<Cow<'a, Number>>),
        MeasuredNumbers(Vec<Cow<'a, MeasuredNumber>>),
    }

    #[derive(Debug)]
    enum ListResult<'a> {
        Numbers {
            numbers: Vec<Cow<'a, Number>>,
            first_number_span: &'a Span,
        },
        MeasuredNumbers {
            numbers: Vec<Cow<'a, MeasuredNumber>>,
            expected_unit: &'a Unit,
            expected_unit_value_span: &'a Span,
        },
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "callers enforce non-empty list to provide correct error message"
    )]
    pub fn extract_homogeneous_numbers_list(
        values: &[(Value, Span)],
    ) -> Result<HomogeneousNumberList<'_>, Vec<EvalError>> {
        assert!(!values.is_empty());

        // if there are exactly two values, then the unit of
        // one value may be inferred from the other
        if values.len() == 2 {
            return extract_two_numbers_list(values);
        }

        // otherwise, we need to iterate over the values and
        // collect them into a homogeneous list
        let mut list_result: Option<ListResult<'_>> = None;
        let mut errors = Vec::new();

        for (value, value_span) in values {
            match value {
                Value::MeasuredNumber(number) => {
                    handle_measured_number(number, value_span, &mut list_result, &mut errors);
                }
                Value::Number(number) => {
                    handle_number(number, value_span, &mut list_result, &mut errors);
                }
                Value::String(_) | Value::Boolean(_) => {
                    handle_invalid_type(value, value_span, &mut errors);
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let list_result = list_result.expect("at least one number");
        Ok(convert_to_homogeneous_list(list_result))
    }

    /// Extracts a list of two homogeneous numbers from the given values.
    ///
    /// If one value is a number and the other is a measured number, then
    /// the unit of the number is inferred from the measured number.
    #[expect(
        clippy::panic_in_result_fn,
        reason = "callers enforce exactly two values"
    )]
    fn extract_two_numbers_list(
        values: &[(Value, Span)],
    ) -> Result<HomogeneousNumberList<'_>, Vec<EvalError>> {
        assert!(values.len() == 2);

        let (left, left_span) = &values[0];
        let (right, right_span) = &values[1];

        match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
                let left = Cow::Borrowed(left);
                let right = Cow::Borrowed(right);

                Ok(HomogeneousNumberList::Numbers(vec![left, right]))
            }
            (Value::MeasuredNumber(left), Value::MeasuredNumber(right)) => {
                let left = Cow::Borrowed(left);
                let right = Cow::Borrowed(right);

                Ok(HomogeneousNumberList::MeasuredNumbers(vec![left, right]))
            }
            (Value::Number(left), Value::MeasuredNumber(right)) => {
                let right_unit = right.unit();
                let left = MeasuredNumber::from_number_and_unit(*left, right_unit.clone());
                let left = Cow::Owned(left);

                let right = Cow::Borrowed(right);

                Ok(HomogeneousNumberList::MeasuredNumbers(vec![left, right]))
            }
            (Value::MeasuredNumber(left), Value::Number(right)) => {
                let left = Cow::Borrowed(left);

                let left_unit = left.unit();
                let right = MeasuredNumber::from_number_and_unit(*right, left_unit.clone());
                let right = Cow::Owned(right);

                Ok(HomogeneousNumberList::MeasuredNumbers(vec![left, right]))
            }
            (
                left @ (Value::Boolean(_) | Value::String(_)),
                right @ (Value::Boolean(_) | Value::String(_)),
            ) => Err(vec![
                EvalError::InvalidType {
                    expected_type: ExpectedType::NumberOrMeasuredNumber,
                    found_type: left.type_(),
                    found_span: *left_span,
                },
                EvalError::InvalidType {
                    expected_type: ExpectedType::NumberOrMeasuredNumber,
                    found_type: right.type_(),
                    found_span: *right_span,
                },
            ]),
            (
                left @ (Value::Boolean(_) | Value::String(_)),
                _right @ (Value::Number(_) | Value::MeasuredNumber(_)),
            ) => Err(vec![EvalError::InvalidType {
                expected_type: ExpectedType::NumberOrMeasuredNumber,
                found_type: left.type_(),
                found_span: *left_span,
            }]),
            (
                _left @ (Value::Number(_) | Value::MeasuredNumber(_)),
                right @ (Value::Boolean(_) | Value::String(_)),
            ) => Err(vec![EvalError::InvalidType {
                expected_type: ExpectedType::NumberOrMeasuredNumber,
                found_type: right.type_(),
                found_span: *right_span,
            }]),
        }
    }

    fn handle_measured_number<'a>(
        number: &'a MeasuredNumber,
        value_span: &'a Span,
        list_result: &mut Option<ListResult<'a>>,
        errors: &mut Vec<EvalError>,
    ) {
        match list_result {
            Some(ListResult::MeasuredNumbers {
                numbers,
                expected_unit,
                expected_unit_value_span,
            }) => {
                if number.unit().dimensionally_eq(expected_unit) {
                    let number = Cow::Borrowed(number);
                    numbers.push(number);
                } else {
                    errors.push(EvalError::UnitMismatch {
                        expected_unit: expected_unit.display_unit.clone(),
                        expected_source_span: **expected_unit_value_span,
                        found_unit: number.unit().display_unit.clone(),
                        found_span: *value_span,
                    });
                }
            }
            Some(ListResult::Numbers {
                numbers: _,
                first_number_span,
            }) => {
                errors.push(EvalError::UnitMismatch {
                    expected_unit: DisplayUnit::One,
                    expected_source_span: **first_number_span,
                    found_unit: number.unit().display_unit.clone(),
                    found_span: *value_span,
                });
            }
            None => {
                let number_unit = number.unit();
                let number = Cow::Borrowed(number);

                *list_result = Some(ListResult::MeasuredNumbers {
                    numbers: vec![number],
                    expected_unit: number_unit,
                    expected_unit_value_span: value_span,
                });
            }
        }
    }

    fn handle_number<'a>(
        number: &'a Number,
        value_span: &'a Span,
        list_result: &mut Option<ListResult<'a>>,
        errors: &mut Vec<EvalError>,
    ) {
        match list_result {
            Some(ListResult::MeasuredNumbers {
                numbers: _,
                expected_unit,
                expected_unit_value_span,
            }) => {
                errors.push(EvalError::UnitMismatch {
                    expected_unit: expected_unit.display_unit.clone(),
                    expected_source_span: **expected_unit_value_span,
                    found_unit: DisplayUnit::One,
                    found_span: *value_span,
                });
            }
            Some(ListResult::Numbers { numbers, .. }) => {
                let number = Cow::Borrowed(number);
                numbers.push(number);
            }
            None => {
                let number = Cow::Borrowed(number);
                *list_result = Some(ListResult::Numbers {
                    numbers: vec![number],
                    first_number_span: value_span,
                });
            }
        }
    }

    fn handle_invalid_type(value: &Value, value_span: &Span, errors: &mut Vec<EvalError>) {
        errors.push(EvalError::InvalidType {
            expected_type: ExpectedType::NumberOrMeasuredNumber,
            found_type: value.type_(),
            found_span: *value_span,
        });
    }

    fn convert_to_homogeneous_list(list_result: ListResult<'_>) -> HomogeneousNumberList<'_> {
        match list_result {
            ListResult::Numbers { numbers, .. } => HomogeneousNumberList::Numbers(numbers),
            ListResult::MeasuredNumbers { numbers, .. } => {
                HomogeneousNumberList::MeasuredNumbers(numbers)
            }
        }
    }
}
