use indexmap::IndexMap;
use oneil_frontend::{InstanceGraph, InstancedModel};
use oneil_ir as ir;
use oneil_shared::{EvalInstanceKey, partial::MaybePartialResult};

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
        if key.instance_path.is_root() {
            context.set_evaluation_cache_root(key.model_path.clone());
        }

        context.force_all_pending_on(&key);
        if tests.is_empty() {
            continue;
        }
        context.push_active_model(key.clone());
        for (test_index, test) in tests {
            let test_result = eval_test(&test, context);
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
    test: &ir::Test,
    context: &mut EvalContext<'_, E>,
) -> Result<output::Test, Vec<EvalError>> {
    context.begin_expression_evaluation();
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

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use oneil_ir::{
        self as ir,
        test_helpers::{
            expr::{compare, fallback, imported_call, lit_bool, lit_number, param_var, unary},
            parameter::{
                builtin_dependencies_singleton, external_dependencies_singleton,
                parameter_dependencies_singleton,
            },
            test::make_test,
        },
    };
    use oneil_output::{
        self as output, EvalError, ExpectedType, Number, NumberType, TestResult, Value, ValueType,
    };
    use oneil_shared::{
        EvalInstanceKey,
        paths::PythonPath,
        span::Span,
        symbols::{BuiltinValueName, ParameterName, PyFunctionName, ReferenceName},
    };

    use crate::{
        check_is_close,
        context::EvalContext,
        test_context::{TestExternalContext, test_model_path},
        test_fixtures::{output_parameter, with_root_context},
    };

    use super::*;

    /// Evaluates `test` with a fresh context and no pre-seeded parameters.
    fn eval_test_simple(test: &ir::Test) -> Result<output::Test, Vec<EvalError>> {
        with_root_context(|context| eval_test(test, context))
    }

    #[test]
    fn eval_test_passes_on_true() {
        let test = make_test(lit_bool(true), ir::Dependencies::new());
        let result = eval_test_simple(&test).expect("eval should succeed");
        assert!(matches!(result.result, TestResult::Passed));
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn eval_test_fails_on_false_with_empty_debug_info() {
        let test = make_test(lit_bool(false), ir::Dependencies::new());
        let result = eval_test_simple(&test).expect("eval should succeed");
        let TestResult::Failed { debug_info } = result.result else {
            panic!("expected Failed, got {:?}", result.result);
        };
        assert!(debug_info.builtin_dependency_values.is_empty());
        assert!(debug_info.parameter_dependency_values.is_empty());
        assert!(debug_info.external_dependency_values.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn eval_test_fails_with_parameter_dependency_values() {
        let dependencies = parameter_dependencies_singleton("x");

        // x > 10  with x = 5 → false
        let expr = compare(
            ir::ComparisonOp::GreaterThan,
            param_var("x"),
            lit_number(10.0),
        );
        let test = make_test(expr, dependencies);

        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));
        context.add_parameter_result(
            ParameterName::from("x"),
            Ok(output_parameter("x", Value::Number(Number::Scalar(5.0)))),
        );

        let result = eval_test(&test, &mut context).expect("eval should succeed");
        let TestResult::Failed { debug_info } = result.result else {
            panic!("expected Failed, got {:?}", result.result);
        };

        assert_eq!(debug_info.parameter_dependency_values.len(), 1);
        let value = debug_info
            .parameter_dependency_values
            .get(&ParameterName::from("x"))
            .expect("x should be in debug info");
        let Value::Number(Number::Scalar(n)) = value else {
            panic!("expected scalar, got {value:?}");
        };
        check_is_close(5.0, *n).assert();
    }

    #[test]
    fn eval_test_fails_with_builtin_dependency_values() {
        let dependencies = builtin_dependencies_singleton("pi");

        let test = make_test(lit_bool(false), dependencies);
        let result = eval_test_simple(&test).expect("eval should succeed");
        let TestResult::Failed { debug_info } = result.result else {
            panic!("expected Failed, got {:?}", result.result);
        };

        assert_eq!(debug_info.builtin_dependency_values.len(), 1);
        let value = debug_info
            .builtin_dependency_values
            .get(&BuiltinValueName::from("pi"))
            .expect("pi should be in debug info");
        let Value::Number(Number::Scalar(n)) = value else {
            panic!("expected scalar, got {value:?}");
        };
        check_is_close(PI, *n).assert();
    }

    #[test]
    fn eval_test_fails_with_external_dependency_values() {
        let dependencies = external_dependencies_singleton("y", "child");

        let test = make_test(lit_bool(false), dependencies);

        let mut external = TestExternalContext::new();
        let mut context = EvalContext::new(&mut external);

        let parent = EvalInstanceKey::root(test_model_path("parent"));
        let child = EvalInstanceKey::root(test_model_path("child"));
        context.add_parameter_result_to(
            &child,
            ParameterName::from("y"),
            Ok(output_parameter("y", Value::Number(Number::Scalar(7.0)))),
        );
        context.push_active_model(parent);
        context.add_reference(ReferenceName::from("child"), child);

        let result = eval_test(&test, &mut context).expect("eval should succeed");
        let TestResult::Failed { debug_info } = result.result else {
            panic!("expected Failed, got {:?}", result.result);
        };

        assert_eq!(debug_info.external_dependency_values.len(), 1);
        let key = (ReferenceName::from("child"), ParameterName::from("y"));
        let value = debug_info
            .external_dependency_values
            .get(&key)
            .expect("y.child should be in debug info");
        let Value::Number(Number::Scalar(n)) = value else {
            panic!("expected scalar, got {value:?}");
        };
        check_is_close(7.0, *n).assert();
    }

    #[test]
    fn eval_test_rejects_non_boolean_result() {
        let test = make_test(lit_number(1.0), ir::Dependencies::new());
        let errors = eval_test_simple(&test).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::InvalidType {
                    expected_type: ExpectedType::Boolean,
                    found_type: ValueType::Number {
                        number_type: NumberType::Scalar
                    },
                    ..
                }
            ),
            "expected InvalidType Boolean vs Scalar, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_test_propagates_expression_errors() {
        // !1 is a type error
        let expr = unary(ir::UnaryOp::Not, lit_number(1.0));
        let test = make_test(expr, ir::Dependencies::new());
        let errors = eval_test_simple(&test).expect_err("eval should fail");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0],
                EvalError::InvalidType {
                    expected_type: ExpectedType::Boolean,
                    found_type: ValueType::Number {
                        number_type: NumberType::Scalar
                    },
                    ..
                }
            ),
            "expected InvalidType Boolean vs Scalar, got {:?}",
            errors[0]
        );
    }

    #[test]
    fn eval_test_collects_fallback_warnings_on_pass() {
        let mut external = TestExternalContext::new();
        external.register_imported_function(
            PythonPath::from_str_no_ext("helpers"),
            PyFunctionName::from("fail"),
            |_args| {
                Err(Box::new(EvalError::PythonEvalError {
                    function_name: PyFunctionName::from("fail"),
                    function_call_span: Span::synthetic(),
                    message: "boom".to_string(),
                    traceback: None,
                }))
            },
        );

        let mut context = EvalContext::new(&mut external);
        context.set_evaluation_cache_root(test_model_path("test"));
        context.push_active_model(EvalInstanceKey::root(test_model_path("test")));

        // (fail() ? true)  → passes, with a UsedFallback warning
        let left = imported_call("helpers", "fail", vec![]);
        let expr = fallback(left, lit_bool(true));
        let test = make_test(expr, ir::Dependencies::new());

        let result = eval_test(&test, &mut context).expect("eval should succeed");
        assert!(matches!(result.result, TestResult::Passed));
        assert_eq!(result.warnings.len(), 1);
        assert!(
            matches!(
                &result.warnings[0],
                output::EvalWarning::UsedFallback {
                    function_name,
                    message,
                    ..
                } if function_name.as_str() == "fail" && message == "boom"
            ),
            "expected UsedFallback warning, got {:?}",
            result.warnings[0]
        );
    }
}
