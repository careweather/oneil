use std::path::Path;

use anstream::{eprintln, print, println};
use indexmap::{IndexMap, IndexSet};
use oneil_runtime::output::{
    DebugInfo, Parameter, PrintLevel, TestResult, Value, reference::ModelReference,
};
use oneil_shared::{paths::ModelPath, span::Span};

use crate::{
    command::{PrintMode, Variable, VariableList},
    print_utils::{self, PrintUtilsConfig},
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
    pub recursive: bool,
    pub with_header: bool,
    pub with_test_report: bool,
    pub ignore_failing_tests: bool,
    pub print_utils_config: PrintUtilsConfig,
    /// Path to show in the "Run `oneil test <path>`" hint.
    /// When running a design file, this should be the design path.
    pub hint_path: Option<std::path::PathBuf>,
}

pub fn print_eval_result(
    model_result: ModelReference<'_>,
    expr_results: &IndexMap<&str, Value>,
    model_config: &ModelPrintConfig,
) {
    let test_info = get_model_tests(model_result, model_config.recursive, TestInfo::default());

    let divider_line = divider_line();

    if model_config.with_header {
        println!("{divider_line}");
        print_model_header(model_result.path(), &test_info);
    }

    if !model_config.ignore_failing_tests && test_info.passed_count != test_info.test_count {
        if model_config.with_test_report {
            println!("{divider_line}");
            print_failing_tests(&test_info, model_config.print_utils_config);
            println!("{divider_line}");
        } else {
            println!("{divider_line}");
            // Use hint_path if provided (e.g., when running a design file),
            // otherwise fall back to the model path
            let hint_path: &Path = model_config
                .hint_path
                .as_deref()
                .unwrap_or(model_result.path().as_path());
            print_test_failure_hint(hint_path, &test_info);
            println!("{divider_line}");
        }
    }

    if let Some(variables) = &model_config.variables {
        print_parameters_by_list(model_result, model_config, variables);
        print_expr_results(expr_results, model_config.print_utils_config);
    } else if !expr_results.is_empty() {
        print_expr_results(expr_results, model_config.print_utils_config);
    } else {
        print_parameters_by_filter(model_result, model_config);
    }
}

fn print_expr_results(expr_results: &IndexMap<&str, Value>, print_utils_config: PrintUtilsConfig) {
    for (expr, value) in expr_results {
        let styled_expr = stylesheet::EXPR.style(expr);
        print!("{styled_expr} = ");
        print_utils::print_value(value, print_utils_config);
        println!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPrintConfig {
    pub with_header: bool,
    pub recursive: bool,
    pub print_utils_config: PrintUtilsConfig,
}

pub fn print_test_results(eval_result: ModelReference<'_>, test_config: &TestPrintConfig) {
    let test_info = get_model_tests(eval_result, test_config.recursive, TestInfo::default());

    let divider_line = divider_line();
    println!("{divider_line}");

    if test_config.with_header {
        print_model_header(eval_result.path(), &test_info);
    }

    let mut visited = IndexSet::new();

    print_all_tests(eval_result, test_config, &mut visited);
}

#[derive(Default)]
struct TestInfo<'runtime> {
    pub test_count: usize,
    pub passed_count: usize,
    pub failed_tests: IndexMap<&'runtime ModelPath, Vec<(Span, &'runtime DebugInfo)>>,
}

fn get_model_tests<'runtime>(
    model_ref: ModelReference<'runtime>,
    recursive: bool,
    mut test_info: TestInfo<'runtime>,
) -> TestInfo<'runtime> {
    let tests = model_ref.tests();
    let test_count = tests.len();
    let failed_tests = tests
        .values()
        .filter_map(|test| match &test.result {
            TestResult::Failed { debug_info } => Some((test.expr_span.clone(), &**debug_info)),
            TestResult::Passed => None,
        })
        .collect::<Vec<_>>();

    test_info.test_count += test_count;
    test_info.passed_count += test_count - failed_tests.len();

    if !failed_tests.is_empty() {
        test_info
            .failed_tests
            .insert(model_ref.path(), failed_tests);
    }

    if recursive {
        model_ref
            .references()
            .values()
            .fold(test_info, |test_info, submodel| {
                get_model_tests(*submodel, recursive, test_info)
            })
    } else {
        test_info
    }
}

