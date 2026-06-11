//! Standard builtin functions (e.g. min, max, sqrt).

use indexmap::IndexMap;
use oneil_output::{EvalError, Value};
use oneil_shared::{
    span::Span,
    symbols::{BuiltinFunctionName, UnitBaseName},
};

use crate::unit::BuiltinUnit;

/// Information about a builtin function.
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    /// The name of the function.
    pub name: BuiltinFunctionName,
    /// The arguments of the function.
    pub args: &'static [&'static str],
    /// The description of the function.
    pub description: &'static str,
    /// The units that the function uses.
    pub units: IndexMap<UnitBaseName, BuiltinUnit>,
    /// The function implementation.
    pub function: BuiltinFunctionFn,
}

impl BuiltinFunction {
    /// Invokes the builtin function with the given span and arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the function fails to evaluate.
    pub fn call(&self, span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        (self.function)(span, args, &self.units)
    }
}

/// Type alias for standard builtin function type.
pub type BuiltinFunctionFn = fn(
    Span,
    Vec<(Value, Span)>,
    &IndexMap<UnitBaseName, BuiltinUnit>,
) -> Result<Value, Vec<EvalError>>;

#[expect(clippy::too_many_lines, reason = "this is a list of builtin functions")]
/// Returns an iterator over all standard builtin functions.
pub fn builtin_functions_complete(
    units: &IndexMap<UnitBaseName, BuiltinUnit>,
) -> impl Iterator<Item = (BuiltinFunctionName, BuiltinFunction)> {
    [
        BuiltinFunction {
            name: BuiltinFunctionName::from("min"),
            args: &["n", "..."],
            description: fns::MIN_DESCRIPTION,
            units: get_builtin_units(fns::MIN_BUILTIN_UNITS, units),
            function: fns::min as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("max"),
            args: &["n", "..."],
            description: fns::MAX_DESCRIPTION,
            units: get_builtin_units(fns::MAX_BUILTIN_UNITS, units),
            function: fns::max as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("sin"),
            args: &["x"],
            description: fns::SIN_DESCRIPTION,
            units: get_builtin_units(fns::SIN_BUILTIN_UNITS, units),
            function: fns::sin as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("cos"),
            args: &["x"],
            description: fns::COS_DESCRIPTION,
            units: get_builtin_units(fns::COS_BUILTIN_UNITS, units),
            function: fns::cos as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("tan"),
            args: &["x"],
            description: fns::TAN_DESCRIPTION,
            units: get_builtin_units(fns::TAN_BUILTIN_UNITS, units),
            function: fns::tan as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("asin"),
            args: &["x"],
            description: fns::ASIN_DESCRIPTION,
            units: get_builtin_units(fns::ASIN_BUILTIN_UNITS, units),
            function: fns::asin as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("acos"),
            args: &["x"],
            description: fns::ACOS_DESCRIPTION,
            units: get_builtin_units(fns::ACOS_BUILTIN_UNITS, units),
            function: fns::acos as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("atan"),
            args: &["x"],
            description: fns::ATAN_DESCRIPTION,
            units: get_builtin_units(fns::ATAN_BUILTIN_UNITS, units),
            function: fns::atan as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("sqrt"),
            args: &["x"],
            description: fns::SQRT_DESCRIPTION,
            units: get_builtin_units(fns::SQRT_BUILTIN_UNITS, units),
            function: fns::sqrt as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("ln"),
            args: &["x"],
            description: fns::LN_DESCRIPTION,
            units: get_builtin_units(fns::LN_BUILTIN_UNITS, units),
            function: fns::ln as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("log2"),
            args: &["x"],
            description: fns::LOG2_DESCRIPTION,
            units: get_builtin_units(fns::LOG2_BUILTIN_UNITS, units),
            function: fns::log2 as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("log10"),
            args: &["x"],
            description: fns::LOG10_DESCRIPTION,
            units: get_builtin_units(fns::LOG10_BUILTIN_UNITS, units),
            function: fns::log10 as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("floor"),
            args: &["x"],
            description: fns::FLOOR_DESCRIPTION,
            units: get_builtin_units(fns::FLOOR_BUILTIN_UNITS, units),
            function: fns::floor as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("ceiling"),
            args: &["x"],
            description: fns::CEILING_DESCRIPTION,
            units: get_builtin_units(fns::CEILING_BUILTIN_UNITS, units),
            function: fns::ceiling as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("range"),
            args: &["x", "y?"],
            description: fns::RANGE_DESCRIPTION,
            units: get_builtin_units(fns::RANGE_BUILTIN_UNITS, units),
            function: fns::range as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("abs"),
            args: &["x"],
            description: fns::ABS_DESCRIPTION,
            units: get_builtin_units(fns::ABS_BUILTIN_UNITS, units),
            function: fns::abs as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("sign"),
            args: &["x"],
            description: fns::SIGN_DESCRIPTION,
            units: get_builtin_units(fns::SIGN_BUILTIN_UNITS, units),
            function: fns::sign as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("mid"),
            args: &["x", "y?"],
            description: fns::MID_DESCRIPTION,
            units: get_builtin_units(fns::MID_BUILTIN_UNITS, units),
            function: fns::mid as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("strip"),
            args: &["x"],
            description: fns::STRIP_DESCRIPTION,
            units: get_builtin_units(fns::STRIP_BUILTIN_UNITS, units),
            function: fns::strip as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("mnmx"),
            args: &["n", "..."],
            description: fns::MNMX_DESCRIPTION,
            units: get_builtin_units(fns::MNMX_BUILTIN_UNITS, units),
            function: fns::mnmx as BuiltinFunctionFn,
        },
        BuiltinFunction {
            name: BuiltinFunctionName::from("mxmn"),
            args: &["n", "..."],
            description: fns::MXMN_DESCRIPTION,
            units: get_builtin_units(fns::MXMN_BUILTIN_UNITS, units),
            function: fns::mxmn as BuiltinFunctionFn,
        },
    ]
    .into_iter()
    .map(|function| (function.name.clone(), function))
}

