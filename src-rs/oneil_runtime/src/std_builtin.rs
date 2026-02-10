//! The standard builtin values, functions, units, and prefixes
//! that come with Oneil.

use indexmap::IndexMap;

use oneil_eval::EvalError;
use oneil_output::{Dimension, DimensionMap, DisplayUnit, Number, Unit, Value};
use oneil_shared::span::Span;

#[derive(Debug, Clone)]
pub struct StdBuiltins {
    values: IndexMap<&'static str, BuiltinValue>,
    functions: IndexMap<&'static str, BuiltinFunction>,
    units: IndexMap<&'static str, BuiltinUnit>,
    prefixes: IndexMap<&'static str, BuiltinPrefix>,
}

impl StdBuiltins {
    pub fn new() -> Self {
        Self {
            values: builtin_values_complete().collect(),
            functions: builtin_functions_complete().collect(),
            units: builtin_units_complete().collect(),
            prefixes: builtin_prefixes_complete().collect(),
        }
    }

    pub fn has_builtin_value(&self, identifier: &str) -> bool {
        self.values.contains_key(identifier)
    }

    pub fn has_builtin_function(&self, identifier: &str) -> bool {
        self.functions.contains_key(identifier)
    }

    pub fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.values.get(identifier).map(|value| &value.value)
    }

    pub fn get_function(&self, identifier: &str) -> Option<BuiltinFunctionFn> {
        self.functions
            .get(identifier)
            .map(|function| function.function)
    }

    pub fn get_unit(&self, name: &str) -> Option<&Unit> {
        self.units.get(name).map(|unit| &unit.unit)
    }

    pub fn builtin_prefixes(&self) -> impl Iterator<Item = (&str, f64)> {
        self.prefixes
            .iter()
            .map(|(name, prefix)| (*name, prefix.value))
    }

    /// Returns documentation for all builtin units.
    ///
    /// Each item is the canonical unit name and a list of all aliases
    /// (which may not include the canonical name).
    pub fn builtin_units_docs(&self) -> impl Iterator<Item = (&'static str, Vec<&'static str>)> {
        let mut by_name: IndexMap<&'static str, Vec<&'static str>> = IndexMap::new();
        for unit in self.units.values() {
            by_name
                .entry(unit.readable_name)
                .or_default()
                .push(unit.alias);
        }
        by_name.into_iter()
    }

    /// Returns documentation for all builtin functions.
    ///
    /// Each item is the function name, its argument names, and its description.
    pub fn builtin_functions_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static [&'static str], &'static str))> + '_ {
        self.functions
            .iter()
            .map(|(name, f)| (*name, (f.args, f.description)))
    }

    /// Returns documentation for all builtin values.
    ///
    /// Each item is the value name, its description, and the value itself.
    pub fn builtin_values_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, Value))> + '_ {
        self.values
            .iter()
            .map(|(name, v)| (*name, (v.description, v.value.clone())))
    }

    /// Returns documentation for all builtin prefixes.
    ///
    /// Each item is the prefix name, its description, and its numeric value.
    pub fn builtin_prefixes_docs(
        &self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, f64))> + '_ {
        self.prefixes
            .iter()
            .map(|(name, p)| (*name, (p.description, p.value)))
    }
}

#[derive(Debug, Clone)]
struct BuiltinValue {
    name: &'static str,
    value: Value,
    description: &'static str,
}

fn builtin_values_complete() -> impl Iterator<Item = (&'static str, BuiltinValue)> {
    [
        BuiltinValue {
            name: "pi",
            value: Value::Number(Number::Scalar(std::f64::consts::PI)),
            description: "The mathematical constant π",
        },
        BuiltinValue {
            name: "e",
            value: Value::Number(Number::Scalar(std::f64::consts::E)),
            description: "The mathematical constant e",
        },
    ]
    .into_iter()
    .map(|value| (value.name, value))
}

#[derive(Debug, Clone)]
pub struct BuiltinPrefix {
    prefix: &'static str,
    value: f64,
    description: &'static str,
}