/// When the detailed test report is omitted, prints a short hint if any tests failed.
fn print_test_failure_hint(hint_path: &Path, test_info: &TestInfo<'_>) {
    let failed = test_info.test_count.saturating_sub(test_info.passed_count);

    let failed_message = format!(
        "{failed}/{} tests failed. Run `oneil test {}` to see results.",
        test_info.test_count,
        hint_path.display()
    );
    let styled_failed_message = stylesheet::TESTS_FAIL_COLOR.style(failed_message);

    println!("{styled_failed_message}");
}

fn print_failing_tests(test_info: &TestInfo<'_>, print_utils_config: PrintUtilsConfig) {
    if test_info.failed_tests.is_empty() {
        return;
    }

    let tests_label = stylesheet::TESTS_FAIL_COLOR.style("FAILING TESTS");
    println!("{tests_label}");

    for (index, (model_path, failing_tests)) in test_info.failed_tests.iter().enumerate() {
        print_model_failing_tests(model_path, failing_tests, print_utils_config);

        if index < test_info.failed_tests.len() - 1 {
            println!();
        }
    }
}

fn print_model_failing_tests(
    model_path: &ModelPath,
    failing_tests: &[(Span, &DebugInfo)],
    print_utils_config: PrintUtilsConfig,
) {
    let file_contents = std::fs::read_to_string(model_path.as_path());

    let file_contents = match file_contents {
        Ok(file_contents) => file_contents,
        Err(e) => {
            let error_label = stylesheet::ERROR_COLOR.style("error");

            println!(
                "{error_label}: couldn't read `{}` - {}",
                model_path.as_path().display(),
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

        if let Some(test_expr_str) = test_expr_str {
            let test_label = stylesheet::TEST_FAIL_EXPR_LABEL.style("test:");
            let test_expr_str = stylesheet::TEST_FAIL_EXPR_STR.style(test_expr_str);

            println!("{test_label} {test_expr_str}");
        } else {
            let error_label = stylesheet::ERROR_COLOR.style("error");
            let test_start_line = test_span.start().line;
            let test_start_column = test_span.start().column;

            println!(
                "{error_label}: couldn't get test expression for test at line {test_start_line}, column {test_start_column}"
            );
        }

        print_debug_info(debug_info, print_utils_config);
    }
}

fn print_model_header(model_path: &ModelPath, test_info: &TestInfo<'_>) {
    let model_label = stylesheet::MODEL_LABEL.style("Model");
    let tests_label = stylesheet::TESTS_LABEL.style("Tests");

    let test_count = test_info.test_count;
    let passed_count = test_info.passed_count;

    let test_result_string = if passed_count == test_count {
        stylesheet::TESTS_PASS_COLOR.style("PASS")
    } else {
        stylesheet::TESTS_FAIL_COLOR.style("FAIL")
    };

    println!("{model_label}: {}", model_path.as_path().display());
    println!("{tests_label}: {passed_count}/{test_count} ({test_result_string})");
}

fn print_model_path_header(model_path: &ModelPath) {
    let header = stylesheet::MODEL_PATH_HEADER.style(model_path.as_path().display());
    println!("{header}");
}

fn print_parameters_by_list(
    model_ref: ModelReference<'_>,
    model_config: &ModelPrintConfig,
    variables: &VariableList,
) {
    let ModelParametersToPrint {
        parameters: parameters_to_print,
        parameters_not_found,
    } = get_model_parameters_by_list(model_ref, variables);

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
        print_parameter(parameter, Some(parameter_name.as_str()), model_config);
    }
}

struct ModelParametersToPrint<'runtime> {
    pub parameters: IndexMap<String, &'runtime Parameter>,
    pub parameters_not_found: IndexSet<String>,
}

