use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anstream::{eprintln, print, println};
use oneil_eval::{
    result,
    value::{self, Value},
};
use oneil_shared::span::Span;

use crate::{
    command::{PrintMode, VariableList},
    stylesheet,
};

#[expect(
    clippy::struct_excessive_bools,
    reason = "this is a configuration struct for printing model results"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPrintConfig {
    pub print_mode: PrintMode,
    pub print_debug_info: bool,
    pub variables: Option<VariableList>,
    pub top_model_only: bool,
    pub no_header: bool,
    pub no_test_report: bool,
    pub no_parameters: bool,
}

pub fn divider_line() -> String {
    "─".repeat(80)
}

pub fn print(model_result: &result::Model, model_config: ModelPrintConfig) {
    let test_info = get_model_tests(
        model_result,
        model_config.top_model_only,
        TestInfo::default(),
    );

    let divider_line = divider_line();
    println!("{divider_line}");

    if !model_config.no_header {
        print_model_header(&model_result.path, &test_info);
    }

    if !model_config.no_test_report {
        print_failing_tests(&test_info);
    }

    if !model_config.no_parameters {
        if let Some(variables) = model_config.variables {
            print_parameters_by_list(model_result, model_config.print_debug_info, variables);
        } else {
            print_parameters_by_filter(model_result, model_config.print_debug_info, &model_config);
        }
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

    println!("{model_label}: {}", model_path.display());
    println!("{tests_label}: {passed_count}/{test_count} ({test_result_string})");
    println!("{divider_line}");
}

fn print_model_path_header(model_path: &Path) {
    let header = stylesheet::MODEL_PATH_HEADER.style(model_path.display());
    println!("{header}");
}

fn print_parameters_by_list(
    model_result: &result::Model,
    print_debug_info: bool,
    variables: VariableList,
) {
    let ModelParametersToPrint {
        parameters: parameters_to_print,
        parameters_not_found,
    } = get_model_parameters_by_list(model_result, variables);

    if parameters_to_print.is_empty() && parameters_not_found.is_empty() {
        let message = stylesheet::NO_PARAMETERS_MESSAGE.style("(No parameters found)");
        eprintln!("{message}");
        return;
    }

    for parameter_name in parameters_not_found {
        let message = stylesheet::ERROR_COLOR
            .bold()
            .style(format!("Parameter not found: \"{parameter_name}\""));
        eprintln!("{message}");
    }

    for (parameter_name, parameter) in parameters_to_print {
        let styled_parameter_name = stylesheet::PARAMETERS_NAME_LABEL.style(parameter_name);
        print!("{styled_parameter_name}: ");
        print_parameter(parameter, print_debug_info);
    }
}

struct ModelParametersToPrint<'a> {
    pub parameters: HashMap<String, &'a result::Parameter>,
    pub parameters_not_found: HashSet<String>,
}

fn get_model_parameters_by_list(
    model_result: &result::Model,
    variables: VariableList,
) -> ModelParametersToPrint<'_> {
    let mut parameters = HashMap::new();
    let mut parameters_not_found = HashSet::new();

    for variable in variables.into_iter() {
        let variable_name = variable.to_string();
        let result = get_parameter_from_model(model_result, variable);

        match result {
            Some(variable_value) => {
                parameters.insert(variable_name, variable_value);
            }
            None => {
                parameters_not_found.insert(variable_name);
            }
        }
    }

    ModelParametersToPrint {
        parameters,
        parameters_not_found,
    }
}

fn get_parameter_from_model(
    model_result: &result::Model,
    param: crate::command::Variable,
) -> Option<&result::Parameter> {
    let mut param_vec = param.into_vec();

    let parameter = param_vec.remove(0);

    return recurse(model_result, parameter, param_vec);

    #[expect(
        clippy::items_after_statements,
        reason = "this is an internal recursive function, we keep it here for clarity"
    )]
    fn recurse(
        model: &result::Model,
        parameter: String,
        mut submodels: Vec<String>,
    ) -> Option<&result::Parameter> {
        // note that we're popping the last submodel since that's the
        // top-most submodel
        let Some(submodel) = submodels.pop() else {
            return model.parameters.get(&parameter);
        };

        let submodel = model.submodels.get(&submodel)?;

        recurse(submodel, parameter, submodels)
    }
}

fn print_parameters_by_filter(
    model_result: &result::Model,
    print_debug_info: bool,
    model_config: &ModelPrintConfig,
) {
    let parameters_to_print = get_model_parameters_by_filter(
        model_result,
        model_config.print_mode,
        model_config.top_model_only,
    );

    let parameter_kind = match model_config.print_mode {
        PrintMode::Trace => "trace ",
        PrintMode::Performance => "performance ",
        PrintMode::All => "",
    };

    if parameters_to_print.is_empty() {
        let message = stylesheet::NO_PARAMETERS_MESSAGE
            .style(format!("(No {parameter_kind}parameters found)"));
        eprintln!("{message}");
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

fn get_model_parameters_by_filter(
    model_result: &result::Model,
    print_level: PrintMode,
    top_model_only: bool,
) -> HashMap<&Path, Vec<&result::Parameter>> {
    return recurse(model_result, print_level, top_model_only, HashMap::new());

    #[expect(
        clippy::items_after_statements,
        reason = "this is an internal recursive function, we keep it here for clarity"
    )]
    fn recurse<'a>(
        model_result: &'a result::Model,
        print_level: PrintMode,
        top_model_only: bool,
        mut parameters: HashMap<&'a Path, Vec<&'a result::Parameter>>,
    ) -> HashMap<&'a Path, Vec<&'a result::Parameter>> {
        let parameters_to_print: Vec<_> = match print_level {
            PrintMode::All => model_result
                .parameters
                .values()
                .filter(|_parameter| true)
                .collect(),
            PrintMode::Trace => model_result
                .parameters
                .values()
                .filter(|parameter| parameter.should_print(result::PrintLevel::Trace))
                .collect(),
            PrintMode::Performance => model_result
                .parameters
                .values()
                .filter(|parameter| parameter.should_print(result::PrintLevel::Performance))
                .collect(),
        };

        if !parameters_to_print.is_empty() {
            parameters.insert(&model_result.path, parameters_to_print);
        }

        if top_model_only {
            parameters
        } else {
            model_result
                .submodels
                .values()
                .fold(parameters, |parameters, submodel| {
                    recurse(submodel, print_level, top_model_only, parameters)
                })
        }
    }
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
