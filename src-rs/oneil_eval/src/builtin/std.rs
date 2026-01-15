//! The standard builtin values, functions, units, and prefixes
//! that come with Oneil.

use ::std::collections::HashMap;
use std::sync::Arc;

use crate::{
    EvalError,
    value::{Dimension, MeasuredNumber, Number, SizedUnit, Unit, Value},
};

/// The builtin values that come with Oneil:
/// - `pi` - the mathematical constant π
/// - `e` - the mathematical constant e
#[must_use]
pub fn builtin_values() -> HashMap<String, Value> {
    HashMap::from([
        (
            "pi".to_string(),
            Value::Number(MeasuredNumber::new(
                Number::Scalar(std::f64::consts::PI),
                Unit::unitless(),
            )),
        ),
        (
            "e".to_string(),
            Value::Number(MeasuredNumber::new(
                Number::Scalar(std::f64::consts::E),
                Unit::unitless(),
            )),
        ),
    ])
}

/// The builtin unit prefixes that come with Oneil:
/// - `q` - quecto
/// - `r` - ronto
/// - `y` - yocto
/// - `z` - zepto
/// - `a` - atto
/// - `f` - femto
/// - `p` - pico
/// - `n` - nano
/// - `u` - micro
/// - `m` - milli
/// - `k` - kilo
/// - `M` - mega
/// - `G` - giga
/// - `T` - tera
/// - `P` - peta
/// - `E` - exa
/// - `Z` - zetta
/// - `Y` - yotta
/// - `R` - ronna
/// - `Q` - quetta
#[must_use]
pub fn builtin_prefixes() -> HashMap<String, f64> {
    HashMap::from(
        [
            ("q", 1e-30), // quecto
            ("r", 1e-27), // ronto
            ("y", 1e-24), // yocto
            ("z", 1e-21), // zepto
            ("a", 1e-18), // atto
            ("f", 1e-15), // femto
            ("p", 1e-12), // pico
            ("n", 1e-9),  // nano
            ("u", 1e-6),  // micro
            ("m", 1e-3),  // milli
            ("k", 1e3),   // kilo
            ("M", 1e6),   // mega
            ("G", 1e9),   // giga
            ("T", 1e12),  // tera
            ("P", 1e15),  // peta
            ("E", 1e18),  // exa
            ("Z", 1e21),  // zetta
            ("Y", 1e24),  // yotta
            ("R", 1e27),  // ronna
            ("Q", 1e30),  // quetta
        ]
        .map(|(k, v)| (k.to_string(), v)),
    )
}