pub fn get_builtin_units(
    units: impl IntoIterator<Item = &'static str>,
    units_map: &IndexMap<UnitBaseName, BuiltinUnit>,
) -> IndexMap<UnitBaseName, BuiltinUnit> {
    units
        .into_iter()
        .map(|unit_name| {
            let unit_name = UnitBaseName::from(unit_name);
            let unit = units_map
                .get(&unit_name)
                .expect("unit should exist or else builtin is broken")
                .clone();
            (unit_name, unit)
        })
        .collect()
}

mod fns {
    use indexmap::IndexMap;
    use std::borrow::Cow;

    use oneil_output::{
        EvalError, MeasuredNumber, Number, NumberType, Value,
        error::{ExpectedArgumentCount, ExpectedType, convert::binary_eval_error_to_eval_error},
    };
    use oneil_shared::{span::Span, symbols::BuiltinFunctionName, symbols::UnitBaseName};

    use crate::unit::BuiltinUnit;

    use super::helper;

    pub const MIN_DESCRIPTION: &str = "Find the minimum value of the given values.\n\nIf a value is an interval, the minimum value of the interval is used.";
    pub const MIN_BUILTIN_UNITS: [&str; 0] = [];

    #[expect(
        clippy::needless_pass_by_value,
        reason = "matches the expected signature"
    )]
    pub fn min(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: BuiltinFunctionName::from("min"),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::AtLeast(1),
                actual_argument_count: args.len(),
            }]);
        }

        let number_list = helper::extract_homogeneous_numbers_list(&args)?;

        match number_list {
            helper::HomogeneousNumberList::Numbers(numbers) => numbers
                .into_iter()
                .filter_map(|number| match number.into_owned() {
                    Number::Scalar(value) => Some(value),
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
    pub const MAX_BUILTIN_UNITS: [&str; 0] = [];

    #[expect(
        clippy::needless_pass_by_value,
        reason = "matches the expected signature"
    )]
    pub fn max(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: BuiltinFunctionName::from("max"),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::AtLeast(1),
                actual_argument_count: args.len(),
            }]);
        }

        let number_list = helper::extract_homogeneous_numbers_list(&args)?;

        match number_list {
            helper::HomogeneousNumberList::Numbers(numbers) => numbers
                .into_iter()
                .filter_map(|number| match number.into_owned() {
                    Number::Scalar(value) => Some(value),
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
    pub const SIN_BUILTIN_UNITS: [&str; 1] = ["rad"];

    pub fn sin(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        let rad = builtins
            .get(&UnitBaseName::from("rad"))
            .expect("rad unit should exist");

        helper::unary_measured_number_fn(identifier_span, args, "sin", |m, arg_span| {
            let number = helper::measured_number_into_number_using_unit(m, &rad.unit, arg_span)?;
            Ok(Value::Number(number.sin()))
        })
    }

    pub const COS_DESCRIPTION: &str = "Compute the cosine of an angle.";
    pub const COS_BUILTIN_UNITS: [&str; 1] = ["rad"];

    pub fn cos(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        let rad = builtins
            .get(&UnitBaseName::from("rad"))
            .expect("rad unit should exist");

        helper::unary_measured_number_fn(identifier_span, args, "cos", |m, arg_span| {
            let number = helper::measured_number_into_number_using_unit(m, &rad.unit, arg_span)?;
            Ok(Value::Number(number.cos()))
        })
    }

    pub const TAN_DESCRIPTION: &str = "Compute the tangent of an angle.";
    pub const TAN_BUILTIN_UNITS: [&str; 1] = ["rad"];

    pub fn tan(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        let rad = builtins
            .get(&UnitBaseName::from("rad"))
            .expect("rad unit should exist");

        helper::unary_measured_number_fn(identifier_span, args, "tan", |m, arg_span| {
            let number = helper::measured_number_into_number_using_unit(m, &rad.unit, arg_span)?;
            Ok(Value::Number(number.tan()))
        })
    }

    pub const ASIN_DESCRIPTION: &str =
        "Compute the arcsine (inverse sine) of a value, returning an angle.";
    pub const ASIN_BUILTIN_UNITS: [&str; 1] = ["rad"];

    pub fn asin(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        let rad = builtins
            .get(&UnitBaseName::from("rad"))
            .expect("rad unit should exist");

        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "asin",
            |n, _arg_span| {
                let number = n.asin();
                let unit = rad.unit.clone();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
            |m, arg_span| {
                let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
                let number = number.asin();
                let unit = rad.unit.clone();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
        )
    }

    pub const ACOS_DESCRIPTION: &str =
        "Compute the arccosine (inverse cosine) of a value, returning an angle.";
    pub const ACOS_BUILTIN_UNITS: [&str; 1] = ["rad"];

    pub fn acos(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        let rad = builtins
            .get(&UnitBaseName::from("rad"))
            .expect("rad unit should exist");

        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "acos",
            |n, _arg_span| {
                let number = n.acos();
                let unit = rad.unit.clone();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
            |m, arg_span| {
                let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
                let number = number.acos();
                let unit = rad.unit.clone();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
        )
    }

    pub const ATAN_DESCRIPTION: &str =
        "Compute the arctangent (inverse tangent) of a value, returning an angle.";
    pub const ATAN_BUILTIN_UNITS: [&str; 1] = ["rad"];

    pub fn atan(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        let rad = builtins
            .get(&UnitBaseName::from("rad"))
            .expect("rad unit should exist");

        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "atan",
            |n, _arg_span| {
                let number = n.atan();
                let unit = rad.unit.clone();

                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
            |m, arg_span| {
                let number = helper::dimensionless_measured_number_as_number(m, arg_span)?;
                let number = number.atan();
                let unit = rad.unit.clone();
                let measured_number = MeasuredNumber::from_number_and_unit(number, unit);

                Ok(Value::MeasuredNumber(measured_number))
            },
        )
    }

    pub const SQRT_DESCRIPTION: &str = "Compute the square root of a value.";
    pub const SQRT_BUILTIN_UNITS: [&str; 0] = [];

    pub fn sqrt(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "sqrt",
            |n, _arg_span| Ok(Value::Number(n.sqrt())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.sqrt())),
        )
    }

    pub const LN_DESCRIPTION: &str = "Compute the natural logarithm (base e) of a value.";
    pub const LN_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the natural logarithm (base e) of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn ln(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "ln",
            |n, _arg_span| Ok(Value::Number(n.ln())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.ln())),
        )
    }

    pub const LOG10_DESCRIPTION: &str = "Compute the base-10 logarithm of a value.";
    pub const LOG10_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the base-10 logarithm of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn log10(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "log10",
            |n, _arg_span| Ok(Value::Number(n.log10())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.log10())),
        )
    }

    pub const LOG2_DESCRIPTION: &str = "Compute the base-2 logarithm of a value.";
    pub const LOG2_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the base-2 logarithm of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn log2(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "log2",
            |n, _arg_span| Ok(Value::Number(n.log2())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.log2())),
        )
    }

    pub const FLOOR_DESCRIPTION: &str = "Round a value down to the nearest integer.";
    pub const FLOOR_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the single numerical argument rounded down to the nearest integer.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn floor(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_number_or_measured_number_fn(
            identifier_span,
            args,
            "floor",
            |n, _arg_span| Ok(Value::Number(n.floor())),
            |m, _arg_span| Ok(Value::MeasuredNumber(m.floor())),
        )
    }

    pub const CEILING_DESCRIPTION: &str = "Round a value up to the nearest integer.";
    pub const CEILING_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the single numerical argument rounded up to the nearest integer.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn ceiling(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
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
    pub const RANGE_BUILTIN_UNITS: [&str; 0] = [];

    pub fn range(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
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
                            expected_type: ExpectedType::NumberOrMeasuredNumber {
                                number_type: None,
                            },
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
                function_name: BuiltinFunctionName::from("range"),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Between(1, 2),
                actual_argument_count: args.len(),
            }]),
        }
    }

    pub const ABS_DESCRIPTION: &str = "Compute the absolute value of a number.";
    pub const ABS_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the absolute value of the single numerical argument.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn abs(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
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
    pub const SIGN_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the sign of the single numerical argument (-1, 0, or 1).
    ///
    /// For measured numbers, the unit is dropped since sign is dimensionless.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn sign(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
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
    pub const MID_BUILTIN_UNITS: [&str; 0] = [];

    pub fn mid(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
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
                            expected_type: ExpectedType::NumberOrMeasuredNumber {
                                number_type: None,
                            },
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
                function_name: BuiltinFunctionName::from("mid"),
                function_name_span: identifier_span,
                expected_argument_count: ExpectedArgumentCount::Between(1, 2),
                actual_argument_count: args.len(),
            }]),
        }
    }

    pub const STRIP_DESCRIPTION: &str =
        "Strip units from a measured number, returning just the numeric value.";
    pub const STRIP_BUILTIN_UNITS: [&str; 0] = [];

    /// Strips units from a measured number, returning the numeric value.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the argument count is not exactly one, or if the
    /// argument is not a number or measured number.
    pub fn strip(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        helper::unary_measured_number_fn(identifier_span, args, "strip", |m, _arg_span| {
            Ok(Value::Number(m.into_number_and_unit().0))
        })
    }

    pub const MNMX_DESCRIPTION: &str =
        "Return both the minimum and maximum values from the given values.";
    pub const MNMX_BUILTIN_UNITS: [&str; 0] = [];

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
    pub fn mnmx(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: BuiltinFunctionName::from("mnmx"),
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

    pub const MXMN_DESCRIPTION: &str = "Return the intersection of the given values.\n\nThe minimum is the maximum of all value minimums, and the maximum is the minimum of all value maximums. If the resulting minimum is greater than the maximum, returns an empty interval.";
    pub const MXMN_BUILTIN_UNITS: [&str; 0] = [];

    /// Returns the intersection of all given values (max of mins, min of maxes).
    ///
    /// # Errors
    ///
    /// Returns `Err` if no arguments are given, or if arguments are not
    /// homogeneous numbers or measured numbers.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "matches the expected signature"
    )]
    pub fn mxmn(
        identifier_span: Span,
        args: Vec<(Value, Span)>,
        _builtins: &IndexMap<UnitBaseName, BuiltinUnit>,
    ) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount {
                function_name: BuiltinFunctionName::from("mxmn"),
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
                    .reduce(Number::intersection)
                    .expect("there should be at least one number");

                Ok(Value::Number(result))
            }
            helper::HomogeneousNumberList::MeasuredNumbers(numbers) => {
                let result = numbers
                    .into_iter()
                    .map(Cow::into_owned)
                    .reduce(|a, b| {
                        a.checked_intersection(&b)
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

    use oneil_output::{
        EvalError, MeasuredNumber, Number, Unit, Value, ValueType,
        error::{ExpectedArgumentCount, ExpectedType},
    };
    use oneil_shared::{span::Span, symbols::BuiltinFunctionName};

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
                function_name: BuiltinFunctionName::from(name),
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
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
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
                function_name: BuiltinFunctionName::from(name),
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
                    expected_type: ExpectedType::MeasuredNumber {
                        number_type: None,
                        unit: None,
                    },
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

    /// Converts a measured number to a raw number expressed in the given unit.
    ///
    /// Returns an error if the measured number's unit is not dimensionally
    /// equivalent to the target unit.
    pub fn measured_number_into_number_using_unit(
        measured: MeasuredNumber,
        unit: &Unit,
        argument_span: Span,
    ) -> Result<Number, Vec<EvalError>> {
        if !measured.unit().dimensionally_eq(unit) {
            return Err(vec![EvalError::InvalidUnit {
                expected_unit: Some(unit.display_unit.clone()),
                found_unit: Some(measured.unit().display_unit.clone()),
                found_span: argument_span,
            }]);
        }

        Ok(measured.into_number_using_unit(unit))
    }

    #[derive(Debug)]
    pub enum HomogeneousNumberList<'input> {
        /// Numbers (including effectively unitless measured numbers converted to numbers).
        Numbers(Vec<Cow<'input, Number>>),
        /// Measured numbers.
        MeasuredNumbers(Vec<Cow<'input, MeasuredNumber>>),
    }

    #[derive(Debug)]
    enum ListResult<'input> {
        Numbers {
            numbers: Vec<Cow<'input, Number>>,
            first_number_span: &'input Span,
        },
        MeasuredNumbers {
            numbers: Vec<Cow<'input, MeasuredNumber>>,
            expected_unit: &'input Unit,
            expected_unit_value_span: &'input Span,
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

        // iterate over the values and collect them into a homogeneous list
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
                    numbers.push(Cow::Borrowed(number));
                } else {
                    errors.push(EvalError::UnitMismatch {
                        expected_unit: expected_unit.display_unit.clone(),
                        expected_source_span: (**expected_unit_value_span).clone(),
                        found_unit: number.unit().display_unit.clone(),
                        found_span: value_span.clone(),
                    });
                }
            }
            Some(ListResult::Numbers {
                numbers,
                first_number_span,
            }) => {
                if number.is_dimensionless() {
                    let number = number.clone().into_number_using_unit(&Unit::one());
                    numbers.push(Cow::Owned(number));
                } else {
                    errors.push(EvalError::TypeMismatch {
                        expected_type: ExpectedType::Number { number_type: None },
                        expected_source_span: (**first_number_span).clone(),
                        found_type: ValueType::MeasuredNumber {
                            number_type: number.normalized_value().type_(),
                            unit: number.unit().clone(),
                        },
                        found_span: value_span.clone(),
                    });
                }
            }
            None => {
                let number_unit = number.unit();

                *list_result = Some(ListResult::MeasuredNumbers {
                    numbers: vec![Cow::Borrowed(number)],
                    expected_unit: number_unit,
                    expected_unit_value_span: value_span,
                });
            }
        }
    }

    fn handle_number<'input>(
        number: &'input Number,
        value_span: &'input Span,
        list_result: &mut Option<ListResult<'input>>,
        errors: &mut Vec<EvalError>,
    ) {
        match list_result {
            Some(ListResult::MeasuredNumbers {
                numbers,
                expected_unit,
                expected_unit_value_span,
            }) => {
                if expected_unit.is_dimensionless() {
                    let number = MeasuredNumber::from_number_and_unit(*number, Unit::one());
                    numbers.push(Cow::Owned(number));
                } else {
                    errors.push(EvalError::TypeMismatch {
                        expected_type: ExpectedType::MeasuredNumber {
                            number_type: None,
                            unit: Some(expected_unit.display_unit.clone()),
                        },
                        expected_source_span: (**expected_unit_value_span).clone(),
                        found_type: ValueType::Number {
                            number_type: number.type_(),
                        },
                        found_span: value_span.clone(),
                    });
                }
            }
            Some(ListResult::Numbers { numbers, .. }) => {
                numbers.push(Cow::Borrowed(number));
            }
            None => {
                *list_result = Some(ListResult::Numbers {
                    numbers: vec![Cow::Borrowed(number)],
                    first_number_span: value_span,
                });
            }
        }
    }

    fn handle_invalid_type(value: &Value, value_span: &Span, errors: &mut Vec<EvalError>) {
        errors.push(EvalError::InvalidType {
            expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
            found_type: value.type_(),
            found_span: value_span.clone(),
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