fn get_model_parameters_by_list<'runtime>(
    model_ref: ModelReference<'runtime>,
    variables: &VariableList,
) -> ModelParametersToPrint<'runtime> {
    let mut parameters = IndexMap::new();
    let mut parameters_not_found = IndexSet::new();

    for variable in variables.iter() {
        let variable_name = variable.to_string();
        let result = get_parameter_from_model(model_ref, variable);

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

fn get_parameter_from_model<'runtime>(
    model_ref: ModelReference<'runtime>,
    param: &Variable,
) -> Option<&'runtime Parameter> {
    let mut param_vec = param.to_vec();

    let parameter = param_vec.remove(0);

    return recurse(model_ref, parameter, param_vec);

    #[expect(
        clippy::items_after_statements,
        reason = "this is an internal recursive function, we keep it here for clarity"
    )]
    fn recurse(
        model_ref: ModelReference<'_>,
        parameter: String,
        mut param_vec: Vec<String>,
    ) -> Option<&Parameter> {
        // note that we're popping the last submodel since that's the
        // top-most submodel
        let Some(submodel) = param_vec.pop() else {
            return model_ref.parameters().get(parameter.as_str()).map(|p| &**p);
        };

        // `ModelReference::references` is the unified eval-time child map
        // populated by `seed_subtree`: it merges every named child of the
        // host (cross-file `reference` imports, owned `submodel`s, and
        // extraction-list aliases) keyed by their reference name. A single
        // lookup against this map therefore navigates any of the three
        // import flavors without us having to branch on what kind of
        // child it is.
        let references = model_ref.references();
        let model = references
            .iter()
            .find(|(name, _)| name.as_str() == submodel.as_str())
            .map(|(_, m)| *m)?;

        recurse(model, parameter, param_vec)
    }
}

fn print_parameters_by_filter(model_ref: ModelReference<'_>, model_config: &ModelPrintConfig) {
    let parameters_to_print =
        get_model_parameters_by_filter(model_ref, model_config.print_mode, model_config.recursive);

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

    let only_top_model_is_printed =
        parameters_to_print.len() == 1 && parameters_to_print.contains_key(model_ref.path());

    for (index, (path, parameters)) in parameters_to_print.iter().enumerate() {
        if !only_top_model_is_printed {
            print_model_path_header(path);
        }

        for parameter in parameters {
            print_parameter(parameter, None, model_config);
        }

        if index < parameters_to_print.len() - 1 {
            println!();
        }
    }
}

fn get_model_parameters_by_filter(
    model_ref: ModelReference<'_>,
    print_level: PrintMode,
    recursive: bool,
) -> IndexMap<&ModelPath, Vec<&Parameter>> {
    return recurse(model_ref, print_level, recursive, IndexMap::new());

    #[expect(
        clippy::items_after_statements,
        reason = "this is an internal recursive function, we keep it here for clarity"
    )]
    fn recurse<'runtime>(
        model_ref: ModelReference<'runtime>,
        print_level: PrintMode,
        recursive: bool,
        mut parameters: IndexMap<&'runtime ModelPath, Vec<&'runtime Parameter>>,
    ) -> IndexMap<&'runtime ModelPath, Vec<&'runtime Parameter>> {
        let model_parameters = model_ref.parameters();
        let parameters_to_print: Vec<_> = match print_level {
            PrintMode::All => model_parameters
                .values()
                .filter(|_parameter| true)
                .map(|p| &**p)
                .collect(),
            PrintMode::Trace => model_parameters
                .values()
                .filter(|parameter| parameter.should_print(PrintLevel::Trace))
                .map(|p| &**p)
                .collect(),
            PrintMode::Performance => model_parameters
                .values()
                .filter(|parameter| parameter.should_print(PrintLevel::Performance))
                .map(|p| &**p)
                .collect(),
        };

        if !parameters_to_print.is_empty() {
            parameters.insert(model_ref.path(), parameters_to_print);
        }

        if recursive {
            // `ModelReference::references` is the unified eval-time child
            // map (cross-file references + owned submodels + extraction
            // aliases), so iterating it walks every named child once.
            let references = model_ref.references();
            references.values().fold(parameters, |parameters, model| {
                recurse(*model, print_level, recursive, parameters)
            })
        } else {
            parameters
        }
    }
}

