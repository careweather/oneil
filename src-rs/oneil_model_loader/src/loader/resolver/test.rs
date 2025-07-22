//! Model test resolution for the Oneil model loader
//!
//! This module provides functionality for resolving model tests and submodel tests
//! in Oneil models. Test resolution involves processing test declarations and
//! submodel test inputs to create executable test structures.
//!
//! # Overview
//!
//! Tests in Oneil allow models to define validation logic and test scenarios.
//! This module handles two types of tests:
//!
//! ## Model Tests
//! Model tests are defined using the `test` declaration syntax:
//! ```oneil
//! test: x > 0
//! test {x, y}: x + y == 10
//! test {param}: param > 100
//! ```
//!
//! ## Submodel Tests
//! Submodel tests are created from `use` declarations with inputs:
//! ```oneil
//! use sensor_model(location="north", height=100) as sensor
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
//! - **Parameter Resolution Errors**: When test inputs reference undefined parameters
//! - **Submodel Resolution Errors**: When test expressions reference undefined submodels
//!
//! All errors are collected and returned rather than causing the function to fail,
//! allowing for partial success scenarios.

use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_ir::{
    reference::Identifier,
    test::{ModelTest, SubmodelTest, SubmodelTestInputs, TestIndex},
};

use crate::{
    error::{self, ModelTestResolutionError, SubmodelTestInputResolutionError},
    loader::resolver::{
        ModelInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
        trace_level::resolve_trace_level,
    },
};

