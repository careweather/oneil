//! Shared printing utilities for the Oneil CLI

use anstream::print;
use oneil_output::util::format_number_for_display;
use oneil_runtime::output::{Number, Unit, Value};

use crate::stylesheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrintUtilsConfig {
    pub sig_figs: usize,
}

/// Prints a value in a format suitable for display.
pub fn print_value(value: &Value, config: PrintUtilsConfig) {
    match value {
        Value::String(string) => print!("'{string}'"),
        Value::Boolean(boolean) => print!("{boolean}"),
        Value::Number(number) => print_number_value(number, config),
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.clone().into_number_and_unit();

            print_number_value(&number, config);

            if !unit.is_effectively_unitless() {
                print_number_unit(&unit);
            }
        }
    }
}

/// Prints a number value.
pub fn print_number_value(value: &Number, config: PrintUtilsConfig) {
    print!("{}", format_number_for_display(value, config.sig_figs));
}

/// Prints a number unit.
pub fn print_number_unit(unit: &Unit) {
    let styled_display_unit = stylesheet::PARAMETER_UNIT.style(unit.display_unit.to_string());
    print!(" :{styled_display_unit}");
}
