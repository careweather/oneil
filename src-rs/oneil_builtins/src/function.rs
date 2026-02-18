//! Standard builtin functions (e.g. min, max, sqrt).

use oneil_eval::EvalError;
use oneil_output::Value;
use oneil_shared::span::Span;

/// Information about a builtin function.
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: &'static str,
    pub args: &'static [&'static str],
    pub description: &'static str,
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
            name: "log",
            args: &["x"],
            description: fns::LOG_DESCRIPTION,
            function: fns::log as BuiltinFunctionFn,
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
            name: "extent",
            args: &["x"],
            description: fns::EXTENT_DESCRIPTION,
            function: fns::extent as BuiltinFunctionFn,
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
    use oneil_shared::span::Span;

    use oneil_eval::{
        EvalError,
        error::{
            ExpectedArgumentCount, ExpectedType,
            convert::{binary_eval_error_expect_only_lhs, binary_eval_error_to_eval_error},
        },
    };
    use oneil_output::{DisplayUnit, MeasuredNumber, Number, NumberType, Unit, Value};

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

        let number_list = extract_homogeneous_numbers_list(&args)?;

        match number_list {
            HomogeneousNumberList::Numbers(numbers) => {
                let min = numbers
                    .into_iter()
                    .filter_map(|number| match number {
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
                    .expect("there should be at least one number");

                Ok(Value::Number(Number::Scalar(min)))
            }
            HomogeneousNumberList::MeasuredNumbers(numbers) => {
                let min = numbers
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
                        if a.normalized_value() < b.normalized_value() {
                            a
                        } else {
                            b
                        }
                    })
                    .expect("there should be at least one number");

                Ok(Value::MeasuredNumber(min))
            }
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

        let number_list = extract_homogeneous_numbers_list(&args)?;

        match number_list {
            HomogeneousNumberList::Numbers(numbers) => {
                let max = numbers
                    .into_iter()
                    .filter_map(|number| match number {
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
                    .expect("there should be at least one number");

                Ok(Value::Number(Number::Scalar(max)))
            }
            HomogeneousNumberList::MeasuredNumbers(numbers) => {
                let max = numbers
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
                        if a.normalized_value() > b.normalized_value() {
                            a
                        } else {
                            b
                        }
                    })
                    .expect("there should be at least one number");

                Ok(Value::MeasuredNumber(max))
            }
        }
    }

    pub const SIN_DESCRIPTION: &str = "Compute the sine of an angle in radians.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn sin(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("sin".to_string()),
            will_be_supported: true,
        }])
    }

    pub const COS_DESCRIPTION: &str = "Compute the cosine of an angle in radians.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn cos(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("cos".to_string()),
            will_be_supported: true,
        }])
    }

    pub const TAN_DESCRIPTION: &str = "Compute the tangent of an angle in radians.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn tan(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("tan".to_string()),
            will_be_supported: true,
        }])
    }

    pub const ASIN_DESCRIPTION: &str =
        "Compute the arcsine (inverse sine) of a value, returning an angle in radians.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn asin(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("asin".to_string()),
            will_be_supported: true,
        }])
    }

    pub const ACOS_DESCRIPTION: &str =
        "Compute the arccosine (inverse cosine) of a value, returning an angle in radians.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn acos(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("acos".to_string()),
            will_be_supported: true,
        }])
    }

    pub const ATAN_DESCRIPTION: &str =
        "Compute the arctangent (inverse tangent) of a value, returning an angle in radians.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn atan(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("atan".to_string()),
            will_be_supported: true,
        }])
    }

    pub const SQRT_DESCRIPTION: &str = "Compute the square root of a value.";

    pub fn sqrt(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        if args.len() != 1 {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: "sqrt".to_string(),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Exact(1),
                actual_argument_count: args.len(),
            }]);
        }

        let mut args = args.into_iter();

        let (arg, arg_span) = args.next().expect("there should be one argument");

        arg.checked_pow(Value::from(0.5))
            .map_err(|error| vec![binary_eval_error_expect_only_lhs(error, arg_span)])
    }

    pub const LN_DESCRIPTION: &str = "Compute the natural logarithm (base e) of a value.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn ln(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("ln".to_string()),
            will_be_supported: true,
        }])
    }

    pub const LOG_DESCRIPTION: &str = "Compute the logarithm of a value.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn log(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("log".to_string()),
            will_be_supported: true,
        }])
    }

    pub const LOG10_DESCRIPTION: &str = "Compute the base-10 logarithm of a value.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn log10(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("log10".to_string()),
            will_be_supported: true,
        }])
    }

    pub const FLOOR_DESCRIPTION: &str = "Round a value down to the nearest integer.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn floor(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("floor".to_string()),
            will_be_supported: true,
        }])
    }

    pub const CEILING_DESCRIPTION: &str = "Round a value up to the nearest integer.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn ceiling(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("ceiling".to_string()),
            will_be_supported: true,
        }])
    }

    pub const EXTENT_DESCRIPTION: &str = "Compute the extent of a value.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn extent(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("extent".to_string()),
            will_be_supported: true,
        }])
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

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn abs(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("abs".to_string()),
            will_be_supported: true,
        }])
    }

    pub const SIGN_DESCRIPTION: &str =
        "Compute the sign of a number, returning -1 for negative, 0 for zero, or 1 for positive.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn sign(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("sign".to_string()),
            will_be_supported: true,
        }])
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

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn strip(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("strip".to_string()),
            will_be_supported: true,
        }])
    }

    pub const MNMX_DESCRIPTION: &str =
        "Return both the minimum and maximum values from the given values.";

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn mnmx(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("mnmx".to_string()),
            will_be_supported: true,
        }])
    }

    /// Homogeneous number list helpers
    enum HomogeneousNumberList<'a> {
        Numbers(Vec<&'a Number>),
        MeasuredNumbers(Vec<&'a MeasuredNumber>),
    }

    enum ListResult<'a> {
        Numbers {
            numbers: Vec<&'a Number>,
            first_number_span: &'a Span,
        },
        MeasuredNumbers {
            numbers: Vec<&'a MeasuredNumber>,
            expected_unit: &'a Unit,
            expected_unit_value_span: &'a Span,
        },
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "callers enforce non-empty list to provide correct error message"
    )]
    fn extract_homogeneous_numbers_list(
        values: &[(Value, Span)],
    ) -> Result<HomogeneousNumberList<'_>, Vec<EvalError>> {
        assert!(!values.is_empty());
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
                    expected_unit: DisplayUnit::Unitless,
                    expected_source_span: **first_number_span,
                    found_unit: number.unit().display_unit.clone(),
                    found_span: *value_span,
                });
            }
            None => {
                *list_result = Some(ListResult::MeasuredNumbers {
                    numbers: vec![number],
                    expected_unit: number.unit(),
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
                    found_unit: DisplayUnit::Unitless,
                    found_span: *value_span,
                });
            }
            Some(ListResult::Numbers { numbers, .. }) => {
                numbers.push(number);
            }
            None => {
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
