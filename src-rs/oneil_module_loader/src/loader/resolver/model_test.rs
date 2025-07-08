//! Model test resolution for the Oneil module loader
//!
//! This module provides functionality for resolving model tests and submodel tests
//! in Oneil modules. Test resolution involves processing test declarations and
//! submodel test inputs to create executable test structures.
//!
//! # Overview
//!
//! Tests in Oneil allow modules to define validation logic and test scenarios.
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
//! The module provides comprehensive error handling for various failure scenarios:
//! - **Variable Resolution Errors**: When test expressions reference undefined variables
//! - **Parameter Resolution Errors**: When test inputs reference undefined parameters
//! - **Submodel Resolution Errors**: When test expressions reference undefined submodels
//!
//! All errors are collected and returned rather than causing the function to fail,
//! allowing for partial success scenarios.

use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_module::{
    reference::Identifier,
    test::{ModelTest, SubmodelTest, SubmodelTestInputs, TestIndex},
};

use crate::{
    error::{self, ModelTestResolutionError, SubmodelTestInputResolutionError},
    loader::resolver::{
        ModuleInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
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
/// * `defined_parameters_info` - Information about available parameters in the module
/// * `submodel_info` - Information about available submodels in the module
/// * `module_info` - Information about all available modules
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
    tests: Vec<ast::Test>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> (
    HashMap<TestIndex, ModelTest>,
    HashMap<TestIndex, Vec<ModelTestResolutionError>>,
) {
    let tests = tests.into_iter().enumerate().map(|(test_index, test)| {
        let test_index = TestIndex::new(test_index);

        let trace_level = resolve_trace_level(&test.trace_level);

        // TODO: verify that there are no duplicate inputs
        let inputs = test
            .inputs
            .into_iter()
            .map(|input| Identifier::new(input))
            .collect();

        let local_variables = &inputs;

        let test_expr = resolve_expr(
            &test.expr,
            local_variables,
            defined_parameters_info,
            submodel_info,
            module_info,
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
/// * `defined_parameters_info` - Information about available parameters in the module
/// * `submodel_info` - Information about available submodels in the module
/// * `module_info` - Information about all available modules and their loading status
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
    submodel_tests: Vec<(Identifier, Vec<ast::declaration::ModelInput>)>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> (
    Vec<SubmodelTest>,
    HashMap<Identifier, Vec<SubmodelTestInputResolutionError>>,
) {
    let submodel_tests = submodel_tests.into_iter().map(|(submodel_name, inputs)| {
        // TODO: verify that there are no duplicate inputs
        let inputs: Vec<_> = inputs
            .into_iter()
            .map(|input| {
                let identifier = Identifier::new(input.name);
                let value = resolve_expr(
                    &input.value,
                    &HashSet::new(),
                    defined_parameters_info,
                    submodel_info,
                    module_info,
                )?;

                Ok((identifier, value))
            })
            .collect();

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
    use oneil_ast::{
        Parameter,
        declaration::ModelInput,
        expression::{BinaryOp, Expr, Literal, Variable},
        parameter::TraceLevel,
        test::Test,
    };
    use oneil_module::debug_info::TraceLevel as ModuleTraceLevel;
    use std::collections::HashSet;

    /// Creates test parameter information for testing
    fn create_empty_parameter_info() -> ParameterInfo<'static> {
        ParameterInfo::new(HashMap::new(), HashSet::new())
    }

    /// Creates test submodel information for testing
    fn create_empty_submodel_info() -> SubmodelInfo<'static> {
        SubmodelInfo::new(HashMap::new(), HashSet::new())
    }

    /// Creates test module information for testing
    fn create_empty_module_info() -> ModuleInfo<'static> {
        ModuleInfo::new(HashMap::new(), HashSet::new())
    }

    #[test]
    fn test_resolve_model_tests_empty() {
        // create the model tests
        let tests = vec![];

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
            Test {
                trace_level: TraceLevel::None,
                inputs: vec![],
                expr: Expr::Literal(Literal::Boolean(true)),
            },
            // test {x, y}: x > 0
            Test {
                trace_level: TraceLevel::None,
                inputs: vec!["x".to_string(), "y".to_string()],
                expr: Expr::BinaryOp {
                    left: Box::new(Expr::Variable(ast::expression::Variable::Identifier(
                        "x".to_string(),
                    ))),
                    op: ast::expression::BinaryOp::GreaterThan,
                    right: Box::new(Expr::Literal(Literal::Number(0.0))),
                },
            },
            // * test {param}: param == 42
            Test {
                trace_level: TraceLevel::Trace,
                inputs: vec!["param".to_string()],
                expr: Expr::BinaryOp {
                    left: Box::new(Expr::Variable(ast::expression::Variable::Identifier(
                        "param".to_string(),
                    ))),
                    op: ast::expression::BinaryOp::Eq,
                    right: Box::new(Expr::Literal(Literal::Number(42.0))),
                },
            },
        ];

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 3);

        let test_0 = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test_0.trace_level(), &ModuleTraceLevel::None);
        assert_eq!(test_0.inputs().len(), 0);

        let test_1 = resolved_tests.get(&TestIndex::new(1)).unwrap();
        assert_eq!(test_1.trace_level(), &ModuleTraceLevel::None);
        assert_eq!(test_1.inputs().len(), 2);
        assert!(test_1.inputs().contains(&Identifier::new("x")));
        assert!(test_1.inputs().contains(&Identifier::new("y")));

        let test_2 = resolved_tests.get(&TestIndex::new(2)).unwrap();
        assert_eq!(test_2.trace_level(), &ModuleTraceLevel::Trace);
        assert_eq!(test_2.inputs().len(), 1);
        assert!(test_2.inputs().contains(&Identifier::new("param")));
    }

    #[test]
    fn test_resolve_model_tests_with_debug_trace() {
        // create the model tests
        let tests = vec![
            // ** test {x}: true
            Test {
                trace_level: TraceLevel::Debug,
                inputs: vec!["x".to_string()],
                expr: Expr::Literal(Literal::Boolean(true)),
            },
        ];

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert_eq!(resolved_tests.len(), 1);
        let test = resolved_tests.get(&TestIndex::new(0)).unwrap();
        assert_eq!(test.trace_level(), &ModuleTraceLevel::Debug);
        assert_eq!(test.inputs().len(), 1);
        assert!(test.inputs().contains(&Identifier::new("x")));
    }

    #[test]
    fn test_resolve_model_tests_with_undefined_variable() {
        // create the model tests
        let tests = vec![
            // test {x}: undefined_var
            Test {
                trace_level: TraceLevel::None,
                inputs: vec!["x".to_string()],
                expr: Expr::Variable(Variable::Identifier("undefined_var".to_string())),
            },
        ];

        // resolve the model tests
        let (resolved_tests, errors) = resolve_model_tests(
            tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
            Test {
                trace_level: TraceLevel::None,
                inputs: vec!["x".to_string()],
                expr: Expr::Literal(Literal::Boolean(true)),
            },
            // test {y}: undefined_var
            Test {
                trace_level: TraceLevel::Trace,
                inputs: vec!["y".to_string()],
                expr: Expr::Variable(Variable::Identifier("undefined_var".to_string())),
            },
        ];

        let (resolved_tests, errors) = resolve_model_tests(
            tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
        assert_eq!(test.trace_level(), &ModuleTraceLevel::None);
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
            &create_empty_module_info(),
        );

        // check the errors
        assert!(errors.is_empty());

        // check the resolved tests
        assert!(resolved_tests.is_empty());
    }

    #[test]
    fn test_resolve_submodel_tests_basic() {
        // create the submodel tests
        let submodel_tests = vec![
            // use my_sensor(location = "north", height = 100) as sensor
            (
                Identifier::new("sensor"),
                vec![
                    ModelInput {
                        name: "location".to_string(),
                        value: Expr::Literal(Literal::String("north".to_string())),
                    },
                    ModelInput {
                        name: "height".to_string(),
                        value: Expr::Literal(Literal::Number(100.0)),
                    },
                ],
            ),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
        let submodel_tests = vec![
            // use my_sensor1(param1 = 10) as sensor1
            (
                Identifier::new("sensor1"),
                vec![ModelInput {
                    name: "param1".to_string(),
                    value: Expr::Literal(Literal::Number(10.0)),
                }],
            ),
            // use my_sensor2(param2 = "value") as sensor2
            (
                Identifier::new("sensor2"),
                vec![ModelInput {
                    name: "param2".to_string(),
                    value: Expr::Literal(Literal::String("value".to_string())),
                }],
            ),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
        let submodel_tests = vec![
            // use my_sensor(param = undefined_var) as sensor
            (
                Identifier::new("sensor"),
                vec![ModelInput {
                    name: "param".to_string(),
                    value: Expr::Variable(Variable::Identifier("undefined_var".to_string())),
                }],
            ),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
        let submodel_tests = vec![
            // use my_sensor1(param1 = 10) as sensor1
            (
                Identifier::new("sensor1"),
                vec![ModelInput {
                    name: "param1".to_string(),
                    value: Expr::Literal(Literal::Number(10.0)),
                }],
            ),
            // use my_sensor2(param2 = undefined_var) as sensor2
            (
                Identifier::new("sensor2"),
                vec![ModelInput {
                    name: "param2".to_string(),
                    value: Expr::Variable(Variable::Identifier("undefined_var".to_string())),
                }],
            ),
        ];

        // resolve the submodel tests
        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
        let submodel_tests = vec![
            // use my_sensor(calculation = 5 + 3, is_valid = 10 > 5) as sensor
            (
                Identifier::new("sensor"),
                vec![
                    ModelInput {
                        name: "calculation".to_string(),
                        value: Expr::BinaryOp {
                            left: Box::new(Expr::Literal(Literal::Number(5.0))),
                            op: BinaryOp::Add,
                            right: Box::new(Expr::Literal(Literal::Number(3.0))),
                        },
                    },
                    ModelInput {
                        name: "is_valid".to_string(),
                        value: Expr::BinaryOp {
                            left: Box::new(Expr::Literal(Literal::Number(10.0))),
                            op: BinaryOp::GreaterThan,
                            right: Box::new(Expr::Literal(Literal::Number(5.0))),
                        },
                    },
                ],
            ),
        ];

        let (resolved_tests, errors) = resolve_submodel_tests(
            submodel_tests,
            &create_empty_parameter_info(),
            &create_empty_submodel_info(),
            &create_empty_module_info(),
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
        let submodel_tests = vec![(
            Identifier::new("sensor"),
            vec![ModelInput {
                name: "param".to_string(),
                value: Expr::Variable(Variable::Identifier("test_param".to_string())),
            }],
        )];

        // create the parameter info
        let test_param_id = Identifier::new("test_param");
        let test_param = oneil_module::parameter::Parameter::new(
            HashSet::new(),
            test_param_id.clone(),
            oneil_module::parameter::ParameterValue::Simple(
                oneil_module::expr::Expr::literal(oneil_module::expr::Literal::number(10.0)),
                None,
            ),
            oneil_module::parameter::Limits::default(),
            false,
            oneil_module::debug_info::TraceLevel::None,
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
            &create_empty_module_info(),
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
