use std::collections::{HashMap, HashSet};

use oneil_ir as ir;

use crate::{
    EvalError,
    builtin::BuiltinFunction,
    context::EvalContext,
    eval_expr,
    eval_parameter::{self, TypecheckInfo},
    result,
    value::Value,
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

        let parameter_result = value
            .map(|(value, typecheck_info)| parameter_result_from(value, typecheck_info, parameter));

        context.add_parameter_result(parameter_name.as_str().to_string(), parameter_result);
    }

    // Evaluate tests
    let tests = model.get_tests();
    for test in tests.values() {
        let value = eval_test(test, &context);
        let test_result = value.map(|value| result::Test {
            value,
            expr_span: test.span(),
        });
        context.add_test_result(test_result);
    }

    context.clear_active_model();

    context
}

fn parameter_result_from(
    value: Value,
    typecheck_info: TypecheckInfo,
    parameter: &ir::Parameter,
) -> result::Parameter {
    let unit = match typecheck_info {
        TypecheckInfo::Number { sized_unit } => Some(sized_unit),
        TypecheckInfo::String | TypecheckInfo::Boolean => None,
    };

    let trace = match parameter.trace_level() {
        ir::TraceLevel::None => result::TraceLevel::None,
        ir::TraceLevel::Trace => result::TraceLevel::Trace,
        ir::TraceLevel::Debug => result::TraceLevel::Debug,
    };

    let dependencies = parameter
        .dependencies()
        .iter()
        .map(|dependency| dependency.as_str().to_string())
        .collect();

    result::Parameter {
        ident: parameter.name().as_str().to_string(),
        label: parameter.label().as_str().to_string(),
        value,
        unit,
        is_performance: parameter.is_performance(),
        trace,
        dependencies,
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
    for dependency in parameter.dependencies() {
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
) -> Result<Value, Vec<EvalError>> {
    eval_expr(test.test_expr(), context)
}
