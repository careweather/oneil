use std::{collections::HashMap, path::Path};

use anstream::{print, println};
use oneil_eval::{
    result,
    value::{self, Value},
};
use oneil_shared::span::Span;

use crate::{command::PrintParams, stylesheet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPrintConfig {
    pub print_level: PrintParams,
    pub top_model_only: bool,
}

pub fn divider_line() -> String {
    "─".repeat(80)
}

pub fn print(model_result: &result::Model, print_debug_info: bool, model_config: ModelPrintConfig) {
    let parameters_to_print = get_model_parameters(
        model_result,
        model_config.print_level,
        model_config.top_model_only,
        HashMap::new(),
    );

    let test_info = get_model_tests(
        model_result,
        model_config.top_model_only,
        TestInfo::default(),
    );

    print_model_header(&model_result.path, &test_info);

    print_failing_tests(&test_info);

    if parameters_to_print.is_empty() {
        let message = stylesheet::NO_PARAMETERS_MESSAGE.style("(No performance parameters found)");
        println!("{message}");
        return;
    }

    let only_top_model_is_printed = parameters_to_print.len() == 1
        && parameters_to_print.contains_key(model_result.path.as_path());

    for (index, (path, parameters)) in parameters_to_print.iter().enumerate() {
        if !only_top_model_is_printed {
            print_model_path_header(path);
        }

        for parameter in parameters {
            print_parameter(parameter, print_debug_info);
        }

        if index < parameters_to_print.len() - 1 {
            println!();
        }
    }
}

fn get_model_parameters<'a>(
    model_result: &'a result::Model,
    print_level: PrintParams,
    top_model_only: bool,
    mut models: HashMap<&'a Path, Vec<&'a result::Parameter>>,
) -> HashMap<&'a Path, Vec<&'a result::Parameter>> {
    let parameters_to_print = match print_level {
        PrintParams::All => filter_parameters(&model_result.parameters, |_parameter| true),
        PrintParams::Trace => filter_parameters(&model_result.parameters, |parameter| {
            parameter.should_print(result::PrintLevel::Trace)
        }),
        PrintParams::Performance => filter_parameters(&model_result.parameters, |parameter| {
            parameter.should_print(result::PrintLevel::Performance)
        }),
    };

    if !parameters_to_print.is_empty() {
        models.insert(&model_result.path, parameters_to_print);
    }

    if top_model_only {
        models
    } else {
        model_result
            .submodels
            .values()
            .fold(models, |models, submodel| {
                get_model_parameters(submodel, print_level, top_model_only, models)
            })
    }
}

#[derive(Default)]
struct TestInfo<'a> {
    pub test_count: usize,
    pub passed_count: usize,
    // this is a vec in order to preserve the order of the models
    pub failed_tests: Vec<(&'a Path, Vec<(Span, &'a result::DebugInfo)>)>,
}

fn get_model_tests<'a>(
    model_result: &'a result::Model,
    top_model_only: bool,
    mut test_info: TestInfo<'a>,
) -> TestInfo<'a> {
    let test_count = model_result.tests.len();
    let failed_tests = model_result
        .tests
        .iter()
        .filter_map(|test| match &test.result {
            result::TestResult::Failed { debug_info } => Some((test.expr_span, debug_info)),
            result::TestResult::Passed => None,
        })
        .collect::<Vec<_>>();

    test_info.test_count += test_count;
    test_info.passed_count += test_count - failed_tests.len();

    if !failed_tests.is_empty() {
        test_info
            .failed_tests
            .push((&model_result.path, failed_tests));
    }

    if top_model_only {
        test_info
    } else {
        model_result
            .submodels
            .values()
            .fold(test_info, |test_info, submodel| {
                get_model_tests(submodel, top_model_only, test_info)
            })
    }
}

fn filter_parameters(
    parameters: &HashMap<String, result::Parameter>,
    arg: impl Fn(&&result::Parameter) -> bool,
) -> Vec<&result::Parameter> {
    parameters.values().filter(arg).collect()
}

fn print_failing_tests(test_info: &TestInfo<'_>) {
    if test_info.failed_tests.is_empty() {
        return;
    }

    let divider_line = divider_line();

    let tests_label = stylesheet::TESTS_FAIL_COLOR.style("FAILING TESTS");
    println!("{tests_label}");

    for (index, (model_path, failing_tests)) in test_info.failed_tests.iter().enumerate() {
        print_model_failing_tests(model_path, failing_tests);

        if index < test_info.failed_tests.len() - 1 {
            println!();
        }
    }

    println!("{divider_line}");
}

fn print_model_failing_tests(model_path: &Path, failing_tests: &[(Span, &result::DebugInfo)]) {
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

            return;
        }
    };

    print_model_path_header(model_path);

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
}

fn print_model_header(model_path: &Path, test_info: &TestInfo<'_>) {
    let divider_line = divider_line();
    let model_label = stylesheet::MODEL_LABEL.style("Model");
    let tests_label = stylesheet::TESTS_LABEL.style("Tests");

    let test_count = test_info.test_count;
    let passed_count = test_info.passed_count;

    let test_result_string = if passed_count == test_count {
        stylesheet::TESTS_PASS_COLOR.style("PASS")
    } else {
        stylesheet::TESTS_FAIL_COLOR.style("FAIL")
    };

    println!("{divider_line}");
    println!("{model_label}: {}", model_path.display());
    println!("{tests_label}: {passed_count}/{test_count} ({test_result_string})");
    println!("{divider_line}");
}

fn print_model_path_header(model_path: &Path) {
    let header = stylesheet::MODEL_PATH_HEADER.style(model_path.display());
    println!("{header}");
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
    let indent = 2;
    for (dependency_name, dependency_value) in &debug_info.builtin_dependency_values {
        print_variable_value(dependency_name, dependency_value, indent);
    }
    for (dependency_name, dependency_value) in &debug_info.parameter_dependency_values {
        print_variable_value(dependency_name, dependency_value, indent);
    }
    for ((reference_name, parameter_name), dependency_value) in
        &debug_info.external_dependency_values
    {
        let variable_name = format!("{parameter_name}.{reference_name}");
        print_variable_value(&variable_name, dependency_value, indent);
    }
}

fn print_variable_value(name: &str, value: &Value, indent: usize) {
    let indent = " ".repeat(indent);
    let styled_dependency_name = stylesheet::PARAMETER_IDENTIFIER.style(name);
    print!("{indent}- {styled_dependency_name} = ");
    print_value(value);
    println!();
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
