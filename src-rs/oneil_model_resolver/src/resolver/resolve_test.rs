//! Test resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;

use crate::{
    ExternalResolutionContext, ResolutionContext,
    error::{self},
    resolver::{
        resolve_expr::{get_expr_dependencies, resolve_expr},
        resolve_trace_level::resolve_trace_level,
    },
};

/// Resolves tests from AST test declarations.
pub fn resolve_tests<E>(
    tests: Vec<&ast::TestNode>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    let tests = tests.into_iter().enumerate().map(|(test_index, test)| {
        let test_index = ir::TestIndex::new(test_index);
        let test_span = test.span();

        let trace_level = resolve_trace_level(test.trace_level());

        let test_expr = resolve_expr(test.expr(), resolution_context)
            .map_err(|errors| (test_index, error::convert_errors(errors)))?;

        let dependencies = get_expr_dependencies(&test_expr);

        Ok((
            test_index,
            ir::Test::new(test_span, trace_level, test_expr, dependencies),
        ))
    });

    let (resolved_tests, errors): (Vec<_>, _) = error::split_ok_and_errors(tests);

    for (test_index, resolved_test) in resolved_tests {
        resolution_context.add_test_to_active_model(test_index, resolved_test);
    }

    for (test_index, errors) in errors {
        for error in errors {
            resolution_context.add_test_error_to_active_model(test_index, error);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::VariableResolutionError,
        test::{
            external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
            test_ast,
        },
    };

    use super::*;

    use oneil_ir as ir;

    #[test]
    fn resolve_tests_empty() {
        // build the tests
        let tests: [ast::TestNode; 0] = [];
        let tests_refs: Vec<&ast::TestNode> = tests.iter().collect();

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the test resolution
        resolve_tests(tests_refs, &mut resolution_context);

        // check the errors
        let test_errors = resolution_context.get_active_model_test_errors();
        assert!(
            test_errors.is_empty(),
            "expected no test errors, got {test_errors:?}"
        );

        // check the tests
        assert!(resolution_context.get_active_model_tests().is_empty());

        // check the errors
        assert!(resolution_context.get_active_model_test_errors().is_empty());
    }

    #[test]
    fn resolve_tests_basic() {
        // build the tests
        let tests = [
            // > test: true
            test_ast::TestNodeBuilder::new()
                .with_boolean_expr(true)
                .build(),
        ];
        let tests_refs: Vec<&ast::TestNode> = tests.iter().collect();

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the test resolution
        resolve_tests(tests_refs, &mut resolution_context);

        // check the resolved tests
        let resolved_tests = resolution_context.get_active_model_tests();
        assert_eq!(resolved_tests.len(), 1);

        let test_0 = resolved_tests
            .get(&ir::TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test_0.trace_level(), ir::TraceLevel::None);

        // check the errors
        let test_errors = resolution_context.get_active_model_test_errors();
        assert!(
            test_errors.is_empty(),
            "expected no test errors, got {test_errors:?}"
        );
    }

    #[test]
    fn resolve_tests_with_debug_trace() {
        // build the tests with debug trace level
        let tests = [
            // > ** test: true
            test_ast::TestNodeBuilder::new()
                .with_boolean_expr(true)
                .with_debug_trace_level()
                .build(),
        ];
        let tests_refs: Vec<&ast::TestNode> = tests.iter().collect();

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the test resolution
        resolve_tests(tests_refs, &mut resolution_context);

        // check the resolved tests
        let resolved_tests = resolution_context.get_active_model_tests();
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests
            .get(&ir::TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test.trace_level(), ir::TraceLevel::Debug);

        // check the errors
        let test_errors = resolution_context.get_active_model_test_errors();
        assert!(test_errors.is_empty(), "expected no test errors");
    }

    #[test]
    fn resolve_tests_with_undefined_variable() {
        // build the tests with undefined variable
        let tests = [
            // > test: undefined_var
            test_ast::TestNodeBuilder::new()
                .with_variable_expr("undefined_var")
                .build(),
        ];
        let tests_refs: Vec<&ast::TestNode> = tests.iter().collect();

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the test resolution
        resolve_tests(tests_refs, &mut resolution_context);

        // check the resolved tests
        assert!(resolution_context.get_active_model_tests().is_empty());

        // check the errors
        let test_errors = resolution_context.get_active_model_test_errors();
        assert_eq!(test_errors.len(), 1);

        let errors_for_test_0 = test_errors
            .get(&ir::TestIndex::new(0))
            .expect("test 0 errors should exist");
        assert_eq!(errors_for_test_0.len(), 1);

        let error = &errors_for_test_0[0];
        let VariableResolutionError::UndefinedParameter {
            model_path,
            parameter_name,
            reference_span: _,
        } = error
        else {
            panic!("expected undefined parameter error, got {error:?}");
        };

        assert_eq!(model_path, &None);
        assert_eq!(
            parameter_name,
            &ir::ParameterName::new("undefined_var".to_string())
        );
    }

    #[test]
    fn resolve_tests_mixed_success_and_error() {
        // build the tests with mixed success and error cases
        let tests = [
            // > test: true
            test_ast::TestNodeBuilder::new()
                .with_boolean_expr(true)
                .build(),
            // > test: undefined_var
            test_ast::TestNodeBuilder::new()
                .with_variable_expr("undefined_var")
                .build(),
        ];
        let tests_refs: Vec<&ast::TestNode> = tests.iter().collect();

        // build the context
        let active_path = ir::ModelPath::new("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the test resolution
        resolve_tests(tests_refs, &mut resolution_context);

        // check the resolved tests
        let resolved_tests = resolution_context.get_active_model_tests();
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests
            .get(&ir::TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test.trace_level(), ir::TraceLevel::None);

        // check the errors
        let test_errors = resolution_context.get_active_model_test_errors();
        assert_eq!(test_errors.len(), 1);

        let errors_for_test_1 = test_errors
            .get(&ir::TestIndex::new(1))
            .expect("test 1 errors should exist");
        assert_eq!(errors_for_test_1.len(), 1);

        let error = &errors_for_test_1[0];
        let VariableResolutionError::UndefinedParameter {
            model_path,
            parameter_name,
            reference_span: _,
        } = error
        else {
            panic!("expected undefined parameter error, got {error:?}");
        };

        assert_eq!(model_path, &None);
        assert_eq!(
            parameter_name,
            &ir::ParameterName::new("undefined_var".to_string())
        );
    }
}