/// Resolves model tests from AST test declarations.
///
/// This function processes a collection of `ast::Test` declarations and resolves
/// them into executable `ModelTest` structures with proper variable scoping and
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
/// * `HashMap<TestIndex, ModelTest>` - Successfully resolved model tests mapped to their indices
/// * `HashMap<TestIndex, Vec<ModelTestResolutionError>>` - Any resolution errors that occurred
///
/// # Error Handling
///
/// All errors are collected and returned rather than causing the function to fail.
/// Each test is processed independently, so errors in one test don't affect others.
pub fn resolve_model_tests(
    tests: Vec<&ast::test::TestNode>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> (
    HashMap<TestIndex, ModelTest>,
    HashMap<TestIndex, Vec<ModelTestResolutionError>>,
) {
    let tests = tests.into_iter().enumerate().map(|(test_index, test)| {
        let test_index = TestIndex::new(test_index);

        let trace_level = resolve_trace_level(test.trace_level());

        // TODO: verify that there are no duplicate inputs
        let inputs = test
            .inputs()
            .map(|inputs| {
                inputs
                    .iter()
                    .map(|input| Identifier::new(input.as_str()))
                    .collect()
            })
            .unwrap_or_default();

        let local_variables = &inputs;

        let test_expr = resolve_expr(
            &test.expr(),
            local_variables,
            defined_parameters_info,
            submodel_info,
            model_info,
        )
        .map_err(|errors| (test_index.clone(), error::convert_errors(errors)))?;

        Ok((test_index, ModelTest::new(trace_level, inputs, test_expr)))
    });

    error::split_ok_and_errors(tests)
}

/// Resolves submodel tests from submodel test input declarations.
///
/// This function processes a collection of submodel test inputs and resolves
/// them into executable `SubmodelTest` structures. These tests are typically
/// created from `use` declarations that include input parameters.
///
/// # Arguments
///
/// * `submodel_tests` - A vector of submodel test inputs, each containing a submodel
///   identifier and a list of model input declarations
/// * `defined_parameters_info` - Information about available parameters in the model
/// * `submodel_info` - Information about available submodels in the model
/// * `model_info` - Information about all available models and their loading status
///
/// # Returns
///
/// A tuple containing:
/// * `Vec<SubmodelTest>` - Successfully resolved submodel tests
/// * `HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>` - Any resolution errors that occurred
///
/// # Error Handling
///
/// All errors are collected and returned rather than causing the function to fail.
/// Each submodel test is processed independently, so errors in one test don't affect others.
pub fn resolve_submodel_tests(
    submodel_tests: Vec<(Identifier, Option<&ast::declaration::ModelInputListNode>)>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> (
    Vec<SubmodelTest>,
    HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
) {
    let submodel_tests = submodel_tests.into_iter().map(|(submodel_name, inputs)| {
        // TODO: verify that there are no duplicate inputs
        let inputs: Vec<_> = inputs
            .map(|inputs| {
                inputs
                    .inputs()
                    .iter()
                    .map(|input| {
                        let identifier = Identifier::new(input.ident().as_str());
                        let value = resolve_expr(
                            &input.value(),
                            &HashSet::new(),
                            defined_parameters_info,
                            submodel_info,
                            model_info,
                        )?;

                        Ok((identifier, value))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let inputs = error::combine_error_list(inputs)
            .map_err(|errors| (submodel_name.clone(), error::convert_errors(errors)))?;
        let inputs = HashMap::from_iter(inputs);
        let inputs = SubmodelTestInputs::new(inputs);

        Ok(SubmodelTest::new(submodel_name, inputs))
    });

    error::split_ok_and_errors(submodel_tests)
}

#[cfg(test)]
mod tests {
    use crate::error::VariableResolutionError;

    use super::*;

    use oneil_ir::debug_info::TraceLevel as ModelTraceLevel;
    use std::collections::HashSet;

    /// Creates test parameter information for testing
    fn create_empty_parameter_info() -> ParameterInfo<'static> {
        ParameterInfo::new(HashMap::new(), HashSet::new())
    }

    /// Creates test submodel information for testing
    fn create_empty_submodel_info() -> SubmodelInfo<'static> {
        SubmodelInfo::new(HashMap::new(), HashSet::new())
    }

    /// Creates test model information for testing
    fn create_empty_model_info() -> ModelInfo<'static> {
        ModelInfo::new(HashMap::new(), HashSet::new())
    }

    /// Helper function to create a test span
    fn test_span(start: usize, end: usize) -> ast::Span {
        ast::Span::new(start, end - start, 0)
    }

    /// Helper function to create an identifier node
    fn create_identifier_node(name: &str, start: usize) -> ast::naming::IdentifierNode {
        let identifier = ast::naming::Identifier::new(name.to_string());
        ast::node::Node::new(test_span(start, start + name.len()), identifier)
    }

    /// Helper function to create a literal expression node
    fn create_literal_expr_node(
        literal: ast::expression::Literal,
        start: usize,
        end: usize,
    ) -> ast::expression::ExprNode {
        let literal_node = ast::node::Node::new(test_span(start, end), literal);
        let expr = ast::expression::Expr::Literal(literal_node);
        ast::node::Node::new(test_span(start, end), expr)
    }

    /// Helper function to create a simple identifier variable
    fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
        let identifier_node = create_identifier_node(name, 0);
        let variable = ast::expression::Variable::Identifier(identifier_node);
        ast::node::Node::new(test_span(0, name.len()), variable)
    }

    /// Helper function to create a variable expression node
    fn create_variable_expr_node(
        variable: ast::expression::VariableNode,
        start: usize,
        end: usize,
    ) -> ast::expression::ExprNode {
        let expr = ast::expression::Expr::Variable(variable);
        ast::node::Node::new(test_span(start, end), expr)
    }

    /// Helper function to create a binary operation expression node
    fn create_binary_op_expr_node(
        left: ast::expression::ExprNode,
        op: ast::expression::BinaryOp,
        right: ast::expression::ExprNode,
        start: usize,
        end: usize,
    ) -> ast::expression::ExprNode {
        let op_node = ast::node::Node::new(test_span(start, end), op);
        let expr = ast::expression::Expr::BinaryOp {
            left: Box::new(left),
            op: op_node,
            right: Box::new(right),
        };
        ast::node::Node::new(test_span(start, end), expr)
    }

    /// Helper function to create a test inputs node
    fn create_test_inputs_node(
        inputs: Vec<&str>,
        start: usize,
        end: usize,
    ) -> ast::test::TestInputsNode {
        let input_nodes = inputs
            .iter()
            .enumerate()
            .map(|(i, name)| create_identifier_node(name, start + i * (name.len() + 2)))
            .collect();
        let test_inputs = ast::test::TestInputs::new(input_nodes);
        ast::node::Node::new(test_span(start, end), test_inputs)
    }

    /// Helper function to create a test node
    fn create_test_node(
        trace_level: Option<ast::debug_info::TraceLevel>,
        inputs: Option<Vec<&str>>,
        expr: ast::expression::ExprNode,
        start: usize,
        end: usize,
    ) -> ast::test::TestNode {
        let trace_level_node =
            trace_level.map(|tl| ast::node::Node::new(test_span(start, start + 1), tl));

        let inputs_node = inputs.map(|input_list| create_test_inputs_node(input_list, start, end));

        let test = ast::test::Test::new(trace_level_node, inputs_node, expr);
        ast::node::Node::new(test_span(start, end), test)
    }

    /// Helper function to create a model input node
    fn create_model_input_node(
        ident: &str,
        value: ast::expression::ExprNode,
        start: usize,
        end: usize,
    ) -> ast::declaration::ModelInputNode {
        let ident_node = create_identifier_node(ident, start);
        let model_input = ast::declaration::ModelInput::new(ident_node, value);
        ast::node::Node::new(test_span(start, end), model_input)
    }

    /// Helper function to create a model input list node
    fn create_model_input_list_node(
        inputs: Vec<ast::declaration::ModelInputNode>,
        start: usize,
        end: usize,
    ) -> ast::declaration::ModelInputListNode {
        let input_list = ast::declaration::ModelInputList::new(inputs);
        ast::node::Node::new(test_span(start, end), input_list)
    }

    #[test]
    fn test_resolve_model_tests_empty() {
        // create the model tests
        let tests = vec![];
        let tests_refs = tests.iter().collect();

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests_refs,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_model_tests_basic() {
        // create the model tests
        let tests = vec![
            // test: true
            create_test_node(
                None,
                None,
                create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4),
                0,
                4,
            ),
            // test {x, y}: x > 0
            create_test_node(
                None,
                Some(vec!["x", "y"]),
                create_binary_op_expr_node(
                    create_variable_expr_node(create_identifier_variable("x"), 0, 1),
                    ast::expression::BinaryOp::GreaterThan,
                    create_literal_expr_node(ast::expression::Literal::Number(0.0), 4, 5),
                    0,
                    5,
                ),
                0,
                5,
            ),
            // * test {param}: param == 42
            create_test_node(
                Some(ast::debug_info::TraceLevel::Trace),
                Some(vec!["param"]),
                create_binary_op_expr_node(
                    create_variable_expr_node(create_identifier_variable("param"), 0, 5),
                    ast::expression::BinaryOp::Eq,
                    create_literal_expr_node(ast::expression::Literal::Number(42.0), 9, 11),
                    0,
                    11,
                ),
                0,
                11,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests_refs,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 3);

        let test_0 = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test_0.trace_level(), &ModelTraceLevel::None);
        assert_eq!(test_0.inputs().len(), 0);

        let test_1 = resolved_tests.get(&TestIndex::new(1)).unwrap();
        assert_eq!(test_1.trace_level(), &ModelTraceLevel::None);
        assert_eq!(test_1.inputs().len(), 2);
        assert!(test_1.inputs().contains(&Identifier::new("x")));
        assert!(test_1.inputs().contains(&Identifier::new("y")));

        let test_2 = resolved_tests.get(&TestIndex::new(2)).unwrap();
        assert_eq!(test_2.trace_level(), &ModelTraceLevel::Trace);
        assert_eq!(test_2.inputs().len(), 1);
        assert!(test_2.inputs().contains(&Identifier::new("param")));
    }

    #[test]
    fn test_resolve_model_tests_with_debug_trace() {
        // create the model tests
        let tests = vec![
            // ** test {x}: true
            create_test_node(
                Some(ast::debug_info::TraceLevel::Debug),
                Some(vec!["x"]),
                create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4),
                0,
                4,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests_refs,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test.trace_level(), &ModelTraceLevel::Debug);
        assert_eq!(test.inputs().len(), 1);
        assert!(test.inputs().contains(&Identifier::new("x")));
    }

    #[test]
    fn test_resolve_model_tests_with_undefined_variable() {
        // create the model tests
        let tests = vec![
            // test {x}: undefined_var
            create_test_node(
                None,
                Some(vec!["x"]),
                create_variable_expr_node(create_identifier_variable("undefined_var"), 0, 13),
                0,
                13,
            ),
        ];
        let tests_refs = tests.iter().collect();

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests_refs,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert_eq!(errors.len(), 1);

        let test_errors = errors.get(&TestIndex::new(0)).unwrap();
        assert!(test_errors.len() == 1);
        assert_eq!(
            test_errors[0],
            ModelTestResolutionError::new(VariableResolutionError::undefined_parameter(
                Identifier::new("undefined_var"),
            )),
        );

        // check the resolved tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_model_tests_mixed_success_and_error() {
        let tests = vec![
            // test {x}: true
            create_test_node(
                None,
                Some(vec!["x"]),
                create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4),
                0,
                4,
            ),
            // test {y}: undefined_var
            create_test_node(
                Some(ast::debug_info::TraceLevel::Trace),
                Some(vec!["y"]),
                create_variable_expr_node(create_identifier_variable("undefined_var"), 0, 13),
                0,
                13,
            ),
        ];
        let tests_refs = tests.iter().collect();

        let (resolved_tests, errors) = resolve_model_tests(
            tests_refs,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert_eq!(errors.len(), 1);
        let test_errors = errors.get(&TestIndex::new(1)).unwrap();
        assert!(test_errors.len() == 1);
        assert_eq!(
            test_errors[0],
            ModelTestResolutionError::new(VariableResolutionError::undefined_parameter(
                Identifier::new("undefined_var"),
            )),
        );

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test.trace_level(), &ModelTraceLevel::None);
        assert_eq!(test.inputs().len(), 1);
        assert!(test.inputs().contains(&Identifier::new("x")));
    }

    #[test]
    fn test_resolve_submodel_tests_empty() {
        // create the submodel tests
        let submodel_tests = vec![];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_submodel_tests_basic() {
        // create the submodel tests
        let input_list = create_model_input_list_node(
            vec![
                create_model_input_node(
                    "location",
                    create_literal_expr_node(
                        ast::expression::Literal::String("north".to_string()),
                        0,
                        7,
                    ),
                    0,
                    7,
                ),
                create_model_input_node(
                    "height",
                    create_literal_expr_node(ast::expression::Literal::Number(100.0), 0, 6),
                    0,
                    6,
                ),
            ],
            0,
            20,
        );
        let submodel_tests = vec![
            // use my_sensor(location = "north", height = 100) as sensor
            (Identifier::new("sensor"), Some(&input_list)),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert!(resolved_tests.len() == 1);
        let test = &resolved_tests[0];
        assert_eq!(test.submodel_name(), &Identifier::new("sensor"));
        assert_eq!(test.inputs().len(), 2);
        assert!(test.inputs().contains_key(&Identifier::new("location")));
        assert!(test.inputs().contains_key(&Identifier::new("height")));
    }

    #[test]
    fn test_resolve_submodel_tests_multiple() {
        // create the submodel tests
        let input_list1 = create_model_input_list_node(
            vec![create_model_input_node(
                "param1",
                create_literal_expr_node(ast::expression::Literal::Number(10.0), 0, 4),
                0,
                4,
            )],
            0,
            10,
        );
        let input_list2 = create_model_input_list_node(
            vec![create_model_input_node(
                "param2",
                create_literal_expr_node(
                    ast::expression::Literal::String("value".to_string()),
                    0,
                    7,
                ),
                0,
                7,
            )],
            0,
            10,
        );
        let submodel_tests = vec![
            // use my_sensor1(param1 = 10) as sensor1
            (Identifier::new("sensor1"), Some(&input_list1)),
            // use my_sensor2(param2 = "value") as sensor2
            (Identifier::new("sensor2"), Some(&input_list2)),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 2);

        let test_0 = &resolved_tests[0];
        assert_eq!(test_0.submodel_name(), &Identifier::new("sensor1"));
        assert_eq!(test_0.inputs().len(), 1);
        assert!(test_0.inputs().contains_key(&Identifier::new("param1")));

        let test_1 = &resolved_tests[1];
        assert_eq!(test_1.submodel_name(), &Identifier::new("sensor2"));
        assert_eq!(test_1.inputs().len(), 1);
        assert!(test_1.inputs().contains_key(&Identifier::new("param2")));
    }

    #[test]
    fn test_resolve_submodel_tests_with_undefined_variable() {
        // create the submodel tests
        let input_list = create_model_input_list_node(
            vec![create_model_input_node(
                "param",
                create_variable_expr_node(create_identifier_variable("undefined_var"), 0, 13),
                0,
                13,
            )],
            0,
            20,
        );
        let submodel_tests = vec![
            // use my_sensor(param = undefined_var) as sensor
            (Identifier::new("sensor"), Some(&input_list)),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert_eq!(errors.len(), 1);

        let test_errors = errors.get(&Identifier::new("sensor")).unwrap();
        assert!(test_errors.len() == 1);
        assert_eq!(
            test_errors[0],
            SubmodelTestInputResolutionError::VariableResolution(
                VariableResolutionError::undefined_parameter(Identifier::new("undefined_var"),)
            ),
        );

        // check the resolved tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_submodel_tests_mixed_success_and_error() {
        // create the submodel tests
        let input_list1 = create_model_input_list_node(
            vec![create_model_input_node(
                "param1",
                create_literal_expr_node(ast::expression::Literal::Number(10.0), 0, 4),
                0,
                4,
            )],
            0,
            10,
        );
        let input_list2 = create_model_input_list_node(
            vec![create_model_input_node(
                "param2",
                create_variable_expr_node(create_identifier_variable("undefined_var"), 0, 13),
                0,
                13,
            )],
            0,
            20,
        );
        let submodel_tests = vec![
            // use my_sensor1(param1 = 10) as sensor1
            (Identifier::new("sensor1"), Some(&input_list1)),
            // use my_sensor2(param2 = undefined_var) as sensor2
            (Identifier::new("sensor2"), Some(&input_list2)),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert_eq!(errors.len(), 1);
        let test_errors = errors.get(&Identifier::new("sensor2")).unwrap();
        assert!(test_errors.len() == 1);
        assert_eq!(
            test_errors[0],
            SubmodelTestInputResolutionError::VariableResolution(
                VariableResolutionError::undefined_parameter(Identifier::new("undefined_var"),)
            ),
        );

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = &resolved_tests[0];
        assert_eq!(test.submodel_name(), &Identifier::new("sensor1"));
        assert_eq!(test.inputs().len(), 1);
        assert!(test.inputs().contains_key(&Identifier::new("param1")));
    }

    #[test]
    fn test_resolve_submodel_tests_with_complex_expressions() {
        // create the submodel tests
        let input_list = create_model_input_list_node(
            vec![
                create_model_input_node(
                    "calculation",
                    create_binary_op_expr_node(
                        create_literal_expr_node(ast::expression::Literal::Number(5.0), 0, 1),
                        ast::expression::BinaryOp::Add,
                        create_literal_expr_node(ast::expression::Literal::Number(3.0), 4, 5),
                        0,
                        5,
                    ),
                    0,
                    5,
                ),
                create_model_input_node(
                    "is_valid",
                    create_binary_op_expr_node(
                        create_literal_expr_node(ast::expression::Literal::Number(10.0), 0, 2),
                        ast::expression::BinaryOp::GreaterThan,
                        create_literal_expr_node(ast::expression::Literal::Number(5.0), 6, 7),
                        0,
                        7,
                    ),
                    0,
                    7,
                ),
            ],
            0,
            20,
        );
        let submodel_tests = vec![
            // use my_sensor(calculation = 5 + 3, is_valid = 10 > 5) as sensor
            (Identifier::new("sensor"), Some(&input_list)),
        ];

        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);

        let test = &resolved_tests[0];
        assert_eq!(test.submodel_name(), &Identifier::new("sensor"));
        assert_eq!(test.inputs().len(), 2);
        assert!(test.inputs().contains_key(&Identifier::new("calculation")));
        assert!(test.inputs().contains_key(&Identifier::new("is_valid")));
    }

    #[test]
    fn test_resolve_submodel_tests_with_parameter_reference() {
        // create the submodel tests
        let input_list = create_model_input_list_node(
            vec![create_model_input_node(
                "param",
                create_variable_expr_node(create_identifier_variable("test_param"), 0, 9),
                0,
                9,
            )],
            0,
            15,
        );
        let submodel_tests = vec![(Identifier::new("sensor"), Some(&input_list))];

        // create the parameter info
        let test_param_id = Identifier::new("test_param");
        let test_param = oneil_ir::parameter::Parameter::new(
            HashSet::new(),
            test_param_id.clone(),
            oneil_ir::parameter::ParameterValue::Simple(
                oneil_ir::expr::Expr::literal(oneil_ir::expr::Literal::number(10.0)),
                None,
            ),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );
        let parameter_info = ParameterInfo::new(
            HashMap::from([(&test_param_id, &test_param)]),
            HashSet::new(),
        );

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &parameter_info,
            &create_empty_submodel_info(),
            &create_empty_model_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = &resolved_tests[0];
        assert_eq!(test.submodel_name(), &Identifier::new("sensor"));
        assert_eq!(test.inputs().len(), 1);
        assert!(test.inputs().contains_key(&Identifier::new("param")));
    }
}
