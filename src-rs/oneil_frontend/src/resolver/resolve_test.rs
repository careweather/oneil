//! Test resolution for the Oneil model loader

use std::ops::Deref;

use oneil_ir as ir;
use oneil_shared::symbols::TestIndex;

use crate::{
    ExternalResolutionContext, ResolutionContext,
    error::{self},
    resolver::{
        resolve_expr::{get_expr_dependencies, resolve_expr},
        resolve_trace_level::resolve_trace_level,
        util::TestWithSection,
    },
};

/// Resolves tests from AST test declarations.
pub fn resolve_tests<E>(
    tests: Vec<TestWithSection<'_>>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    let tests = tests.into_iter().enumerate().map(|(test_index, decl)| {
        let test_index = TestIndex::new(test_index);
        let test_span = decl.test.span().clone();

        let trace_level = resolve_trace_level(decl.test.trace_level());

        let test_expr = resolve_expr(decl.test.expr(), resolution_context)
            .map_err(|errors| (test_index, error::convert_errors(errors)))?;

        let dependencies = get_expr_dependencies(&test_expr);

        let note = decl
            .test
            .note()
            .map(|n| ir::Note::new(n.value().to_string()));

        Ok((
            test_index,
            ir::Test::new(
                test_span,
                trace_level,
                test_expr,
                dependencies,
                decl.section_label.map(|label| label.deref().clone()),
                note,
            ),
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
        resolver::TestWithSection,
        test::{
            external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
            test_ast, test_model_path,
        },
    };

    use super::*;

    use oneil_ir as ir;

    #[test]
    fn resolve_tests_empty() {
        // build the tests
        let tests_refs: Vec<TestWithSection<'_>> = vec![];

        // build the context
        let active_path = test_model_path("main");
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
        let tests_refs: Vec<_> = tests
            .iter()
            .map(|t| TestWithSection {
                test: t,
                section_label: None,
            })
            .collect();

        // build the context
        let active_path = test_model_path("main");
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
            .get(&TestIndex::new(0))
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
        let tests_refs: Vec<_> = tests
            .iter()
            .map(|t| TestWithSection {
                test: t,
                section_label: None,
            })
            .collect();

        // build the context
        let active_path = test_model_path("main");
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
            .get(&TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test.trace_level(), ir::TraceLevel::Debug);

        // check the errors
        let test_errors = resolution_context.get_active_model_test_errors();
        assert!(test_errors.is_empty(), "expected no test errors");
    }
}
