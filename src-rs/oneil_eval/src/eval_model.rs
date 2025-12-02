use std::collections::{HashMap, HashSet};

use oneil_ir as ir;

use crate::{context::EvalContext, eval_parameter};

pub fn eval_model(model: &ir::Model, mut context: EvalContext) -> EvalContext {
    let python_imports = model.get_python_imports();
    let references = model.get_references();

    context.activate_python_imports(python_imports);
    context.activate_references(references);

    let submodels = model.get_submodels();
    for (submodel_name, submodel_import) in submodels {
        context.add_submodel(submodel_name.as_str(), submodel_import.path());
    }

    let parameters = model.get_parameters();
    let evaluation_order = get_evaluation_order(parameters);

    for parameter_name in evaluation_order {
        let parameter = parameters
            .get(&parameter_name)
            .expect("parameter should exist because it comes from the keys of the parameters map");

        let value = eval_parameter(parameter, &context);
        context.add_parameter_result(parameter_name, value);
    }

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

        (evaluation_order, visited) = get_parameter_dependencies(
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

fn get_parameter_dependencies(
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

        let dependency_parameter = parameters.get(dependency).expect("dependency should exist");

        (evaluation_order, visited) = get_parameter_dependencies(
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
