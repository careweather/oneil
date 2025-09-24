//! Test resolution for the Oneil model loader
//!
//! This module provides functionality for resolving tests in Oneil models.
//! Test resolution involves processing test declarations to create executable
//! test structures.
//!
//! # Overview
//!
//! Tests in Oneil allow models to define validation logic and test scenarios.
//! This module handles two types of tests:
//!
//! ## Tests
//! Tests are defined using the `test` declaration syntax:
//! ```oneil
//! test: x > 0
//! test {x, y}: x + y == 10
//! test {param}: param > 100
//! ```
//!
//! # Resolution Process
//!
//! The resolution process involves:
//! 1. **Trace Level Resolution**: Converting trace level indicators to `TraceLevel` enum
//! 2. **Input Processing**: Converting input identifiers to `Identifier` types
//! 3. **Expression Resolution**: Resolving test expressions with proper variable scope
//! 4. **Error Collection**: Gathering and categorizing resolution errors
//!
//! # Error Handling
//!
//! The model provides comprehensive error handling for various failure scenarios:
//! - **Variable Resolution Errors**: When test expressions reference undefined variables
//!
//! All errors are collected and returned rather than causing the function to fail,
//! allowing for partial success scenarios.

use std::collections::HashMap;

use oneil_ast as ast;
use oneil_ir as ir;

use crate::{
    BuiltinRef,
    error::{self, TestResolutionError},
    resolver::{resolve_expr::resolve_expr, resolve_trace_level::resolve_trace_level},
    util::context::{ParameterContext, ReferenceContext},
};

/// Resolves tests from AST test declarations.
///
/// This function processes a collection of `ast::Test` declarations and resolves
/// them into executable `Test` structures with proper variable scoping and
/// error handling.
///
/// # Arguments
///
/// * `tests` - A vector of AST test declarations to resolve
/// * `defined_parameters_info` - Information about available parameters in the model
/// * `submodel_info` - Information about available submodels in the model
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// A tuple containing:
/// * `HashMap<TestIndex, Test>` - Successfully resolved tests mapped to their indices
/// * `HashMap<TestIndex, Vec<TestResolutionError>>` - Any resolution errors that occurred
///
/// # Error Handling
///
/// All errors are collected and returned rather than causing the function to fail.
/// Each test is processed independently, so errors in one test don't affect others.
pub fn resolve_tests(
    tests: Vec<&ast::TestNode>,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> (
    HashMap<ir::TestIndex, ir::Test>,
    HashMap<ir::TestIndex, Vec<TestResolutionError>>,
) {
    let tests = tests.into_iter().enumerate().map(|(test_index, test)| {
        let test_index = ir::TestIndex::new(test_index);

        let trace_level = resolve_trace_level(test.trace_level());

        let test_expr = resolve_expr(
            test.expr(),
            builtin_ref,
            reference_context,
            parameter_context,
        )
        .map_err(|errors| (test_index, error::convert_errors(errors)))?;

        Ok((test_index, ir::Test::new(trace_level, test_expr)))
    });

    error::split_ok_and_errors(tests)
}

#[cfg(test)]
mod tests {
    use crate::{
        error::VariableResolutionError,
        test::{
            TestBuiltinRef,
            construct::{ParameterContextBuilder, ReferenceContextBuilder, test_ast},
        },
    };

    use super::*;

    use oneil_ir as ir;

    #[test]
    fn resolve_tests_empty() {
        // create the tests
        let tests = [];
        let tests_refs = tests.iter().collect();

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the errors
        assert!(errors.is_empty());

        // check the tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn resolve_tests_basic() {
        // create the tests with various configurations
        let tests = [
            // > test: true
            test_ast::TestNodeBuilder::new()
                .with_boolean_expr(true)
                .build(),
        ];
        let tests_refs = tests.iter().collect();

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);

        let test_0 = resolved_tests
            .get(&ir::TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test_0.trace_level(), ir::TraceLevel::None);
    }

    #[test]
    fn resolve_tests_with_debug_trace() {
        // create the tests with debug trace level
        let tests = [
            // > ** test: true
            test_ast::TestNodeBuilder::new()
                .with_boolean_expr(true)
                .with_debug_trace_level()
                .build(),
        ];
        let tests_refs = tests.iter().collect();

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests
            .get(&ir::TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test.trace_level(), ir::TraceLevel::Debug);
    }

    #[test]
    fn resolve_tests_with_undefined_variable() {
        // create the tests with undefined variable
        let tests = [
            // > test: undefined_var
            test_ast::TestNodeBuilder::new()
                .with_variable_expr("undefined_var")
                .build(),
        ];
        let tests_refs = tests.iter().collect();

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the errors
        assert_eq!(errors.len(), 1);

        let test_errors = errors
            .get(&ir::TestIndex::new(0))
            .expect("test errors should exist");

        assert!(test_errors.len() == 1);

        let error = &test_errors[0];

        let TestResolutionError::VariableResolution(error) = error else {
            panic!("expected variable resolution error, got {error:?}");
        };

        let VariableResolutionError::UndefinedParameter {
            model_path,
            parameter,
            reference_span: _,
        } = error
        else {
            panic!("expected undefined parameter error, got {error:?}");
        };

        assert_eq!(model_path, &None);
        assert_eq!(parameter, &ir::Identifier::new("undefined_var"));

        // check the resolved tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn resolve_tests_mixed_success_and_error() {
        // create the tests with mixed success and error cases
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
        let tests_refs = tests.iter().collect();

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the errors
        assert_eq!(errors.len(), 1);
        let test_errors = errors
            .get(&ir::TestIndex::new(1))
            .expect("test errors should exist");

        assert!(test_errors.len() == 1);

        let error = &test_errors[0];

        let TestResolutionError::VariableResolution(error) = error else {
            panic!("expected variable resolution error, got {error:?}");
        };

        let VariableResolutionError::UndefinedParameter {
            model_path,
            parameter,
            reference_span: _,
        } = error
        else {
            panic!("expected undefined parameter error, got {error:?}");
        };

        assert_eq!(model_path, &None);
        assert_eq!(parameter, &ir::Identifier::new("undefined_var"));

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests
            .get(&ir::TestIndex::new(0))
            .expect("test should exist");
        assert_eq!(test.trace_level(), ir::TraceLevel::None);
    }
}
