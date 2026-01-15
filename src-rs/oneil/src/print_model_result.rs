use std::{collections::HashMap, fmt, path::Path, str};

use anstream::{print, println};
use oneil_eval::{
    result,
    value::{self, Value},
};

use crate::stylesheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPrintConfig {
    pub print_level: PrintLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintLevel {
    All,
    Debug,
    Trace,
    Performance,
}

impl Default for PrintLevel {
    fn default() -> Self {
        Self::Performance
    }
}

impl str::FromStr for PrintLevel {
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

impl fmt::Display for PrintLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
            Self::Performance => write!(f, "perf"),
        }
    }
}

pub fn divider_line() -> String {
    "─".repeat(80)
}

pub fn print(model_result: &result::Model, print_debug: bool, model_config: ModelPrintConfig) {
    if print_debug {
        println!("{model_result:?}");
        return;
    }

    print_model_header(model_result);

    print_failing_tests(&model_result.path, &model_result.tests);

    let parameters_to_print = match model_config.print_level {
        PrintLevel::All => filter_parameters(&model_result.parameters, |_parameter| true),
        PrintLevel::Debug => filter_parameters(&model_result.parameters, |parameter| {
            parameter.should_print(result::PrintLevel::Debug)
        }),
        PrintLevel::Trace => filter_parameters(&model_result.parameters, |parameter| {
            parameter.should_print(result::PrintLevel::Trace)
        }),
        PrintLevel::Performance => filter_parameters(&model_result.parameters, |parameter| {
            parameter.should_print(result::PrintLevel::Performance)
        }),
    };

    let should_print_debug_info = model_config.print_level == PrintLevel::Debug;

    if parameters_to_print.is_empty() {
        let message = stylesheet::NO_PARAMETERS_MESSAGE.style("(No performance parameters found)");
        println!("{message}");
        return;
    }

    for parameter in parameters_to_print {
        print_parameter(parameter, should_print_debug_info);
    }
}

fn filter_parameters(
    parameters: &HashMap<String, result::Parameter>,
    arg: impl Fn(&&result::Parameter) -> bool,
) -> Vec<&result::Parameter> {
    parameters.values().filter(arg).collect()
}

fn print_failing_tests(model_path: &Path, model_tests: &[result::Test]) {
    let failing_tests = model_tests
        .iter()
        .filter_map(|test| match &test.result {
            result::TestResult::Failed { debug_info } => Some((test.expr_span, debug_info)),
            result::TestResult::Passed => None,
        })
        .collect::<Vec<_>>();

    if failing_tests.is_empty() {
        return;
    }

    let divider_line = divider_line();
    let file_contents = std::fs::read_to_string(model_path);

    let file_contents = match file_contents {
        Ok(file_contents) => file_contents,
        Err(e) => {
            let error_label = stylesheet::ERROR_COLOR.style("error");

            println!(
                "{error_label}: couldn't read `{}` - {}",
                model_path.display(),
                e
            );
            println!("{divider_line}");

            return;
        }
    };

    let tests_label = stylesheet::TESTS_FAIL_COLOR.style("FAILING TESTS");
    println!("{tests_label}");

    for (test_span, debug_info) in failing_tests {
        let test_start_offset = test_span.start().offset;
        let test_end_offset = test_span.end().offset;
        let test_expr_str = file_contents.get(test_start_offset..test_end_offset);

        let Some(test_expr_str) = test_expr_str else {
            let error_label = stylesheet::ERROR_COLOR.style("error");
            let test_start_line = test_span.start().line;
            let test_start_column = test_span.start().column;

            println!(
                "{error_label}: couldn't get test expression for test at line {test_start_line}, column {test_start_column}"
            );

            continue;
        };

        println!("{test_expr_str}");
        print_debug_info(debug_info);
    }

    println!("{divider_line}");
}

fn print_model_header(model_result: &result::Model) {
    let divider_line = divider_line();
    let model_label = stylesheet::MODEL_LABEL.style("Model");
    let tests_label = stylesheet::TESTS_LABEL.style("Tests");

    let test_count = model_result.tests.len();
    let passed_count = model_result
        .tests
        .iter()
        .filter(|test| test.passed())
        .count();

    let test_result_string = if passed_count == test_count {
        stylesheet::TESTS_PASS_COLOR.style("PASS")
    } else {
        stylesheet::TESTS_FAIL_COLOR.style("FAIL")
    };

    println!("{divider_line}");
    println!("{model_label}: {}", model_result.path.display());
    println!("{tests_label}: {passed_count}/{test_count} ({test_result_string})");
    println!("{divider_line}");
}

fn print_parameter(parameter: &result::Parameter, should_print_debug_info: bool) {
    let styled_ident = stylesheet::PARAMETER_IDENTIFIER.style(&parameter.ident);
    print!("{styled_ident} = ");

    print_value(&parameter.value);

    let styled_label = stylesheet::PARAMETER_LABEL.style(format!("# {}", parameter.label));
    println!("  {styled_label}");

    if should_print_debug_info && let Some(debug_info) = &parameter.debug_info {
        print_debug_info(debug_info);
    }
}

fn print_debug_info(debug_info: &result::DebugInfo) {
    for (dependency_name, dependency_value) in &debug_info.dependency_values {
        let indent = " ".repeat(2);
        let styled_dependency_name = stylesheet::PARAMETER_IDENTIFIER.style(dependency_name);
        print!("{indent}- {styled_dependency_name} = ");
        print_value(dependency_value);
        println!();
    }
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