/// The builtin units that come with Oneil.
///
/// Base units:
/// - `g` - gram
/// - `m` - meter
/// - `s` - second
/// - `K` - Kelvin
/// - `A` - Ampere
/// - `b` - bit
/// - `$` - dollar
/// - `mol` - mole
/// - `cd` - candela
///
/// Derived units:
/// - `V` - Volt
/// - `W` - Watt
/// - `Hz` - Hertz
/// - `J` - Joule
/// - `Wh` - Watt-hour
/// - `Ah` - Amp-hour
/// - `T` - Tesla
/// - `Ohm` - Ohm
/// - `N` - Newton
/// - `Gs` - Gauss
/// - `lm` - lumen
/// - `lx` - lux
/// - `bps` - bits per second
/// - `B` - byte
/// - `Pa` - Pascal
///
/// Legacy units:
/// - `mil` - millennium
/// - `cen` - century
/// - `dec` - decade
/// - `yr` - year
/// - `mon` - month
/// - `week` - week
/// - `day` - day
/// - `hr` - hour
/// - `min` - minute
/// - `rpm` - revolutions per minute
/// - `k$` - thousand dollars
/// - `M$` - million dollars
/// - `B$` - billion dollars
/// - `T$` - trillion dollars
/// - `g_E` - Earth gravity
/// - `cm` - centimeter
/// - `psi` - pound per square inch
/// - `kpsi` - kilopound per square inch
/// - `atm` - atmosphere
/// - `bar` - bar
/// - `Ba` - barye
/// - `dyne` - dyne
/// - `mmHg` - millimeter of mercury
/// - `torr` - torr
/// - `in` - inch
/// - `ft` - foot
/// - `yd` - yard
/// - `mi` - mile
/// - `nmi` - nautical mile
/// - `lb` - pound
/// - `mph` - mile per hour
///
/// Dimensionless units:
/// - `rev` - revolution
/// - `cyc` - cycle
/// - `rad` - radian
/// - `deg` - degree
/// - `%` - percent
/// - `ppm` - part per million
/// - `ppb` - part per billion
/// - `arcmin` - arcminute
/// - `arcsec` - arcsecond
#[expect(clippy::too_many_lines, reason = "this is a list of builtin units")]
#[expect(clippy::unreadable_literal, reason = "this is a list of builtin units")]
#[must_use]
pub fn builtin_units() -> HashMap<String, Arc<SizedUnit>> {
    let units = [
        // === BASE UNITS ===
        (
            ["g", "gram", "grams"].as_ref(),
            SizedUnit {
                // the kilogram is the base unit of mass, so the gram is 1e-3 of a kilogram
                magnitude: 1e-3,
                unit: Unit::new(HashMap::from([(Dimension::Mass, 1.0)])),
            },
        ),
        (
            ["m", "meter", "meters", "metre", "metres"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["s", "second", "seconds", "sec", "secs"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["K", "Kelvin"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Temperature, 1.0)])),
            },
        ),
        (
            ["A", "Ampere", "Amp"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Current, 1.0)])),
            },
        ),
        (
            ["b", "bit", "bits"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Information, 1.0)])),
            },
        ),
        (
            ["$", "dollar", "dollars"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Currency, 1.0)])),
            },
        ),
        (
            ["mol", "mole", "moles"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::Substance, 1.0)])),
            },
        ),
        (
            ["cd", "candela"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::LuminousIntensity, 1.0)])),
            },
        ),
        // === DERIVED UNITS ===
        (
            ["V", "Volt", "Volts"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -1.0),
                ])),
            },
        ),
        (
            ["W", "Watt", "Watts"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                ])),
            },
        ),
        (
            ["Hz", "Hertz"].as_ref(),
            SizedUnit {
                magnitude: 2.0 * std::f64::consts::PI,
                unit: Unit::new(HashMap::from([(Dimension::Time, -1.0)])),
            },
        ),
        (
            ["J", "Joule", "Joules"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["Wh", "Watt-hour", "Watt-hours"].as_ref(),
            SizedUnit {
                magnitude: 3600.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["Ah", "Amp-hour", "Amp-hours"].as_ref(),
            SizedUnit {
                magnitude: 3600.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Current, 1.0),
                    (Dimension::Time, 1.0),
                ])),
            },
        ),
        (
            ["T", "Tesla", "Teslas"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Time, -2.0),
                    (Dimension::Current, -1.0),
                ])),
            },
        ),
        (
            ["Ohm", "Ohms"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 2.0),
                    (Dimension::Time, -3.0),
                    (Dimension::Current, -2.0),
                ])),
            },
        ),
        (
            ["N", "Newton", "Newtons"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["Gs", "Gauss"].as_ref(),
            SizedUnit {
                magnitude: 0.0001,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Time, -2.0),
                    (Dimension::Current, -1.0),
                ])),
            },
        ),
        (
            ["lm", "Lumen", "Lumens"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([(Dimension::LuminousIntensity, 1.0)])),
            },
        ),
        (
            ["lx", "Lux", "Luxes"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::LuminousIntensity, 1.0),
                    (Dimension::Distance, -2.0),
                ])),
            },
        ),
        (
            ["bps" /* bits per second */].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Information, 1.0),
                    (Dimension::Time, -1.0),
                ])),
            },
        ),
        (
            ["B", "byte", "bytes"].as_ref(),
            SizedUnit {
                magnitude: 8.0,
                unit: Unit::new(HashMap::from([(Dimension::Information, 1.0)])),
            },
        ),
        (
            ["Pa", "Pascal", "Pascals"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        // === LEGACY UNITS ===
        (
            ["mil", "millennium", "millennia"].as_ref(),
            SizedUnit {
                magnitude: 3.1556952e10,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["cen", "century", "centuries"].as_ref(),
            SizedUnit {
                magnitude: 3.1556952e9,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["dec", "decade", "decades"].as_ref(),
            SizedUnit {
                magnitude: 3.1556952e8,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["yr", "year", "years"].as_ref(),
            SizedUnit {
                magnitude: 3.1556952e7,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["mon", "month", "months"].as_ref(),
            SizedUnit {
                magnitude: 2.629746e6,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["week", "weeks"].as_ref(),
            SizedUnit {
                magnitude: 6.048e5,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["day", "days"].as_ref(),
            SizedUnit {
                magnitude: 8.64e4,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["hr", "hour", "hours"].as_ref(),
            SizedUnit {
                magnitude: 3600.0,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["min", "minute", "minutes"].as_ref(),
            SizedUnit {
                magnitude: 60.0,
                unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
            },
        ),
        (
            ["rpm" /* revolutions per minute */].as_ref(),
            SizedUnit {
                magnitude: 0.10471975511965977,
                unit: Unit::new(HashMap::from([(Dimension::Time, -1.0)])),
            },
        ),
        (
            ["k$" /* thousand dollars */].as_ref(),
            SizedUnit {
                magnitude: 1000.0,
                unit: Unit::new(HashMap::from([(Dimension::Currency, 1.0)])),
            },
        ),
        (
            ["M$" /* million dollars */].as_ref(),
            SizedUnit {
                magnitude: 1e6,
                unit: Unit::new(HashMap::from([(Dimension::Currency, 1.0)])),
            },
        ),
        (
            ["B$" /* billion dollars */].as_ref(),
            SizedUnit {
                magnitude: 1e9,
                unit: Unit::new(HashMap::from([(Dimension::Currency, 1.0)])),
            },
        ),
        (
            ["T$" /* trillion dollars */].as_ref(),
            SizedUnit {
                magnitude: 1e12,
                unit: Unit::new(HashMap::from([(Dimension::Currency, 1.0)])),
            },
        ),
        (
            ["g_E" /* Earth gravity */].as_ref(),
            SizedUnit {
                magnitude: 9.81,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            [
                "cm",
                "centimeter",
                "centimeters",
                "centimetre",
                "centimetres",
            ]
            .as_ref(),
            SizedUnit {
                magnitude: 0.01,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["psi" /* pounds per square inch */].as_ref(),
            SizedUnit {
                magnitude: 6894.757293168361,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["atm", "atmosphere", "atmospheres"].as_ref(),
            SizedUnit {
                magnitude: 101325.0,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["bar", "bars"].as_ref(),
            SizedUnit {
                magnitude: 1e5,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["Ba", "barye", "baryes"].as_ref(),
            SizedUnit {
                magnitude: 0.1,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["dyne", "dynes"].as_ref(),
            SizedUnit {
                magnitude: 1e-5,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, 1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["mmHg" /* millimeter of mercury */].as_ref(),
            SizedUnit {
                magnitude: 133.322387415,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["torr", "torrs"].as_ref(),
            SizedUnit {
                magnitude: 133.3224,
                unit: Unit::new(HashMap::from([
                    (Dimension::Mass, 1.0),
                    (Dimension::Distance, -1.0),
                    (Dimension::Time, -2.0),
                ])),
            },
        ),
        (
            ["in", "inch", "inches"].as_ref(),
            SizedUnit {
                magnitude: 0.0254,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["ft", "foot", "feet"].as_ref(),
            SizedUnit {
                magnitude: 0.3048,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["yd", "yard", "yards"].as_ref(),
            SizedUnit {
                magnitude: 0.9144,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["mi", "mile", "miles"].as_ref(),
            SizedUnit {
                magnitude: 1609.344,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["nmi" /* nautical mile */].as_ref(),
            SizedUnit {
                magnitude: 1852.0,
                unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
            },
        ),
        (
            ["lb", "lbs", "pound", "pounds"].as_ref(),
            SizedUnit {
                magnitude: 0.45359237,
                unit: Unit::new(HashMap::from([(Dimension::Mass, 1.0)])),
            },
        ),
        (
            ["mph" /* mile per hour */].as_ref(),
            SizedUnit {
                magnitude: 0.44704,
                unit: Unit::new(HashMap::from([
                    (Dimension::Distance, 1.0),
                    (Dimension::Time, -1.0),
                ])),
            },
        ),
        // === DIMENSIONLESS UNITS ===
        (
            ["rev", "revolution", "revolutions", "rotation", "rotations"].as_ref(),
            SizedUnit {
                // a revolution is 2π radians
                magnitude: 2.0 * std::f64::consts::PI,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["cyc", "cycle", "cycles"].as_ref(),
            SizedUnit {
                // a cycle is 2π radians
                magnitude: 2.0 * std::f64::consts::PI,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["rad", "radian", "radians"].as_ref(),
            SizedUnit {
                magnitude: 1.0,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["deg", "degree", "degrees"].as_ref(),
            SizedUnit {
                magnitude: 0.017453292519943295,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["%", "percent"].as_ref(),
            SizedUnit {
                magnitude: 0.01,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["ppm" /* part per million */].as_ref(),
            SizedUnit {
                magnitude: 1e-6,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["ppb" /* part per billion */].as_ref(),
            SizedUnit {
                magnitude: 1e-9,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["arcmin", "arcminute", "arcminutes"].as_ref(),
            SizedUnit {
                magnitude: 0.0002908882086657216,
                unit: Unit::new(HashMap::from([])),
            },
        ),
        (
            ["arcsec", "arcsecond", "arcseconds"].as_ref(),
            SizedUnit {
                magnitude: 4.84813681109536e-06,
                unit: Unit::new(HashMap::from([])),
            },
        ),
    ];

    units
        .into_iter()
        .flat_map(|(names, unit)| {
            let unit = Arc::new(unit);
            names
                .iter()
                .map(move |name| ((*name).to_string(), Arc::clone(&unit)))
        })
        .collect()
}

/// Type alias for standard builtin function type
pub type StdBuiltinFunction = fn(Vec<Value>) -> Result<Value, Vec<EvalError>>;

/// The builtin functions that come with Oneil:
/// - `min` - minimum
/// - `max` - maximum
/// - `sin` - sine
/// - `cos` - cosine
/// - `tan` - tangent
/// - `asin` - arcsine
/// - `acos` - arccosine
/// - `atan` - arctangent
/// - `sqrt` - square root
/// - `ln` - natural logarithm
/// - `log` - logarithm
/// - `log10` - base 10 logarithm
/// - `floor` - floor
/// - `ceiling` - ceiling
/// - `extent` - extent
/// - `range` - range
/// - `abs` - absolute value
/// - `sign` - sign
/// - `mid` - midpoint
/// - `strip` - strip units
/// - `mnmx` - minimum and maximum
///
/// Note that some of these functions are not yet implemented and
/// will return an `EvalError::Unsupported` error when called. However,
/// we plan to implement them in the future.
#[must_use]
pub fn builtin_functions() -> HashMap<String, StdBuiltinFunction> {
    HashMap::from(
        [
            ("min", fns::min as StdBuiltinFunction),
            ("max", fns::max as StdBuiltinFunction),
            ("sin", fns::sin as StdBuiltinFunction),
            ("cos", fns::cos as StdBuiltinFunction),
            ("tan", fns::tan as StdBuiltinFunction),
            ("asin", fns::asin as StdBuiltinFunction),
            ("acos", fns::acos as StdBuiltinFunction),
            ("atan", fns::atan as StdBuiltinFunction),
            ("sqrt", fns::sqrt as StdBuiltinFunction),
            ("ln", fns::ln as StdBuiltinFunction),
            ("log", fns::log as StdBuiltinFunction),
            ("log10", fns::log10 as StdBuiltinFunction),
            ("floor", fns::floor as StdBuiltinFunction),
            ("ceiling", fns::ceiling as StdBuiltinFunction),
            ("extent", fns::extent as StdBuiltinFunction),
            ("range", fns::range as StdBuiltinFunction),
            ("abs", fns::abs as StdBuiltinFunction),
            ("sign", fns::sign as StdBuiltinFunction),
            ("mid", fns::mid as StdBuiltinFunction),
            ("strip", fns::strip as StdBuiltinFunction),
            ("mnmx", fns::mnmx as StdBuiltinFunction),
        ]
        .map(|(k, v)| (k.to_string(), v)),
    )
}

mod fns {
    use crate::{
        EvalError,
        value::{MeasuredNumber, Number, Value},
    };

    pub fn min(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount]);
        }

        let mut numbers = Vec::new();
        let mut errors = Vec::new();

        let mut unit = None;

        for arg in args {
            match arg {
                Value::Number(number) => {
                    if let Some(ref unit) = unit {
                        if &number.unit != unit {
                            errors.push(EvalError::InvalidUnit);
                            continue;
                        }
                    } else {
                        unit = Some(number.unit.clone());
                    }

                    numbers.push(number);
                }
                Value::String(_) | Value::Boolean(_) => errors.push(EvalError::InvalidType),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let unit =
            unit.expect("there should be at least one number, and that number should have a unit");

        let number_values = numbers.into_iter().filter_map(|number| match number.value {
            Number::Scalar(value) => Some(value),
            Number::Interval(interval) => {
                if interval.is_empty() {
                    None
                } else {
                    Some(interval.min())
                }
            }
        });

        let min = number_values.reduce(f64::min);

        min.map_or_else(
            || Err(vec![EvalError::NoNonEmptyValue]),
            |min| {
                Ok(Value::Number(MeasuredNumber::new(
                    Number::Scalar(min),
                    unit,
                )))
            },
        )
    }

    pub fn max(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        if args.is_empty() {
            return Err(vec![EvalError::InvalidArgumentCount]);
        }

        let mut numbers = Vec::new();
        let mut errors = Vec::new();

        let mut unit = None;

        for arg in args {
            match arg {
                Value::Number(number) => {
                    if let Some(ref unit) = unit {
                        if &number.unit != unit {
                            errors.push(EvalError::InvalidUnit);
                            continue;
                        }
                    } else {
                        unit = Some(number.unit.clone());
                    }

                    numbers.push(number);
                }
                Value::String(_) | Value::Boolean(_) => errors.push(EvalError::InvalidType),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let unit =
            unit.expect("there should be at least one number, and that number should have a unit");

        let number_values = numbers.into_iter().filter_map(|number| match number.value {
            Number::Scalar(value) => Some(value),
            Number::Interval(interval) => {
                if interval.is_empty() {
                    None
                } else {
                    Some(interval.max())
                }
            }
        });

        let max = number_values.reduce(f64::max);

        max.map_or_else(
            || Err(vec![EvalError::NoNonEmptyValue]),
            |max| {
                Ok(Value::Number(MeasuredNumber::new(
                    Number::Scalar(max),
                    unit,
                )))
            },
        )
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn sin(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn cos(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn tan(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn asin(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn acos(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn atan(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    pub fn sqrt(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        if args.len() != 1 {
            return Err(vec![EvalError::InvalidArgumentCount]);
        }

        let mut args = args.into_iter();

        let arg = args.next().expect("there should be one argument");

        arg.checked_pow(Value::from(0.5))
            .map_err(|error| vec![error])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn ln(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn log(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn log10(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn floor(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn ceiling(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn extent(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    pub fn range(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        match args.len() {
            1 => {
                let mut args = args.into_iter();

                let arg = args.next().expect("there should be one argument");

                let Value::Number(number) = arg else {
                    return Err(vec![EvalError::InvalidType]);
                };

                let Number::Interval(interval) = number.value else {
                    return Err(vec![EvalError::InvalidType]);
                };

                Ok(Value::Number(MeasuredNumber::new(
                    Number::Scalar(interval.max() - interval.min()),
                    number.unit,
                )))
            }
            2 => {
                let mut args = args.into_iter();

                let left = args.next().expect("there should be two arguments");
                let right = args.next().expect("there should be two arguments");

                left.checked_sub(right).map_err(|error| vec![error])
            }
            _ => Err(vec![EvalError::InvalidArgumentCount]),
        }
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn abs(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn sign(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    pub fn mid(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        match args.len() {
            1 => {
                let mut args = args.into_iter();

                let arg = args.next().expect("there should be one argument");

                let Value::Number(number) = arg else {
                    return Err(vec![EvalError::InvalidType]);
                };

                let Number::Interval(interval) = number.value else {
                    return Err(vec![EvalError::InvalidType]);
                };

                let mid = f64::midpoint(interval.min(), interval.max());

                Ok(Value::Number(MeasuredNumber::new(
                    Number::Scalar(mid),
                    number.unit,
                )))
            }
            2 => {
                let mut args = args.into_iter();

                let left = args.next().expect("there should be two arguments");
                let right = args.next().expect("there should be two arguments");

                left.checked_add(right)
                    .and_then(|value| value.checked_div(Value::from(2.0)))
                    .map_err(|error| vec![error])
            }
            _ => Err(vec![EvalError::InvalidArgumentCount]),
        }
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn strip(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn mnmx(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }
}
