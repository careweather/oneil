//! Test support for evaluation tests.
//!
//! Provides [`TestExternalContext`] that implements [`ExternalResolutionContext`]
//! with standard builtins included implicitly. In tests, construct an external
//! context with [`TestExternalContext::new`], then pass a mutable reference
//! to it when creating an [`EvalContext`].

use std::path::Path;

use indexmap::IndexMap;

use oneil_ir as ir;
use oneil_output::{Unit, Value};
use oneil_shared::{load_result::LoadResult, span::Span};

use crate::{
    context::{ExternalEvaluationContext, IrLoadError},
    error::EvalError,
};

/// Test double for [`ExternalResolutionContext`] with standard builtins included.
///
/// [`TestExternalContext::new`] creates a context that already has the standard
/// builtin values, functions, units, and prefixes from the [`std`] module.
#[expect(
    clippy::struct_field_names,
    reason = "other non-builtin fields may be added in the future"
)]
#[derive(Debug)]
pub struct TestExternalContext {
    builtin_values: IndexMap<String, Value>,
    builtin_functions: IndexMap<String, std_builtins::StdBuiltinFunction>,
    builtin_units: IndexMap<String, Unit>,
    builtin_prefixes: IndexMap<String, f64>,
}

impl TestExternalContext {
    /// Creates a new test external context with standard builtins.
    #[must_use]
    pub fn new() -> Self {
        Self {
            builtin_values: std_builtins::builtin_values(),
            builtin_functions: std_builtins::builtin_functions(),
            builtin_units: std_builtins::builtin_units(),
            builtin_prefixes: std_builtins::builtin_prefixes(),
        }
    }
}

impl Default for TestExternalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalEvaluationContext for TestExternalContext {
    fn lookup_ir(&self, _path: impl AsRef<Path>) -> Option<LoadResult<&ir::Model, IrLoadError>> {
        panic!("no tests currently use this method")
    }

    fn lookup_builtin_variable(&self, identifier: &ir::Identifier) -> Option<&Value> {
        self.builtin_values.get(identifier.as_str())
    }

    fn evaluate_builtin_function(
        &self,
        identifier: &ir::Identifier,
        identifier_span: Span,
        args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Vec<EvalError>>> {
        self.builtin_functions
            .get(identifier.as_str())
            .map(|f| f(identifier_span, args))
    }

    fn evaluate_imported_function(
        &self,
        _python_path: &ir::PythonPath,
        _identifier: &ir::Identifier,
        _function_call_span: Span,
        _args: Vec<(Value, Span)>,
    ) -> Option<Result<Value, Box<EvalError>>> {
        // For now, we don't support imported functions in tests
        None
    }

    fn lookup_unit(&self, name: &str) -> Option<&Unit> {
        self.builtin_units.get(name)
    }

    fn lookup_prefix(&self, name: &str) -> Option<f64> {
        self.builtin_prefixes.get(name).copied()
    }
}

/// The standard builtin values, functions, units, and prefixes
/// that come with Oneil.
///
/// This duplicates the code in `oneil_runtime::std_builtin`, but
/// I think it is needed for the tests. Maybe there's a better way
/// to do those tests, though. Maybe we define "fake units", or even
/// just a couple of units that are used in the tests.
mod std_builtins {

    use indexmap::IndexMap;

    use oneil_output::{Dimension, DimensionMap, DisplayUnit, Number, Unit, Value};
    use oneil_shared::span::Span;

    use crate::EvalError;

    struct StdBuiltinValue {
        name: &'static str,
        value: Value,
    }

    /// The builtin values that come with Oneil:
    #[must_use]
    pub fn builtin_values() -> IndexMap<String, Value> {
        [
            StdBuiltinValue {
                name: "pi",
                value: Value::Number(Number::Scalar(std::f64::consts::PI)),
            },
            StdBuiltinValue {
                name: "e",
                value: Value::Number(Number::Scalar(std::f64::consts::E)),
            },
        ]
        .into_iter()
        .map(|value| (value.name.to_string(), value.value))
        .collect()
    }

