use ::std::collections::HashMap;

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
    units: HashMap<&'static str, SizedUnit>,
    prefixes: HashMap<&'static str, f64>,
}

impl<F: BuiltinFunction> Builtins<F> {
    pub fn new(
        values: impl IntoIterator<Item = (&'static str, f64)>,
        functions: impl IntoIterator<Item = (&'static str, F)>,
        units: impl IntoIterator<Item = (&'static str, SizedUnit)>,
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
            .map(|(name, unit)| ((*name).to_string(), unit.clone()))
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
    use std::collections::HashMap;

    use oneil_eval::{
        EvalError,
        value::{Dimension, MeasuredNumber, Number, SizedUnit, Unit, Value},
    };

    pub const BUILTIN_VALUES: [(&str, f64); 2] =
        [("pi", std::f64::consts::PI), ("e", std::f64::consts::E)];

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

    pub fn builtin_units() -> HashMap<&'static str, SizedUnit> {
        HashMap::from([
            // the kilogram is the base unit of mass, so the gram is 1e-3 of a kilogram
            (
                "g",
                SizedUnit {
                    magnitude: 1e-3,
                    unit: Unit::new(HashMap::from([(Dimension::Mass, 1.0)])),
                },
            ),
            (
                "m",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Distance, 1.0)])),
                },
            ),
            (
                "s",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Time, 1.0)])),
                },
            ),
            (
                "K",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Temperature, 1.0)])),
                },
            ),
            (
                "A",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Current, 1.0)])),
                },
            ),
            (
                "b",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Information, 1.0)])),
                },
            ),
            (
                "$",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Currency, 1.0)])),
                },
            ),
            (
                "mol",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::Substance, 1.0)])),
                },
            ),
            (
                "cd",
                SizedUnit {
                    magnitude: 1.0,
                    unit: Unit::new(HashMap::from([(Dimension::LuminousIntensity, 1.0)])),
                },
            ),
        ])
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
