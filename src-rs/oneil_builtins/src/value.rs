//! Standard builtin values (e.g. `pi`, `e`).

use oneil_output::{Number, Value};

#[derive(Debug, Clone)]
pub struct BuiltinValue {
    pub name: &'static str,
    pub value: Value,
    pub description: &'static str,
}

/// Returns an iterator over all standard builtin values.
pub fn builtin_values_complete() -> impl Iterator<Item = (&'static str, BuiltinValue)> {
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