fn print_parameter(
    parameter: &Parameter,
    parameter_name: Option<&str>, // if None, use the parameter identifier as the name
    model_config: &ModelPrintConfig,
) {
    let parameter_name = parameter_name.unwrap_or(parameter.ident.as_str());
    let styled_ident = stylesheet::PARAMETER_IDENTIFIER.style(parameter_name);
    print!("{styled_ident} = ");

    print_utils::print_value(&parameter.value, model_config.print_utils_config);

    let styled_label = stylesheet::PARAMETER_LABEL.style(format!("# {}", parameter.label.as_str()));
    println!("  {styled_label}");

    if model_config.print_debug_info
        && let Some(debug_info) = &parameter.debug_info
    {
        print_debug_info(debug_info, model_config.print_utils_config);
    }
}

fn print_debug_info(debug_info: &DebugInfo, print_utils_config: PrintUtilsConfig) {
    let indent = 2;
    for (dependency_name, dependency_value) in &debug_info.builtin_dependency_values {
        print_variable_value(
            dependency_name.as_str(),
            dependency_value,
            indent,
            print_utils_config,
        );
    }
    for (dependency_name, dependency_value) in &debug_info.parameter_dependency_values {
        print_variable_value(
            dependency_name.as_str(),
            dependency_value,
            indent,
            print_utils_config,
        );
    }
    for ((reference_name, parameter_name), dependency_value) in
        &debug_info.external_dependency_values
    {
        let variable_name = format!("{}.{}", parameter_name.as_str(), reference_name.as_str());
        print_variable_value(&variable_name, dependency_value, indent, print_utils_config);
    }
}

fn print_variable_value(
    name: &str,
    value: &Value,
    indent: usize,
    print_utils_config: PrintUtilsConfig,
) {
    let indent = " ".repeat(indent);
    let styled_dependency_name = stylesheet::PARAMETER_IDENTIFIER.style(name);
    print!("{indent}- {styled_dependency_name} = ");
    print_utils::print_value(value, print_utils_config);
    println!();
}

fn print_all_tests<'runtime>(
    model_ref: ModelReference<'runtime>,
    test_config: &TestPrintConfig,
    visited: &mut IndexSet<&'runtime ModelPath>,
) {
    let recursive = test_config.recursive;
    let model_path = model_ref.path();
    let tests = model_ref.tests();

    if visited.contains(model_path) {
        return;
    }

    visited.insert(model_path);

    if recursive && !tests.is_empty() {
        print_model_path_header(model_path);
    }

    let file_contents = std::fs::read_to_string(model_path.as_path());

    let file_contents = match file_contents {
        Ok(file_contents) => file_contents,
        Err(e) => {
            let error_label = stylesheet::ERROR_COLOR.style("error");

            println!(
                "{error_label}: couldn't read `{}` - {}",
                model_path.as_path().display(),
                e
            );

            return;
        }
    };

    for test in tests.values() {
        let test_start_offset = test.expr_span.clone().start().offset;
        let test_end_offset = test.expr_span.clone().end().offset;
        let test_expr_str = file_contents.get(test_start_offset..test_end_offset);

        if let Some(test_expr_str) = test_expr_str {
            let test_label = stylesheet::TEST_EXPR_LABEL.style("test:");
            let test_expr_str = stylesheet::TEST_EXPR_STR.style(test_expr_str);
            println!("{test_label} {test_expr_str}");
        } else {
            let error_label = stylesheet::ERROR_COLOR.style("error");
            let test_start_line = test.expr_span.clone().start().line;
            let test_start_column = test.expr_span.clone().start().column;

            println!(
                "{error_label}: couldn't get test expression for test at line {test_start_line}, column {test_start_column}"
            );
        }

        match &test.result {
            TestResult::Passed => {
                let test_result_str = stylesheet::TESTS_PASS_COLOR.style("PASS");
                println!("  Result: {test_result_str}");
            }
            TestResult::Failed { debug_info } => {
                let test_result_str = stylesheet::TESTS_FAIL_COLOR.style("FAIL");
                println!("  Result: {test_result_str}");
                print_debug_info(debug_info, test_config.print_utils_config);
            }
        }
    }

    if recursive {
        if !tests.is_empty() {
            println!();
        }

        for reference in model_ref.references().values() {
            print_all_tests(*reference, test_config, visited);
        }
    }
}

fn divider_line() -> String {
    "─".repeat(80)
}
