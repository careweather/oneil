use ::std::{collections::HashMap, rc::Rc};

use oneil_eval::{
    builtin::{BuiltinFunction, BuiltinMap},
    value::{MeasuredNumber, Number, SizedUnit, Unit, Value},
};
use oneil_ir as ir;
use oneil_model_resolver::BuiltinRef;

// TODO: later, this will hold the actual values/functions that are built into the language
//       right now, it just holds the names of the builtins
pub struct Builtins<F: BuiltinFunction> {
    values: HashMap<&'static str, f64>,
    functions: HashMap<&'static str, F>,
    // The units are stored as Rc<SizedUnit> so that multiple names
    // can point to the same unit (eg. "in", "inch", "inches")
    units: HashMap<&'static str, Rc<SizedUnit>>,
    prefixes: HashMap<&'static str, f64>,
}

impl<F: BuiltinFunction> Builtins<F> {
    pub fn new(
        values: impl IntoIterator<Item = (&'static str, f64)>,
        functions: impl IntoIterator<Item = (&'static str, F)>,
        units: impl IntoIterator<Item = (&'static str, Rc<SizedUnit>)>,
        prefixes: impl IntoIterator<Item = (&'static str, f64)>,
    ) -> Self {
        Self {
            values: values.into_iter().collect(),
            functions: functions.into_iter().collect(),
            units: units.into_iter().collect(),
            prefixes: prefixes.into_iter().collect(),
        }
    }
}

impl<F: BuiltinFunction> BuiltinRef for Builtins<F> {
    fn has_builtin_value(&self, identifier: &ir::Identifier) -> bool {
        self.values.contains_key(identifier.as_str())
    }

    fn has_builtin_function(&self, identifier: &ir::Identifier) -> bool {
        self.functions.contains_key(identifier.as_str())
        // matches!(
        //     identifier.as_str(),
        // )
    }
}

impl<F: BuiltinFunction + Clone> BuiltinMap<F> for Builtins<F> {
    fn builtin_values(&self) -> HashMap<String, Value> {
        self.values
            .iter()
            .map(|(name, value)| {
                (
                    (*name).to_string(),
                    Value::Number(MeasuredNumber::new(
                        Number::Scalar(*value),
                        Unit::unitless(),
                    )),
                )
            })
            .collect()
    }

    fn builtin_functions(&self) -> HashMap<String, F> {
        self.functions
            .iter()
            .map(|(name, function)| ((*name).to_string(), function.clone()))
            .collect()
    }

    fn builtin_units(&self) -> HashMap<String, SizedUnit> {
        self.units
            .iter()
            .map(|(name, unit)| ((*name).to_string(), unit.as_ref().clone()))
            .collect()
    }

    fn builtin_prefixes(&self) -> HashMap<String, f64> {
        self.prefixes
            .iter()
            .map(|(name, magnitude)| ((*name).to_string(), *magnitude))
            .collect()
    }
}

pub mod std {
    use std::{collections::HashMap, rc::Rc};

    use oneil_eval::{
        EvalError,
        value::{Dimension, MeasuredNumber, Number, SizedUnit, Unit, Value},
    };

    pub const BUILTIN_VALUES: [(&str, f64); 2] =
        [("pi", std::f64::consts::PI), ("e", std::f64::consts::E)];

    #[expect(clippy::too_many_lines, reason = "this is a list of builtin units")]
    #[expect(clippy::unreadable_literal, reason = "this is a list of builtin units")]
    pub fn builtin_units() -> HashMap<&'static str, Rc<SizedUnit>> {
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
                ["mil.", "millennium", "millennia"].as_ref(),
                SizedUnit {
                    magnitude: 3.1556952e10,
                    unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
                },
            ),
            (
                ["cen.", "century", "centuries"].as_ref(),
                SizedUnit {
                    magnitude: 3.1556952e9,
                    unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
                },
            ),
            (
                ["dec.", "decade", "decades"].as_ref(),
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
                let unit = Rc::new(unit);
                names.iter().map(move |name| (*name, Rc::clone(&unit)))
            })
            .collect()
    }

    pub const BUILTIN_PREFIXES: [(&str, f64); 20] = [
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
    ];

    type BuiltinFunction = fn(Vec<Value>) -> Result<Value, Vec<EvalError>>;
    pub const BUILTIN_FUNCTIONS: [(&str, BuiltinFunction); 21] = [
        ("min", min),
        ("max", max),
        ("sin", sin),
        ("cos", cos),
        ("tan", tan),
        ("asin", asin),
        ("acos", acos),
        ("atan", atan),
        ("sqrt", sqrt),
        ("ln", ln),
        ("log", log),
        ("log10", log10),
        ("floor", floor),
        ("ceiling", ceiling),
        ("extent", extent),
        ("range", range),
        ("abs", abs),
        ("sign", sign),
        ("mid", mid),
        ("strip", strip),
        ("mnmx", mnmx),
    ];

    fn min(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
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

    fn max(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
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
    fn sin(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn cos(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn tan(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn asin(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn acos(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn atan(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn sqrt(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn ln(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn log(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn log10(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn floor(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn ceiling(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn extent(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn range(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn abs(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn sign(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn mid(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn strip(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }

    #[expect(unused_variables, reason = "not implemented")]
    #[expect(clippy::needless_pass_by_value, reason = "not implemented")]
    fn mnmx(args: Vec<Value>) -> Result<Value, Vec<EvalError>> {
        Err(vec![EvalError::Unsupported])
    }
}
