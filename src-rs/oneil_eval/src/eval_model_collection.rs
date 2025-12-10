use std::collections::{HashMap, HashSet};

use oneil_ir as ir;

use crate::{
    builtin::{BuiltinFunction, BuiltinMap},
    context::EvalContext,
    eval_model::eval_model,
};

/// Evaluates a model collection and returns the context with the results of the models.
pub fn eval_model_collection<F: BuiltinFunction>(
    model_collection: &ir::ModelCollection,
    builtins: &impl BuiltinMap<F>,
) -> EvalContext<F> {
    eval_model_collection_with_context(model_collection, EvalContext::new(builtins))
}

pub fn eval_model_collection_with_context<F: BuiltinFunction>(
    model_collection: &ir::ModelCollection,
    mut context: EvalContext<F>,
) -> EvalContext<F> {
    for python_path in model_collection.get_python_imports() {
        context.load_python_import(python_path.as_ref().to_path_buf());
    }

    let models = model_collection.get_models();
    let evaluation_order = get_evaluation_order(models);

    for model_path in evaluation_order {
        let model = models
            .get(&model_path)
            .expect("model should exist because it comes from the keys of the models map");

        context = eval_model(&model_path, model, context);
    }

    context
}

fn get_evaluation_order(models: &HashMap<ir::ModelPath, ir::Model>) -> Vec<ir::ModelPath> {
    let mut evaluation_order = Vec::new();
    let mut visited = HashSet::new();

    for (model_path, model) in models {
        if visited.contains(model_path) {
            continue;
        }

        (evaluation_order, visited) =
            process_model_dependencies(model_path, model, visited, evaluation_order, models);
    }

    evaluation_order
}

fn process_model_dependencies(
    model_path: &ir::ModelPath,
    model: &ir::Model,
    mut visited: HashSet<ir::ModelPath>,
    mut evaluation_order: Vec<ir::ModelPath>,
    models: &HashMap<ir::ModelPath, ir::Model>,
) -> (Vec<ir::ModelPath>, HashSet<ir::ModelPath>) {
    for reference_import in model.get_references().values() {
        let reference_model_path = reference_import.path();
        if visited.contains(reference_model_path) {
            continue;
        }

        let reference_model = models
            .get(reference_model_path)
            .expect("reference model should exist because it was checked before");

        (evaluation_order, visited) = process_model_dependencies(
            reference_model_path,
            reference_model,
            visited,
            evaluation_order,
            models,
        );
    }

    evaluation_order.push(model_path.clone());
    visited.insert(model_path.clone());

    (evaluation_order, visited)
}