    struct StdBuiltinPrefix {
        name: &'static str,
        value: f64,
    }

    /// The builtin unit prefixes that come with Oneil.
    #[must_use]
    pub fn builtin_prefixes() -> IndexMap<String, f64> {
        [
            StdBuiltinPrefix {
                name: "q",
                value: 1e-30,
            },
            StdBuiltinPrefix {
                name: "r",
                value: 1e-27,
            },
            StdBuiltinPrefix {
                name: "y",
                value: 1e-24,
            },
            StdBuiltinPrefix {
                name: "z",
                value: 1e-21,
            },
            StdBuiltinPrefix {
                name: "a",
                value: 1e-18,
            },
            StdBuiltinPrefix {
                name: "f",
                value: 1e-15,
            },
            StdBuiltinPrefix {
                name: "p",
                value: 1e-12,
            },
            StdBuiltinPrefix {
                name: "n",
                value: 1e-9,
            },
            StdBuiltinPrefix {
                name: "u",
                value: 1e-6,
            },
            StdBuiltinPrefix {
                name: "m",
                value: 1e-3,
            },
            StdBuiltinPrefix {
                name: "k",
                value: 1e3,
            },
            StdBuiltinPrefix {
                name: "M",
                value: 1e6,
            },
            StdBuiltinPrefix {
                name: "G",
                value: 1e9,
            },
            StdBuiltinPrefix {
                name: "T",
                value: 1e12,
            },
            StdBuiltinPrefix {
                name: "P",
                value: 1e15,
            },
            StdBuiltinPrefix {
                name: "E",
                value: 1e18,
            },
            StdBuiltinPrefix {
                name: "Z",
                value: 1e21,
            },
            StdBuiltinPrefix {
                name: "Y",
                value: 1e24,
            },
            StdBuiltinPrefix {
                name: "R",
                value: 1e27,
            },
            StdBuiltinPrefix {
                name: "Q",
                value: 1e30,
            },
        ]
        .into_iter()
        .map(|prefix| (prefix.name.to_string(), prefix.value))
        .collect()
    }

    struct StdBuiltinUnit {
        aliases: IndexMap<&'static str, Unit>,
    }

