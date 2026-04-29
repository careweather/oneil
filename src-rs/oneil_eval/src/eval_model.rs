use indexmap::IndexMap;
use oneil_frontend::{InstanceGraph, InstancedModel};
use oneil_ir as ir;
use oneil_shared::{EvalInstanceKey, partial::MaybePartialResult, symbols::TestIndex};

use oneil_output::{self as output, EvalError, ExpectedType, Model, ModelEvalErrors, Value};

use crate::{
    context::{EvalContext, ExternalEvaluationContext},
    eval_expr, eval_parameter,
};

/// Evaluates every instance in `graph`, returning per-instance results.
///
/// Use this entry point when callers supply a graph built externally.
pub fn eval_model_from_graph<E: ExternalEvaluationContext>(
    graph: &InstanceGraph,
    external_context: &mut E,
) -> IndexMap<EvalInstanceKey, MaybePartialResult<Model, ModelEvalErrors>> {
    let mut context = EvalContext::from_graph(graph, external_context);

    force_all_models(graph, &mut context);
    propagate_reference_errors(&mut context);

    context.into_result()
}

/// Collects every `(EvalInstanceKey, &InstancedModel)` pair reachable
/// from `graph`'s root subtree and from each pool entry, in pre-order.
fn collect_instances(graph: &InstanceGraph) -> Vec<(EvalInstanceKey, &InstancedModel)> {
    let mut out = Vec::new();
    let root_key = EvalInstanceKey::root(graph.root.path().clone());
    collect_subtree(graph.root.as_ref(), &root_key, &mut out);
    for (path, instance) in &graph.reference_pool {
        let pool_key = EvalInstanceKey::root(path.clone());
        collect_subtree(instance.as_ref(), &pool_key, &mut out);
    }
    out
}

fn collect_subtree<'a>(
    node: &'a InstancedModel,
    key: &EvalInstanceKey,
    out: &mut Vec<(EvalInstanceKey, &'a InstancedModel)>,
) {
    out.push((key.clone(), node));
    for (alias, sub) in node.submodels() {
        let child_key = EvalInstanceKey {
            model_path: sub.instance.path().clone(),
            instance_path: key.instance_path.clone().child(alias.clone()),
        };
        collect_subtree(sub.instance.as_ref(), &child_key, out);
    }
}

/// Drives lazy forcing of every pending parameter on every instance and evaluates tests.
fn force_all_models<E: ExternalEvaluationContext>(
    graph: &InstanceGraph,
    context: &mut EvalContext<'_, E>,
) {
    let pairs: Vec<(EvalInstanceKey, Vec<(_, _)>)> = collect_instances(graph)
        .into_iter()
        .map(|(key, instanced)| {
            let tests: Vec<_> = instanced
                .tests()
                .iter()
                .map(|(idx, test)| (*idx, test.clone()))
                .collect();
            (key, tests)
        })
        .collect();

    for (key, tests) in pairs {
        context.force_all_pending_on(&key);
        if tests.is_empty() {
            continue;
        }
        context.push_active_model(key.clone());
        for (test_index, test) in tests {
            let test_result = eval_test(test_index, &test, context);
            context.add_test_result(&key, test_index, test_result);
        }
        context.pop_active_model(&key);
    }
}

/// After forcing, each parent instance records which of its references had errors.
fn propagate_reference_errors<E: ExternalEvaluationContext>(context: &mut EvalContext<'_, E>) {
    let pairs: Vec<(EvalInstanceKey, EvalInstanceKey)> = context.reference_pairs_snapshot();
    for (parent_key, child_key) in pairs {
        if context.reference_has_errors(&child_key) {
            context.add_reference_error_to(&parent_key, &child_key);
        }
    }
}

/// Evaluates a single test in the context of the currently active scope.
fn eval_test<E: ExternalEvaluationContext>(
    test_index: TestIndex,
    test: &ir::Test,
    context: &mut EvalContext<'_, E>,
) -> Result<output::Test, Vec<EvalError>> {
    context.begin_test_evaluation(test_index);

    let (test_result, expr_span) = eval_expr::eval_expr(test.expr(), context)?;
    let warnings = context.take_expression_warnings();

    let expr_span = expr_span.clone();

    match test_result {
        Value::Boolean(true) => Ok(output::Test {
            result: output::TestResult::Passed,
            expr_span,
            warnings,
        }),
        Value::Boolean(false) => {
            let builtin_dependency_values = eval_parameter::get_builtin_dependency_values(
                test.dependencies().builtin(),
                context,
            );
            let parameter_dependency_values = eval_parameter::get_parameter_dependency_values(
                test.dependencies().parameter(),
                context,
            );
            let external_dependency_values = eval_parameter::get_external_dependency_values(
                test.dependencies().external(),
                context,
            );

            let debug_info = Box::new(output::DebugInfo {
                builtin_dependency_values,
                parameter_dependency_values,
                external_dependency_values,
            });
            Ok(output::Test {
                result: output::TestResult::Failed { debug_info },
                expr_span,
                warnings,
            })
        }
        Value::String(_) | Value::Number(_) | Value::MeasuredNumber(_) => {
            Err(vec![EvalError::InvalidType {
                expected_type: ExpectedType::Boolean,
                found_type: test_result.type_(),
                found_span: expr_span,
            }])
        }
    }
}
