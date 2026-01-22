//! The standard builtin values, functions, units, and prefixes
//! that come with Oneil.

use indexmap::IndexMap;

use oneil_shared::span::Span;

use crate::{
    EvalError,
    value::{Dimension, DimensionMap, DisplayUnit, Number, Unit, Value},
};

struct StdBuiltinValue {
    name: &'static str,
    value: Value,
    description: &'static str,
}
fn builtin_values_complete() -> impl Iterator<Item = StdBuiltinValue> {
    [
        StdBuiltinValue {
            name: "pi",
            value: Value::Number(Number::Scalar(std::f64::consts::PI)),
            description: "The mathematical constant π",
        },
        StdBuiltinValue {
            name: "e",
            value: Value::Number(Number::Scalar(std::f64::consts::E)),
            description: "The mathematical constant e",
        },
    ]
    .into_iter()
}

/// The builtin values that come with Oneil:
#[must_use]
pub fn builtin_values() -> IndexMap<String, Value> {
    builtin_values_complete()
        .map(|value| (value.name.to_string(), value.value))
        .collect()
}

/// The documentation for the builtin values that come with Oneil.
#[must_use]
pub fn builtin_values_docs() -> IndexMap<String, (String, Value)> {
    builtin_values_complete()
        .map(|value| {
            (
                value.name.to_string(),
                (value.description.to_string(), value.value),
            )
        })
        .collect()
}

struct StdBuiltinPrefix {
    name: &'static str,
    value: f64,
    description: &'static str,
}

#[expect(clippy::too_many_lines, reason = "this is a list of builtin prefixes")]
fn builtin_prefixes_complete() -> impl Iterator<Item = StdBuiltinPrefix> {
    [
        StdBuiltinPrefix {
            name: "q",
            value: 1e-30,
            description: "quecto",
        },
        StdBuiltinPrefix {
            name: "r",
            value: 1e-27,
            description: "ronto",
        },
        StdBuiltinPrefix {
            name: "y",
            value: 1e-24,
            description: "yocto",
        },
        StdBuiltinPrefix {
            name: "z",
            value: 1e-21,
            description: "zepto",
        },
        StdBuiltinPrefix {
            name: "a",
            value: 1e-18,
            description: "atto",
        },
        StdBuiltinPrefix {
            name: "f",
            value: 1e-15,
            description: "femto",
        },
        StdBuiltinPrefix {
            name: "p",
            value: 1e-12,
            description: "pico",
        },
        StdBuiltinPrefix {
            name: "n",
            value: 1e-9,
            description: "nano",
        },
        StdBuiltinPrefix {
            name: "u",
            value: 1e-6,
            description: "micro",
        },
        StdBuiltinPrefix {
            name: "m",
            value: 1e-3,
            description: "milli",
        },
        StdBuiltinPrefix {
            name: "k",
            value: 1e3,
            description: "kilo",
        },
        StdBuiltinPrefix {
            name: "M",
            value: 1e6,
            description: "mega",
        },
        StdBuiltinPrefix {
            name: "G",
            value: 1e9,
            description: "giga",
        },
        StdBuiltinPrefix {
            name: "T",
            value: 1e12,
            description: "tera",
        },
        StdBuiltinPrefix {
            name: "P",
            value: 1e15,
            description: "peta",
        },
        StdBuiltinPrefix {
            name: "E",
            value: 1e18,
            description: "exa",
        },
        StdBuiltinPrefix {
            name: "Z",
            value: 1e21,
            description: "zetta",
        },
        StdBuiltinPrefix {
            name: "Y",
            value: 1e24,
            description: "yotta",
        },
        StdBuiltinPrefix {
            name: "R",
            value: 1e27,
            description: "ronna",
        },
        StdBuiltinPrefix {
            name: "Q",
            value: 1e30,
            description: "quetta",
        },
    ]
    .into_iter()
}

/// The builtin unit prefixes that come with Oneil.
#[must_use]
pub fn builtin_prefixes() -> IndexMap<String, f64> {
    builtin_prefixes_complete()
        .map(|prefix| (prefix.name.to_string(), prefix.value))
        .collect()
}

/// The documentation for the builtin prefixes that come with Oneil.
#[must_use]
pub fn builtin_prefixes_docs() -> IndexMap<String, (String, f64)> {
    builtin_prefixes_complete()
        .map(|prefix| {
            (
                prefix.name.to_string(),
                (prefix.description.to_string(), prefix.value),
            )
        })
        .collect()
}

struct StdBuiltinUnit {
    name: &'static str,
    aliases: IndexMap<&'static str, Unit>,
}

