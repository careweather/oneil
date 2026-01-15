use std::collections::{HashMap, HashSet};

use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    EvalError, builtin::BuiltinFunction, context::EvalContext, error::ExpectedType, eval_expr,
    eval_parameter, result, value::Value,
};

/// Evaluates a model and returns the context with the results of the model.
#[expect(
    clippy::missing_panics_doc,
    reason = "the panic is only caused by breaking an internal invariant"
)]
#[must_use]
pub fn eval_model<F: BuiltinFunction>(
    model_path: &ir::ModelPath,
    model: &ir::Model,
    mut context: EvalContext<F>,
) -> EvalContext<F> {
    // Set the current model
    let model_path = model_path.as_ref().to_path_buf();
    context.set_active_model(model_path);

    // Bring Python imports into scope
    let python_imports = model.get_python_imports();
    context.clear_active_python_imports();
    for python_import in python_imports.values() {
        let path = python_import.import_path().as_ref().to_path_buf();
        context.activate_python_import(path);
    }

    // Bring references into scope
    let references = model.get_references();
    for reference in references.values() {
        let path = reference.path().as_ref().to_path_buf();
        context.activate_reference(path);
    }

    // Add submodels to the current model
    let submodels = model.get_submodels();
    for (submodel_name, submodel_import) in submodels {
        context.add_submodel(submodel_name.as_str(), submodel_import.path());
    }

    // Evaluate parameters
    let parameters = model.get_parameters();
    let evaluation_order = get_evaluation_order(parameters);

    for parameter_name in evaluation_order {
        let parameter = parameters
            .get(&parameter_name)
            .expect("parameter should exist because it comes from the keys of the parameters map");

        let value = eval_parameter::eval_parameter(parameter, &context);

        let parameter_result = value.map(|value| parameter_result_from(value, parameter, &context));

        context.add_parameter_result(parameter_name.as_str().to_string(), parameter_result);
    }

    // Evaluate tests
    let tests = model.get_tests();
    for test in tests.values() {
        let test_result = eval_test(test, &context);
        context.add_test_result(test_result);
    }

    context.clear_active_model();

    context
}

fn parameter_result_from<F: BuiltinFunction>(
    value: Value,
    parameter: &ir::Parameter,
    context: &EvalContext<F>,
) -> result::Parameter {
    let print_level = match parameter.trace_level() {
        ir::TraceLevel::Debug if parameter.is_performance() => result::PrintLevel::PerformanceDebug,
        ir::TraceLevel::Trace | ir::TraceLevel::None if parameter.is_performance() => {
            result::PrintLevel::Performance
        }
        ir::TraceLevel::Debug => result::PrintLevel::Debug,
        ir::TraceLevel::Trace => result::PrintLevel::Trace,
        ir::TraceLevel::None => result::PrintLevel::None,
    };

    let dependency_values = get_dependency_values(parameter.dependencies(), context);

    result::Parameter {
        ident: parameter.name().as_str().to_string(),
        label: parameter.label().as_str().to_string(),
        value,
        print_level,
        dependency_values,
    }
}

fn get_evaluation_order(
    parameters: &HashMap<ir::ParameterName, ir::Parameter>,
) -> Vec<ir::ParameterName> {
    let mut evaluation_order = Vec::new();
    let mut visited = HashSet::new();

    for (parameter_name, parameter) in parameters {
        if visited.contains(parameter_name) {
            continue;
        }

        (evaluation_order, visited) = process_parameter_dependencies(
            parameter_name,
            parameter,
            visited,
            evaluation_order,
            parameters,
        );

        evaluation_order.push(parameter_name.clone());
        visited.insert(parameter_name.clone());
    }

    evaluation_order
}

fn process_parameter_dependencies(
    parameter_name: &ir::ParameterName,
    parameter: &ir::Parameter,
    mut visited: HashSet<ir::ParameterName>,
    mut evaluation_order: Vec<ir::ParameterName>,
    parameters: &HashMap<ir::ParameterName, ir::Parameter>,
) -> (Vec<ir::ParameterName>, HashSet<ir::ParameterName>) {
    for dependency in parameter.dependencies().keys() {
        if visited.contains(dependency) {
            continue;
        }

        let Some(dependency_parameter) = parameters.get(dependency) else {
            // dependency is a builtin value, so we don't need to visit it
            continue;
        };

        (evaluation_order, visited) = process_parameter_dependencies(
            dependency,
            dependency_parameter,
            visited,
            evaluation_order,
            parameters,
        );
    }

    evaluation_order.push(parameter_name.clone());
    visited.insert(parameter_name.clone());

    (evaluation_order, visited)
}

fn eval_test<F: BuiltinFunction>(
    test: &ir::Test,
    context: &EvalContext<F>,
) -> Result<result::Test, Vec<EvalError>> {
    let (test_result, expr_span) = eval_expr(test.expr(), context)?;

    match test_result {
        Value::Boolean(true) => Ok(result::Test {
            result: result::TestResult::Passed,
            expr_span: *expr_span,
        }),
        Value::Boolean(false) => {
            let dependency_values = get_dependency_values(test.dependencies(), context);
            Ok(result::Test {
                result: result::TestResult::Failed { dependency_values },
                expr_span: *expr_span,
            })
        }
        Value::String(_) | Value::Number(_) | Value::MeasuredNumber(_) => {
            Err(vec![EvalError::InvalidType {
                expected_type: ExpectedType::Boolean,
                found_type: test_result.type_(),
                found_span: *expr_span,
            }])
        }
    }
}

/// Gets the values of the dependencies for test reporting purposes.
fn get_dependency_values<F: BuiltinFunction>(
    dependencies: &HashMap<ir::ParameterName, Span>,
    context: &EvalContext<F>,
) -> HashMap<String, Value> {
    dependencies
        .iter()
        .map(|(dependency, dependency_span)| {
            let value = context
                .lookup_parameter_value(dependency, *dependency_span)
                .expect(
                    "dependency should be found because the test expression resolved successfully",
                );

            (dependency.as_str().to_string(), value)
        })
        .collect::<HashMap<_, _>>()
}