#[expect(clippy::too_many_lines, reason = "this is a list of builtin prefixes")]
fn builtin_prefixes_complete() -> impl Iterator<Item = (&'static str, BuiltinPrefix)> {
    [
        BuiltinPrefix {
            prefix: "q",
            value: 1e-30,
            description: "quecto",
        },
        BuiltinPrefix {
            prefix: "r",
            value: 1e-27,
            description: "ronto",
        },
        BuiltinPrefix {
            prefix: "y",
            value: 1e-24,
            description: "yocto",
        },
        BuiltinPrefix {
            prefix: "z",
            value: 1e-21,
            description: "zepto",
        },
        BuiltinPrefix {
            prefix: "a",
            value: 1e-18,
            description: "atto",
        },
        BuiltinPrefix {
            prefix: "f",
            value: 1e-15,
            description: "femto",
        },
        BuiltinPrefix {
            prefix: "p",
            value: 1e-12,
            description: "pico",
        },
        BuiltinPrefix {
            prefix: "n",
            value: 1e-9,
            description: "nano",
        },
        BuiltinPrefix {
            prefix: "u",
            value: 1e-6,
            description: "micro",
        },
        BuiltinPrefix {
            prefix: "m",
            value: 1e-3,
            description: "milli",
        },
        BuiltinPrefix {
            prefix: "k",
            value: 1e3,
            description: "kilo",
        },
        BuiltinPrefix {
            prefix: "M",
            value: 1e6,
            description: "mega",
        },
        BuiltinPrefix {
            prefix: "G",
            value: 1e9,
            description: "giga",
        },
        BuiltinPrefix {
            prefix: "T",
            value: 1e12,
            description: "tera",
        },
        BuiltinPrefix {
            prefix: "P",
            value: 1e15,
            description: "peta",
        },
        BuiltinPrefix {
            prefix: "E",
            value: 1e18,
            description: "exa",
        },
        BuiltinPrefix {
            prefix: "Z",
            value: 1e21,
            description: "zetta",
        },
        BuiltinPrefix {
            prefix: "Y",
            value: 1e24,
            description: "yotta",
        },
        BuiltinPrefix {
            prefix: "R",
            value: 1e27,
            description: "ronna",
        },
        BuiltinPrefix {
            prefix: "Q",
            value: 1e30,
            description: "quetta",
        },
    ]
    .into_iter()
    .map(|prefix| (prefix.prefix, prefix))
}

#[derive(Debug, Clone)]
struct BuiltinUnit {
    alias: &'static str,
    unit: Unit,
    readable_name: &'static str,
}

/// The builtin units that come with Oneil.
#[expect(clippy::too_many_lines, reason = "this is a list of builtin units")]
#[expect(clippy::unreadable_literal, reason = "this is a list of builtin units")]
fn builtin_units_complete() -> impl Iterator<Item = (&'static str, BuiltinUnit)> {
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

    units.into_iter().flat_map(
        |UnitInfo {
             name,
             aliases,
             magnitude,
             dimensions,
             is_db,
         }| {
            aliases.iter().map(move |alias| {
                let unit = Unit {
                    dimension_map: dimensions.clone(),
                    magnitude,
                    is_db,
                    display_unit: DisplayUnit::Unit {
                        name: (*alias).to_string(),
                        exponent: 1.0,
                    },
                };
                (
                    *alias,
                    BuiltinUnit {
                        alias,
                        unit,
                        readable_name: name,
                    },
                )
            })
        },
    )
}

/// Information about a builtin function.
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    name: &'static str,
    args: &'static [&'static str],
    description: &'static str,
    function: BuiltinFunctionFn,
}

/// Type alias for standard builtin function type
pub type BuiltinFunctionFn = fn(Span, Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>>;

#[expect(clippy::too_many_lines, reason = "this is a list of builtin functions")]
fn builtin_functions_complete() -> impl Iterator<Item = (&'static str, BuiltinFunction)> {
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
