//! The standard builtin values, functions, units, and prefixes
//! that come with Oneil.

use ::std::collections::HashMap;

use oneil_shared::span::Span;

use crate::{
    EvalError,
    value::{Dimension, DimensionMap, DisplayUnit, Number, Unit, Value},
};

/// The builtin values that come with Oneil:
/// - `pi` - the mathematical constant π
/// - `e` - the mathematical constant e
#[must_use]
pub fn builtin_values() -> HashMap<String, Value> {
    HashMap::from([
        (
            "pi".to_string(),
            Value::Number(Number::Scalar(std::f64::consts::PI)),
        ),
        (
            "e".to_string(),
            Value::Number(Number::Scalar(std::f64::consts::E)),
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
pub fn builtin_units() -> HashMap<String, Unit> {
    /// Information about a builtin unit.
    ///
    /// This is only used in this function to avoid code duplication.
    struct UnitInfo {
        names: &'static [&'static str],
        magnitude: f64,
        dimensions: DimensionMap,
        is_db: bool,
    }

    let units = vec![
        // === BASE UNITS ===
        UnitInfo {
            // the kilogram is the base unit of mass, so the gram is 1e-3 of a kilogram
            names: ["g", "gram", "grams"].as_ref(),
            magnitude: 1e-3,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Mass, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["m", "meter", "meters", "metre", "metres"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["s", "second", "seconds", "sec", "secs"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["K", "Kelvin"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Temperature, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["A", "Ampere", "Amp"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Current, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["b", "bit", "bits"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Information, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["$", "dollar", "dollars"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["mol", "mole", "moles"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Substance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["cd", "candela"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::LuminousIntensity, 1.0)])),
            is_db: false,
        },
        // === DERIVED UNITS ===
        UnitInfo {
            names: ["V", "Volt", "Volts"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
                (Dimension::Current, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["W", "Watt", "Watts"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["Hz", "Hertz"].as_ref(),
            magnitude: 2.0 * std::f64::consts::PI,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, -1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["J", "Joule", "Joules"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["Wh", "Watt-hour", "Watt-hours"].as_ref(),
            magnitude: 3600.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["Ah", "Amp-hour", "Amp-hours"].as_ref(),
            magnitude: 3600.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Current, 1.0),
                (Dimension::Time, 1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["T", "Tesla", "Teslas"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Time, -2.0),
                (Dimension::Current, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["Ohm", "Ohms"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
                (Dimension::Current, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["N", "Newton", "Newtons"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["Gs", "Gauss"].as_ref(),
            magnitude: 0.0001,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Time, -2.0),
                (Dimension::Current, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["lm", "Lumen", "Lumens"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::LuminousIntensity, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["lx", "Lux", "Luxes"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::LuminousIntensity, 1.0),
                (Dimension::Distance, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["bps" /* bits per second */].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Information, 1.0),
                (Dimension::Time, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["B", "byte", "bytes"].as_ref(),
            magnitude: 8.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Information, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["Pa", "Pascal", "Pascals"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        // === LEGACY UNITS ===
        UnitInfo {
            names: ["mil", "millennium", "millennia"].as_ref(),
            magnitude: 3.1556952e10,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["cen", "century", "centuries"].as_ref(),
            magnitude: 3.1556952e9,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["dec", "decade", "decades"].as_ref(),
            magnitude: 3.1556952e8,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["yr", "year", "years"].as_ref(),
            magnitude: 3.1556952e7,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["mon", "month", "months"].as_ref(),
            magnitude: 2.629746e6,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["week", "weeks"].as_ref(),
            magnitude: 6.048e5,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["day", "days"].as_ref(),
            magnitude: 8.64e4,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["hr", "hour", "hours"].as_ref(),
            magnitude: 3600.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["min", "minute", "minutes"].as_ref(),
            magnitude: 60.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["rpm" /* revolutions per minute */].as_ref(),
            magnitude: 0.10471975511965977,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Time, -1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["k$" /* thousand dollars */].as_ref(),
            magnitude: 1000.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["M$" /* million dollars */].as_ref(),
            magnitude: 1e6,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["B$" /* billion dollars */].as_ref(),
            magnitude: 1e9,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["T$" /* trillion dollars */].as_ref(),
            magnitude: 1e12,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["g_E" /* Earth gravity */].as_ref(),
            magnitude: 9.81,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: [
                "cm",
                "centimeter",
                "centimeters",
                "centimetre",
                "centimetres",
            ]
            .as_ref(),
            magnitude: 0.01,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["psi" /* pounds per square inch */].as_ref(),
            magnitude: 6894.757293168361,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["atm", "atmosphere", "atmospheres"].as_ref(),
            magnitude: 101325.0,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["bar", "bars"].as_ref(),
            magnitude: 1e5,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["Ba", "barye", "baryes"].as_ref(),
            magnitude: 0.1,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["dyne", "dynes"].as_ref(),
            magnitude: 1e-5,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["mmHg" /* millimeter of mercury */].as_ref(),
            magnitude: 133.322387415,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["torr", "torrs"].as_ref(),
            magnitude: 133.3224,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            names: ["in", "inch", "inches"].as_ref(),
            magnitude: 0.0254,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["ft", "foot", "feet"].as_ref(),
            magnitude: 0.3048,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["yd", "yard", "yards"].as_ref(),
            magnitude: 0.9144,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["mi", "mile", "miles"].as_ref(),
            magnitude: 1609.344,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["nmi" /* nautical mile */].as_ref(),
            magnitude: 1852.0,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["lb", "lbs", "pound", "pounds"].as_ref(),
            magnitude: 0.45359237,
            dimensions: DimensionMap::new(HashMap::from([(Dimension::Mass, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            names: ["mph" /* mile per hour */].as_ref(),
            magnitude: 0.44704,
            dimensions: DimensionMap::new(HashMap::from([
                (Dimension::Distance, 1.0),
                (Dimension::Time, -1.0),
            ])),
            is_db: false,
        },
        // === DIMENSIONLESS UNITS ===
        UnitInfo {
            names: ["rev", "revolution", "revolutions", "rotation", "rotations"].as_ref(),
            magnitude: 2.0 * std::f64::consts::PI,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["cyc", "cycle", "cycles"].as_ref(),
            magnitude: 2.0 * std::f64::consts::PI,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["rad", "radian", "radians"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["deg", "degree", "degrees"].as_ref(),
            magnitude: 0.017453292519943295,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["%", "percent"].as_ref(),
            magnitude: 0.01,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["ppm" /* part per million */].as_ref(),
            magnitude: 1e-6,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["ppb" /* part per billion */].as_ref(),
            magnitude: 1e-9,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["arcmin", "arcminute", "arcminutes"].as_ref(),
            magnitude: 0.0002908882086657216,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
        UnitInfo {
            names: ["arcsec", "arcsecond", "arcseconds"].as_ref(),
            magnitude: 4.84813681109536e-06,
            dimensions: DimensionMap::new(HashMap::from([])),
            is_db: false,
        },
    ];

    units
        .into_iter()
        .flat_map(
            |UnitInfo {
                 names,
                 magnitude,
                 dimensions,
                 is_db,
             }| {
                names.iter().map(move |name| {
                    let unit = Unit {
                        dimension_map: dimensions.clone(),
                        magnitude,
                        is_db,
                        display_unit: DisplayUnit::Unit {
                            name: (*name).to_string(),
                            exponent: 1.0,
                        },
                    };
                    ((*name).to_string(), unit)
                })
            },
        )
        .collect()
}

/// Type alias for standard builtin function type
pub type StdBuiltinFunction = fn(Span, Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>>;

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
    use oneil_shared::span::Span;

    use crate::{
        EvalError,
        error::{ExpectedArgumentCount, ExpectedType},
        value::{
            MeasuredNumber, Number, NumberType, Value,
            util::{HomogeneousNumberList, extract_homogeneous_numbers_list},
        },
    };

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

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn sin(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("sin".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn cos(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("cos".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn tan(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("tan".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn asin(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("asin".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn acos(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("acos".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn atan(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("atan".to_string()),
            will_be_supported: true,
        }])
    }

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

        arg.checked_pow(Value::from(0.5)).map_err(|error| {
            vec![EvalError::BinaryEvalError {
                lhs_span: arg_span,
                // This isn't relevant, since the only possible error is that
                // the argument is not a number, so we just use the identifier span
                // TODO: figure out a better way to handle this
                rhs_span: identifier_span,
                error,
            }]
        })
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn ln(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("ln".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn log(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("log".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn log10(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("log10".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn floor(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("floor".to_string()),
            will_be_supported: true,
        }])
    }

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
                    vec![EvalError::BinaryEvalError {
                        lhs_span: left_span,
                        rhs_span: right_span,
                        error,
                    }]
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

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn abs(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("abs".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn sign(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("sign".to_string()),
            will_be_supported: true,
        }])
    }

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
                        vec![EvalError::BinaryEvalError {
                            lhs_span: left_span,
                            rhs_span: right_span,
                            error,
                        }]
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

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn strip(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("strip".to_string()),
            will_be_supported: true,
        }])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    pub fn mnmx(identifier_span: Span, args: Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported {
            relevant_span: identifier_span,
            feature_name: Some("mnmx".to_string()),
            will_be_supported: true,
        }])
    }
}
