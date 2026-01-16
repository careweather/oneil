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
    let (print_level, debug_info) = match parameter.trace_level() {
        ir::TraceLevel::Debug if parameter.is_performance() => {
            let builtin_dependency_values =
                get_builtin_dependency_values(parameter.dependencies().builtin(), context);
            let parameter_dependency_values =
                get_parameter_dependency_values(parameter.dependencies().parameter(), context);
            let external_dependency_values =
                get_external_dependency_values(parameter.dependencies().external(), context);
            (
                result::PrintLevel::Performance,
                Some(result::DebugInfo {
                    builtin_dependency_values,
                    parameter_dependency_values,
                    external_dependency_values,
                }),
            )
        }
        ir::TraceLevel::Trace | ir::TraceLevel::None if parameter.is_performance() => {
            (result::PrintLevel::Performance, None)
        }
        ir::TraceLevel::Debug => {
            let builtin_dependency_values =
                get_builtin_dependency_values(parameter.dependencies().builtin(), context);
            let parameter_dependency_values =
                get_parameter_dependency_values(parameter.dependencies().parameter(), context);
            let external_dependency_values =
                get_external_dependency_values(parameter.dependencies().external(), context);
            (
                result::PrintLevel::Trace,
                Some(result::DebugInfo {
                    builtin_dependency_values,
                    parameter_dependency_values,
                    external_dependency_values,
                }),
            )
        }
        ir::TraceLevel::Trace => (result::PrintLevel::Trace, None),
        ir::TraceLevel::None => (result::PrintLevel::None, None),
    };

    result::Parameter {
        ident: parameter.name().as_str().to_string(),
        label: parameter.label().as_str().to_string(),
        value,
        print_level,
        debug_info,
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
            parameter.dependencies(),
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
    parameter_dependencies: &ir::Dependencies,
    mut visited: HashSet<ir::ParameterName>,
    mut evaluation_order: Vec<ir::ParameterName>,
    parameters: &HashMap<ir::ParameterName, ir::Parameter>,
) -> (Vec<ir::ParameterName>, HashSet<ir::ParameterName>) {
    for dependency in parameter_dependencies.parameter().keys() {
        if visited.contains(dependency) {
            continue;
        }

        let Some(dependency_parameter) = parameters.get(dependency) else {
            // dependency is a builtin value, so we don't need to visit it
            continue;
        };

        (evaluation_order, visited) = process_parameter_dependencies(
            dependency,
            dependency_parameter.dependencies(),
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
            let builtin_dependency_values =
                get_builtin_dependency_values(test.dependencies().builtin(), context);
            let parameter_dependency_values =
                get_parameter_dependency_values(test.dependencies().parameter(), context);
            let external_dependency_values =
                get_external_dependency_values(test.dependencies().external(), context);

            let debug_info = result::DebugInfo {
                builtin_dependency_values,
                parameter_dependency_values,
                external_dependency_values,
            };
            Ok(result::Test {
                result: result::TestResult::Failed { debug_info },
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

/// Gets the values of the builtin dependencies for debug reporting purposes.
fn get_builtin_dependency_values<F: BuiltinFunction>(
    dependencies: &HashMap<ir::Identifier, Span>,
    context: &EvalContext<F>,
) -> HashMap<String, Value> {
    dependencies
        .keys()
        .map(|dependency| {
            let value = context.lookup_builtin_variable(dependency);
            (dependency.as_str().to_string(), value)
        })
        .collect::<HashMap<_, _>>()
}

/// Gets the values of the dependencies for debug reporting purposes.
///
/// This should only be called on expressions that have already been evaluated successfully.
///
/// # Panics
///
/// This function will panic if any of the dependencies are not found.
fn get_parameter_dependency_values<F: BuiltinFunction>(
    dependencies: &HashMap<ir::ParameterName, Span>,
    context: &EvalContext<F>,
) -> HashMap<String, Value> {
    dependencies
        .iter()
        .map(|(dependency, dependency_span)| {
            let value = context
                .lookup_parameter_value(dependency, *dependency_span)
                .expect("dependency should be found because the expression evaluated successfully");

            (dependency.as_str().to_string(), value)
        })
        .collect::<HashMap<_, _>>()
}

/// Gets the values of the external dependencies for debug reporting purposes.
///
/// This should only be called on expressions that have already been evaluated successfully.
///
/// # Panics
///
/// This function will panic if any of the dependencies are not found.
fn get_external_dependency_values<F: BuiltinFunction>(
    dependencies: &HashMap<(ir::ReferenceName, ir::ParameterName), (ir::ModelPath, Span)>,
    context: &EvalContext<F>,
) -> HashMap<(String, String), Value> {
    dependencies
        .iter()
        .map(
            |((reference_name, parameter_name), (model_path, dependency_span))| {
                let value = context.lookup_model_parameter_value(
                    model_path,
                    parameter_name,
                    *dependency_span,
                );

                let reference_name = reference_name.as_str().to_string();
                let parameter_name = parameter_name.as_str().to_string();
                let value = value.expect(
                    "dependency should be found because the expression evaluated successfully",
                );

                ((reference_name, parameter_name), value)
            },
        )
        .collect::<HashMap<_, _>>()
}
