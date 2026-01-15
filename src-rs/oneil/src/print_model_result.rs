use std::{fmt, str};

use anstream::{print, println};
use oneil_eval::{
    result,
    value::{self, Value},
};

use crate::stylesheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPrintConfig {
    pub print_mode: PrintMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintMode {
    All,
    Debug,
    Trace,
    Performance,
}

impl Default for PrintMode {
    fn default() -> Self {
        Self::Performance
    }
}

impl str::FromStr for PrintMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            "perf" | "performance" => Ok(Self::Performance),
            _ => Err(format!(
                "Invalid print mode: {s} (valid modes: all, debug, trace, perf, performance)"
            )),
        }
    }
}

impl fmt::Display for PrintMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
            Self::Performance => write!(f, "perf"),
        }
    }
}

pub fn print(model_result: &result::Model, print_debug: bool, model_config: &ModelPrintConfig) {
    if print_debug {
        println!("{model_result:?}");
        return;
    }

    print_model_header(model_result);

    let parameters_to_print = model_result
        .parameters
        .values()
        .filter(|parameter| parameter.is_performance)
        .collect::<Vec<_>>();

    if parameters_to_print.is_empty() {
        let message = stylesheet::NO_PARAMETERS_MESSAGE.style("(No performance parameters found)");
        println!("{message}");
        return;
    }

    for parameter in parameters_to_print {
        print_parameter(parameter);
    }
}

fn print_model_header(model_result: &result::Model) {
    let break_line = "─".repeat(80);
    let model_label = stylesheet::MODEL_LABEL.style("Model");
    let tests_label = stylesheet::TESTS_LABEL.style("Tests");

    let test_count = model_result.tests.len();
    let passed_count = model_result.tests.iter().filter(|test| test.passed).count();

    let test_result_string = if passed_count == test_count {
        stylesheet::TESTS_PASS_COLOR.style("PASS")
    } else {
        stylesheet::TESTS_FAIL_COLOR.style("FAIL")
    };

    println!("{break_line}");
    println!("{model_label}: {}", model_result.path.display());
    println!("{tests_label}: {passed_count}/{test_count} ({test_result_string})");
    println!("{break_line}");
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