/// The builtin units that come with Oneil.
#[expect(clippy::too_many_lines, reason = "this is a list of builtin units")]
#[expect(clippy::unreadable_literal, reason = "this is a list of builtin units")]
fn builtin_units_complete() -> impl Iterator<Item = StdBuiltinUnit> {
    /// Information about a builtin unit.
    ///
    /// This is only used in this function to avoid code duplication.
    struct UnitInfo {
        name: &'static str,
        aliases: &'static [&'static str],
        magnitude: f64,
        dimensions: DimensionMap,
        is_db: bool,
    }

    let units = [
        // === BASE UNITS ===
        UnitInfo {
            // the kilogram is the base unit of mass, so the gram is 1e-3 of a kilogram
            name: "gram",
            aliases: ["g", "gram", "grams"].as_ref(),
            magnitude: 1e-3,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Mass, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "meter",
            aliases: ["m", "meter", "meters", "metre", "metres"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "second",
            aliases: ["s", "second", "seconds", "sec", "secs"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "Kelvin",
            aliases: ["K", "Kelvin"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Temperature, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "Ampere",
            aliases: ["A", "Ampere", "Amp"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Current, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "bit",
            aliases: ["b", "bit", "bits"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Information, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "dollar",
            aliases: ["$", "dollar", "dollars"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "mole",
            aliases: ["mol", "mole", "moles"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Substance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "candela",
            aliases: ["cd", "candela"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::LuminousIntensity, 1.0)])),
            is_db: false,
        },
        // === DERIVED UNITS ===
        UnitInfo {
            name: "Volt",
            aliases: ["V", "Volt", "Volts"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
                (Dimension::Current, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Watt",
            aliases: ["W", "Watt", "Watts"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Hertz",
            aliases: ["Hz", "Hertz"].as_ref(),
            magnitude: 2.0 * std::f64::consts::PI,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, -1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "Joule",
            aliases: ["J", "Joule", "Joules"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Watt-hour",
            aliases: ["Wh", "Watt-hour", "Watt-hours"].as_ref(),
            magnitude: 3600.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Amp-hour",
            aliases: ["Ah", "Amp-hour", "Amp-hours"].as_ref(),
            magnitude: 3600.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Current, 1.0),
                (Dimension::Time, 1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Tesla",
            aliases: ["T", "Tesla", "Teslas"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Time, -2.0),
                (Dimension::Current, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Ohm",
            aliases: ["Ohm", "Ohms"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
                (Dimension::Current, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Newton",
            aliases: ["N", "Newton", "Newtons"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Gauss",
            aliases: ["Gs", "Gauss"].as_ref(),
            magnitude: 0.0001,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Time, -2.0),
                (Dimension::Current, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "Lumen",
            aliases: ["lm", "Lumen", "Lumens"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::LuminousIntensity, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "Lux",
            aliases: ["lx", "Lux", "Luxes"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::LuminousIntensity, 1.0),
                (Dimension::Distance, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "bits per second",
            aliases: ["bps"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Information, 1.0),
                (Dimension::Time, -1.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "byte",
            aliases: ["B", "byte", "bytes"].as_ref(),
            magnitude: 8.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Information, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "Pascal",
            aliases: ["Pa", "Pascal", "Pascals"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        // === LEGACY UNITS ===
        UnitInfo {
            name: "millennium",
            aliases: ["mil", "millennium", "millennia"].as_ref(),
            magnitude: 3.1556952e10,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "century",
            aliases: ["cen", "century", "centuries"].as_ref(),
            magnitude: 3.1556952e9,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "decade",
            aliases: ["dec", "decade", "decades"].as_ref(),
            magnitude: 3.1556952e8,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "year",
            aliases: ["yr", "year", "years"].as_ref(),
            magnitude: 3.1556952e7,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "month",
            aliases: ["mon", "month", "months"].as_ref(),
            magnitude: 2.629746e6,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "week",
            aliases: ["week", "weeks"].as_ref(),
            magnitude: 6.048e5,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "day",
            aliases: ["day", "days"].as_ref(),
            magnitude: 8.64e4,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "hour",
            aliases: ["hr", "hour", "hours"].as_ref(),
            magnitude: 3600.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "minute",
            aliases: ["min", "minute", "minutes"].as_ref(),
            magnitude: 60.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "revolutions per minute",
            aliases: ["rpm"].as_ref(),
            magnitude: 0.10471975511965977,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, -1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "thousand dollars",
            aliases: ["k$"].as_ref(),
            magnitude: 1000.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "million dollars",
            aliases: ["M$"].as_ref(),
            magnitude: 1e6,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "billion dollars",
            aliases: ["B$"].as_ref(),
            magnitude: 1e9,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "trillion dollars",
            aliases: ["T$"].as_ref(),
            magnitude: 1e12,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "Earth gravity",
            aliases: ["g_E"].as_ref(),
            magnitude: 9.81,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "centimeter",
            aliases: [
                "cm",
                "centimeter",
                "centimeters",
                "centimetre",
                "centimetres",
            ]
            .as_ref(),
            magnitude: 0.01,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "pounds per square inch",
            aliases: ["psi"].as_ref(),
            magnitude: 6894.757293168361,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "atmosphere",
            aliases: ["atm", "atmosphere", "atmospheres"].as_ref(),
            magnitude: 101325.0,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "bar",
            aliases: ["bar", "bars"].as_ref(),
            magnitude: 1e5,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "barye",
            aliases: ["Ba", "barye", "baryes"].as_ref(),
            magnitude: 0.1,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "dyne",
            aliases: ["dyne", "dynes"].as_ref(),
            magnitude: 1e-5,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "millimeter of mercury",
            aliases: ["mmHg"].as_ref(),
            magnitude: 133.322387415,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "torr",
            aliases: ["torr", "torrs"].as_ref(),
            magnitude: 133.3224,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, -1.0),
                (Dimension::Time, -2.0),
            ])),
            is_db: false,
        },
        UnitInfo {
            name: "inch",
            aliases: ["in", "inch", "inches"].as_ref(),
            magnitude: 0.0254,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "foot",
            aliases: ["ft", "foot", "feet"].as_ref(),
            magnitude: 0.3048,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "yard",
            aliases: ["yd", "yard", "yards"].as_ref(),
            magnitude: 0.9144,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "mile",
            aliases: ["mi", "mile", "miles"].as_ref(),
            magnitude: 1609.344,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "nautical mile",
            aliases: ["nmi"].as_ref(),
            magnitude: 1852.0,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "pound",
            aliases: ["lb", "lbs", "pound", "pounds"].as_ref(),
            magnitude: 0.45359237,
            dimensions: DimensionMap::new(IndexMap::from([(Dimension::Mass, 1.0)])),
            is_db: false,
        },
        UnitInfo {
            name: "mile per hour",
            aliases: ["mph"].as_ref(),
            magnitude: 0.44704,
            dimensions: DimensionMap::new(IndexMap::from([
                (Dimension::Distance, 1.0),
                (Dimension::Time, -1.0),
            ])),
            is_db: false,
        },
        // === DIMENSIONLESS UNITS ===
        UnitInfo {
            name: "revolution",
            aliases: ["rev", "revolution", "revolutions", "rotation", "rotations"].as_ref(),
            magnitude: 2.0 * std::f64::consts::PI,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "cycle",
            aliases: ["cyc", "cycle", "cycles"].as_ref(),
            magnitude: 2.0 * std::f64::consts::PI,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "radian",
            aliases: ["rad", "radian", "radians"].as_ref(),
            magnitude: 1.0,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "degree",
            aliases: ["deg", "degree", "degrees"].as_ref(),
            magnitude: 0.017453292519943295,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "percent",
            aliases: ["%", "percent"].as_ref(),
            magnitude: 0.01,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "part per million",
            aliases: ["ppm"].as_ref(),
            magnitude: 1e-6,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "part per billion",
            aliases: ["ppb"].as_ref(),
            magnitude: 1e-9,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "arcminute",
            aliases: ["arcmin", "arcminute", "arcminutes"].as_ref(),
            magnitude: 0.0002908882086657216,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
        UnitInfo {
            name: "arcsecond",
            aliases: ["arcsec", "arcsecond", "arcseconds"].as_ref(),
            magnitude: 4.84813681109536e-06,
            dimensions: DimensionMap::new(IndexMap::from([])),
            is_db: false,
        },
    ];

    units.into_iter().map(
        |UnitInfo {
             name,
             aliases,
             magnitude,
             dimensions,
             is_db,
         }| {
            let aliases = aliases
                .iter()
                .map(move |alias| {
                    let unit = Unit {
                        dimension_map: dimensions.clone(),
                        magnitude,
                        is_db,
                        display_unit: DisplayUnit::Unit {
                            name: (*alias).to_string(),
                            exponent: 1.0,
                        },
                    };
                    (*alias, unit)
                })
                .collect();

            StdBuiltinUnit { name, aliases }
        },
    )
}

/// The builtin units that come with Oneil.
#[must_use]
pub fn builtin_units() -> IndexMap<String, Unit> {
    builtin_units_complete()
        .flat_map(|unit| {
            unit.aliases
                .into_iter()
                .map(|(alias, unit)| (alias.to_string(), unit))
        })
        .collect()
}

/// The documentation for the builtin units that come with Oneil.
#[must_use]
pub fn builtin_units_docs() -> IndexMap<&'static str, Vec<&'static str>> {
    builtin_units_complete()
        .map(|unit| {
            let aliases: Vec<&str> = unit.aliases.keys().copied().collect();

            (unit.name, aliases)
        })
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
pub fn builtin_functions() -> IndexMap<String, StdBuiltinFunction> {
    IndexMap::from(
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

        arg.checked_pow(Value::from(0.5))
            .map_err(|error| vec![error.expect_only_lhs_error(arg_span)])
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

                left.checked_sub(right)
                    .map_err(|error| vec![error.into_eval_error(left_span, right_span)])
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
                    .map_err(|error| vec![error.into_eval_error(left_span, right_span)])
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