    /// The builtin units that come with Oneil.
    #[expect(clippy::too_many_lines, reason = "this is a list of builtin units")]
    #[expect(clippy::unreadable_literal, reason = "this is a list of builtin units")]
    #[must_use]
    pub fn builtin_units() -> IndexMap<String, Unit> {
        {
            /// Information about a builtin unit.
            ///
            /// This is only used in this function to avoid code duplication.
            struct UnitInfo {
                aliases: &'static [&'static str],
                magnitude: f64,
                dimensions: DimensionMap,
                is_db: bool,
            }

            let units = [
                // === BASE UNITS ===
                UnitInfo {
                    // the kilogram is the base unit of mass, so the gram is 1e-3 of a kilogram
                    aliases: ["g", "gram", "grams"].as_ref(),
                    magnitude: 1e-3,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Mass, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["m", "meter", "meters", "metre", "metres"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["s", "second", "seconds", "sec", "secs"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["K", "Kelvin"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Temperature, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["A", "Ampere", "Amp"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Current, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["b", "bit", "bits"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Information, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["$", "dollar", "dollars"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["mol", "mole", "moles"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Substance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["cd", "candela"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(
                        Dimension::LuminousIntensity,
                        1.0,
                    )])),
                    is_db: false,
                },
                // === DERIVED UNITS ===
                UnitInfo {
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
                    aliases: ["Hz", "Hertz"].as_ref(),
                    magnitude: 2.0 * std::f64::consts::PI,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, -1.0)])),
                    is_db: false,
                },
                UnitInfo {
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
                    aliases: ["Ah", "Amp-hour", "Amp-hours"].as_ref(),
                    magnitude: 3600.0,
                    dimensions: DimensionMap::new(IndexMap::from([
                        (Dimension::Current, 1.0),
                        (Dimension::Time, 1.0),
                    ])),
                    is_db: false,
                },
                UnitInfo {
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
                    aliases: ["lm", "Lumen", "Lumens"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([(
                        Dimension::LuminousIntensity,
                        1.0,
                    )])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["lx", "Lux", "Luxes"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([
                        (Dimension::LuminousIntensity, 1.0),
                        (Dimension::Distance, -2.0),
                    ])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["bps"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([
                        (Dimension::Information, 1.0),
                        (Dimension::Time, -1.0),
                    ])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["B", "byte", "bytes"].as_ref(),
                    magnitude: 8.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Information, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
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
                    aliases: ["mil", "millennium", "millennia"].as_ref(),
                    magnitude: 3.1556952e10,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["cen", "century", "centuries"].as_ref(),
                    magnitude: 3.1556952e9,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["dec", "decade", "decades"].as_ref(),
                    magnitude: 3.1556952e8,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["yr", "year", "years"].as_ref(),
                    magnitude: 3.1556952e7,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["mon", "month", "months"].as_ref(),
                    magnitude: 2.629746e6,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["week", "weeks"].as_ref(),
                    magnitude: 6.048e5,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["day", "days"].as_ref(),
                    magnitude: 8.64e4,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["hr", "hour", "hours"].as_ref(),
                    magnitude: 3600.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["min", "minute", "minutes"].as_ref(),
                    magnitude: 60.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["rpm"].as_ref(),
                    magnitude: 0.10471975511965977,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Time, -1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["k$"].as_ref(),
                    magnitude: 1000.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["M$"].as_ref(),
                    magnitude: 1e6,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["B$"].as_ref(),
                    magnitude: 1e9,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["T$"].as_ref(),
                    magnitude: 1e12,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Currency, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["g_E"].as_ref(),
                    magnitude: 9.81,
                    dimensions: DimensionMap::new(IndexMap::from([
                        (Dimension::Mass, 1.0),
                        (Dimension::Time, -2.0),
                    ])),
                    is_db: false,
                },
                UnitInfo {
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
                    aliases: ["in", "inch", "inches"].as_ref(),
                    magnitude: 0.0254,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["ft", "foot", "feet"].as_ref(),
                    magnitude: 0.3048,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["yd", "yard", "yards"].as_ref(),
                    magnitude: 0.9144,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["mi", "mile", "miles"].as_ref(),
                    magnitude: 1609.344,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["nmi"].as_ref(),
                    magnitude: 1852.0,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Distance, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["lb", "lbs", "pound", "pounds"].as_ref(),
                    magnitude: 0.45359237,
                    dimensions: DimensionMap::new(IndexMap::from([(Dimension::Mass, 1.0)])),
                    is_db: false,
                },
                UnitInfo {
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
                    aliases: ["rev", "revolution", "revolutions", "rotation", "rotations"].as_ref(),
                    magnitude: 2.0 * std::f64::consts::PI,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["cyc", "cycle", "cycles"].as_ref(),
                    magnitude: 2.0 * std::f64::consts::PI,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["rad", "radian", "radians"].as_ref(),
                    magnitude: 1.0,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["deg", "degree", "degrees"].as_ref(),
                    magnitude: 0.017453292519943295,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["%", "percent"].as_ref(),
                    magnitude: 0.01,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["ppm"].as_ref(),
                    magnitude: 1e-6,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["ppb"].as_ref(),
                    magnitude: 1e-9,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["arcmin", "arcminute", "arcminutes"].as_ref(),
                    magnitude: 0.0002908882086657216,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
                UnitInfo {
                    aliases: ["arcsec", "arcsecond", "arcseconds"].as_ref(),
                    magnitude: 4.84813681109536e-06,
                    dimensions: DimensionMap::new(IndexMap::from([])),
                    is_db: false,
                },
            ];

            units.into_iter().map(
                |UnitInfo {
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

                    StdBuiltinUnit { aliases }
                },
            )
        }
        .flat_map(|unit| {
            unit.aliases
                .into_iter()
                .map(|(alias, unit)| (alias.to_string(), unit))
        })
        .collect()
    }

    /// Information about a builtin function.
    pub struct StdBuiltinFunctionInfo {
        name: &'static str,
        function: StdBuiltinFunction,
    }

    /// Type alias for standard builtin function type
    pub type StdBuiltinFunction = fn(Span, Vec<(Value, Span)>) -> Result<Value, Vec<EvalError>>;

    /// The builtin functions that come with Oneil.
    ///
    /// Note that some of these functions are not yet implemented and
    /// will return an `EvalError::Unsupported` error when called. However,
    /// we plan to implement them in the future.
    #[must_use]
    pub fn builtin_functions() -> IndexMap<String, StdBuiltinFunction> {
        [
            StdBuiltinFunctionInfo {
                name: "min",
                function: fns::min as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "max",
                function: fns::max as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "sin",
                function: fns::sin as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "cos",
                function: fns::cos as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "tan",
                function: fns::tan as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "asin",
                function: fns::asin as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "acos",
                function: fns::acos as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "atan",
                function: fns::atan as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "sqrt",
                function: fns::sqrt as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "ln",
                function: fns::ln as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "log",
                function: fns::log as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "log10",
                function: fns::log10 as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "floor",
                function: fns::floor as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "ceiling",
                function: fns::ceiling as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "extent",
                function: fns::extent as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "range",
                function: fns::range as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "abs",
                function: fns::abs as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "sign",
                function: fns::sign as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "mid",
                function: fns::mid as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "strip",
                function: fns::strip as StdBuiltinFunction,
            },
            StdBuiltinFunctionInfo {
                name: "mnmx",
                function: fns::mnmx as StdBuiltinFunction,
            },
        ]
        .into_iter()
        .map(|info| (info.name.to_string(), info.function))
        .collect()
    }

    mod fns {
        use oneil_output::{DisplayUnit, MeasuredNumber, Number, NumberType, Unit, Value};
        use oneil_shared::span::Span;

        use crate::{
            EvalError,
            error::{
                ExpectedArgumentCount, ExpectedType,
                convert::{binary_eval_error_expect_only_lhs, binary_eval_error_to_eval_error},
            },
        };

        #[expect(
            clippy::needless_pass_by_value,
            reason = "matches the expected signature"
        )]
        pub fn min(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
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
        pub fn max(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
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
        pub fn sin(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("sin".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn cos(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("cos".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn tan(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("tan".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn asin(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("asin".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn acos(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("acos".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn atan(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("atan".to_string()),
                will_be_supported: true,
            }])
        }

        pub fn sqrt(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
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

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn ln(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("ln".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn log(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("log".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn log10(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("log10".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn floor(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
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

        pub fn range(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
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

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn abs(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("abs".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn sign(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("sign".to_string()),
                will_be_supported: true,
            }])
        }

        pub fn mid(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
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

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn strip(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("strip".to_string()),
                will_be_supported: true,
            }])
        }

        #[expect(unused_variables, reason = "not implemented")]
        #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
        pub fn mnmx(
            identifier_span: Span,
            args: Vec<(Value, Span)>,
        ) -> Result<Value, Vec<EvalError>> {
            Err(vec![EvalError::Unsupported {
                relevant_span: identifier_span,
                feature_name: Some("mnmx".to_string()),
                will_be_supported: true,
            }])
        }

        // Duplicate of homogeneous number list logic (also in oneil_runtime::std_builtin::fns).
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
                        expected_unit: DisplayUnit::One,
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
                        found_unit: DisplayUnit::One,
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
}
