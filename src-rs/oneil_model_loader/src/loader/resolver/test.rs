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
use oneil_ir::test::{Test, TestIndex};

use crate::{
    BuiltinRef,
    error::{self, TestResolutionError},
    loader::resolver::{
        ModelInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
        trace_level::resolve_trace_level,
    },
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
    tests: Vec<&ast::test::TestNode>,
    builtin_ref: &impl BuiltinRef,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> (
    HashMap<TestIndex, Test>,
    HashMap<TestIndex, Vec<TestResolutionError>>,
) {
    let tests = tests.into_iter().enumerate().map(|(test_index, test)| {
        let test_index = TestIndex::new(test_index);

        let trace_level = resolve_trace_level(test.trace_level());

        let test_expr = resolve_expr(
            &test.expr(),
            builtin_ref,
            defined_parameters_info,
            submodel_info,
            model_info,
        )
        .map_err(|errors| (test_index.clone(), error::convert_errors(errors)))?;

        Ok((test_index, Test::new(trace_level, test_expr)))
    });

    error::split_ok_and_errors(tests)
}

#[cfg(test)]
mod tests {
    use crate::{error::VariableResolutionError, util::get_span_from_ast_span};

    use super::*;

    use oneil_ir::{debug_info::TraceLevel as ModelTraceLevel, reference::Identifier};
    use std::collections::HashSet;

    // TODO: write tests that test the span of the test inputs

    mod helper {
        use crate::test::TestBuiltinRef;

        use super::*;

        /// Helper function to create empty builtin variables
        pub fn create_empty_builtin_ref() -> TestBuiltinRef {
            TestBuiltinRef::new()
        }

        /// Helper function to create test parameter information for testing
        pub fn create_empty_parameter_info() -> ParameterInfo<'static> {
            ParameterInfo::new(HashMap::new(), HashSet::new())
        }

        /// Helper function to create test submodel information for testing
        pub fn create_empty_submodel_info() -> SubmodelInfo<'static> {
            SubmodelInfo::new(HashMap::new(), HashSet::new())
        }

        /// Helper function to create test model information for testing
        pub fn create_empty_model_info() -> ModelInfo<'static> {
            ModelInfo::new(HashMap::new(), HashSet::new())
        }

        /// Helper function to create a test span
        pub fn test_ast_span(start: usize, end: usize) -> ast::Span {
            ast::Span::new(start, end - start, 0)
        }

        /// Helper function to create an identifier node
        pub fn create_identifier_node(name: &str, start: usize) -> ast::naming::IdentifierNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            ast::node::Node::new(test_ast_span(start, start + name.len()), identifier)
        }

        /// Helper function to create a literal expression node
        pub fn create_literal_expr_node(
            literal: ast::expression::Literal,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let literal_node = ast::node::Node::new(test_ast_span(start, end), literal);
            let expr = ast::expression::Expr::Literal(literal_node);
            ast::node::Node::new(test_ast_span(start, end), expr)
        }

        /// Helper function to create a simple identifier variable
        pub fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
            let identifier_node = create_identifier_node(name, 0);
            let variable = ast::expression::Variable::Identifier(identifier_node);
            ast::node::Node::new(test_ast_span(0, name.len()), variable)
        }

        /// Helper function to create a variable expression node
        pub fn create_variable_expr_node(
            variable: ast::expression::VariableNode,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr = ast::expression::Expr::Variable(variable);
            ast::node::Node::new(test_ast_span(start, end), expr)
        }

        /// Helper function to create a test node
        pub fn create_test_node(
            trace_level: Option<ast::debug_info::TraceLevel>,
            expr: ast::expression::ExprNode,
            start: usize,
            end: usize,
        ) -> ast::test::TestNode {
            let trace_level_node =
                trace_level.map(|tl| ast::node::Node::new(test_ast_span(start, start + 1), tl));

            let test = ast::test::Test::new(trace_level_node, expr, None);
            ast::node::Node::new(test_ast_span(start, end), test)
        }
    }

    #[test]
    fn test_resolve_tests_empty() {
        // create the tests
        let tests = vec![];
        let tests_refs = tests.iter().collect();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &helper::create_empty_builtin_ref(),
            &helper::create_empty_parameter_info(),
            &helper::create_empty_submodel_info(),
            &helper::create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_tests_basic() {
        // create the tests with various configurations
        let tests = vec![
            // test: true
            helper::create_test_node(
                None,
                helper::create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4),
                0,
                4,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &helper::create_empty_builtin_ref(),
            &helper::create_empty_parameter_info(),
            &helper::create_empty_submodel_info(),
            &helper::create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);

        let test_0 = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test_0.trace_level(), &ModelTraceLevel::None);
    }

    #[test]
    fn test_resolve_tests_with_debug_trace() {
        // create the tests with debug trace level
        let tests = vec![
            // ** test: true
            helper::create_test_node(
                Some(ast::debug_info::TraceLevel::Debug),
                helper::create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4),
                0,
                4,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &helper::create_empty_builtin_ref(),
            &helper::create_empty_parameter_info(),
            &helper::create_empty_submodel_info(),
            &helper::create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test.trace_level(), &ModelTraceLevel::Debug);
    }

    #[test]
    fn test_resolve_tests_with_undefined_variable() {
        // create the tests with undefined variable
        let undefined_var = helper::create_identifier_variable("undefined_var");
        let undefined_var_span = get_span_from_ast_span(undefined_var.node_span());
        let tests = vec![
            // test: undefined_var
            helper::create_test_node(
                None,
                helper::create_variable_expr_node(undefined_var, 0, 13),
                0,
                13,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &helper::create_empty_builtin_ref(),
            &helper::create_empty_parameter_info(),
            &helper::create_empty_submodel_info(),
            &helper::create_empty_model_info(),
        );

        // check the errors
        assert_eq!(errors.len(), 1);

        let test_errors = errors.get(&TestIndex::new(0)).unwrap();
        assert!(test_errors.len() == 1);
        assert_eq!(
            test_errors[0],
            TestResolutionError::variable_resolution(VariableResolutionError::undefined_parameter(
                Identifier::new("undefined_var"),
                undefined_var_span,
            )),
        );

        // check the resolved tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_tests_mixed_success_and_error() {
        // create the tests with mixed success and error cases
        let undefined_var = helper::create_identifier_variable("undefined_var");
        let undefined_var_span = get_span_from_ast_span(undefined_var.node_span());
        let tests = vec![
            // test: true
            helper::create_test_node(
                None,
                helper::create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4),
                0,
                4,
            ),
            // test: undefined_var
            helper::create_test_node(
                Some(ast::debug_info::TraceLevel::Trace),
                helper::create_variable_expr_node(
                    helper::create_identifier_variable("undefined_var"),
                    0,
                    13,
                ),
                0,
                13,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the tests
        let (resolved_tests, errors) = resolve_tests(
            tests_refs,
            &helper::create_empty_builtin_ref(),
            &helper::create_empty_parameter_info(),
            &helper::create_empty_submodel_info(),
            &helper::create_empty_model_info(),
        );

        // check the errors
        assert_eq!(errors.len(), 1);
        let test_errors = errors.get(&TestIndex::new(1)).unwrap();
        assert!(test_errors.len() == 1);
        assert_eq!(
            test_errors[0],
            TestResolutionError::variable_resolution(VariableResolutionError::undefined_parameter(
                Identifier::new("undefined_var"),
                undefined_var_span,
            )),
        );

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test.trace_level(), &ModelTraceLevel::None);
    }
}
