//! Standard builtin units (SI, derived, legacy, dimensionless).

use indexmap::IndexMap;

use oneil_output::{Dimension, DimensionMap, DisplayUnit, Unit};

#[derive(Debug, Clone)]
pub struct BuiltinUnit {
    pub alias: &'static str,
    pub unit: Unit,
    pub readable_name: &'static str,
}

/// The builtin units that come with Oneil.
#[expect(clippy::too_many_lines, reason = "this is a list of builtin units")]
#[expect(clippy::unreadable_literal, reason = "this is a list of builtin units")]
/// Returns an iterator over all standard builtin units (by alias).
pub fn builtin_units_complete() -> impl Iterator<Item = (&'static str, BuiltinUnit)> {
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
