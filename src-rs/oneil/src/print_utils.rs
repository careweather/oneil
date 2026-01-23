//! Shared printing utilities for the Oneil CLI

use anstream::print;
use oneil_eval::value::{self, Value};

use crate::stylesheet;

/// Prints a value in a format suitable for display.
pub fn print_value(value: &Value) {
    match value {
        Value::String(string) => print!("'{string}'"),
        Value::Boolean(boolean) => print!("{boolean}"),
        Value::Number(number) => print_number_value(number),
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.clone().into_number_and_unit();
            print_number_value(&number);
            print_number_unit(&unit);
        }
    }
}

/// Prints a number value.
pub fn print_number_value(value: &value::Number) {
    match value {
        value::Number::Scalar(scalar) => print!("{scalar}"),
        value::Number::Interval(interval) => print!("{} | {}", interval.min(), interval.max()),
    }
}

/// Prints a number unit.
pub fn print_number_unit(unit: &value::Unit) {
    let styled_display_unit = stylesheet::PARAMETER_UNIT.style(unit.display_unit.to_string());
    print!(" :{styled_display_unit}");
}
