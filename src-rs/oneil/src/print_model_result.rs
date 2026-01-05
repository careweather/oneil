use anstream::{print, println};
use oneil_eval::{
    result,
    value::{self, SizedMeasuredNumber, Value},
};

use crate::stylesheet;

pub struct ModelPrintConfig {}

pub fn print(model_result: &result::Model, print_debug: bool, model_config: &ModelPrintConfig) {
    if print_debug {
        println!("{model_result:?}");
        return;
    }

    for parameter in model_result.parameters.values() {
        if parameter.is_performance {
        print_parameter(parameter);
    }
}
}

fn print_parameter(parameter: &result::Parameter) {
    let styled_ident = stylesheet::PARAMETER_IDENTIFIER.style(&parameter.ident);
    print!("{styled_ident} = ");

    print_value(&parameter.value, parameter.unit.as_ref());

    let styled_label = stylesheet::PARAMETER_LABEL.style(format!("# {}", parameter.label));
    println!("  {styled_label}");
}

fn print_value(value: &Value, sized_unit: Option<&value::SizedUnit>) {
    match value {
        Value::String(string) => print!("'{}'", string),
        Value::Boolean(boolean) => print!("{}", boolean),
        Value::Number(number) => {
            let sized_unit = sized_unit.expect("number value must have a sized unit");
            let number =
                SizedMeasuredNumber::from_measured_number(number.clone(), sized_unit.clone());

            print_number_value(&number.value);
            print_number_unit(sized_unit);
        }
    }
}

fn print_number_value(value: &value::Number) {
    match value {
        value::Number::Scalar(scalar) => print!("{}", scalar),
        value::Number::Interval(interval) => print!("{} | {}", interval.min(), interval.max()),
    }
}

fn print_number_unit(unit: &value::SizedUnit) {
    if let Some(display_unit) = &unit.display_unit {
        let styled_display_unit = stylesheet::PARAMETER_UNIT.style(display_unit.to_string());
        print!(" :{styled_display_unit}");
    } else {
        print!(""); // nothing for now
    }
}
