use std::collections::{HashMap, HashSet};

use oneil_ir as ir;

use crate::{
    EvalError, builtin::BuiltinFunction, context::EvalContext, eval_expr, eval_parameter,
    value::Value,
};

#[expect(clippy::missing_panics_doc, reason = "the panic should never happen")]
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

        let value = eval_parameter(parameter, &context);
        context.add_parameter_result(
            parameter_name.as_str().to_string(),
            // TODO: for now, we just discard the is_db flag, but we need to handle it eventually
            value.map(|(value, _)| value),
        );
    }

    // Evaluate tests
    let tests = model.get_tests();
    for test in tests.values() {
        let value = eval_test(test, &context);
        context.add_test_result(value);
    }

    context.clear_active_model();

    context
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
