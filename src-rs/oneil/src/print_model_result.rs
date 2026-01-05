use anstream::{print, println};
use oneil_eval::{
    result,
    value::{self, Value},
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

    print_value(&parameter.value);

    let styled_label = stylesheet::PARAMETER_LABEL.style(format!("# {}", parameter.label));
    println!("  {styled_label}");
}

fn print_value(value: &Value) {
    match value {
        Value::String(string) => print!("'{}'", string),
        Value::Boolean(boolean) => print!("{}", boolean),
        Value::Number(number) => print_number_value(number),
        Value::MeasuredNumber(number) => {
            let (number, unit) = number.clone().into_number_and_unit();
            print_number_value(&number);
            print_number_unit(&unit);
        }
    }
}

fn print_number_value(value: &value::Number) {
    match value {
        value::Number::Scalar(scalar) => print!("{}", scalar),
        value::Number::Interval(interval) => print!("{} | {}", interval.min(), interval.max()),
    }
}

fn print_number_unit(unit: &value::Unit) {
    let styled_display_unit = stylesheet::PARAMETER_UNIT.style(unit.display_unit.to_string());
    print!(" :{styled_display_unit}");
}
