//! Parameter resolution model for the Oneil model loader.
//!
//! This module handles the resolution of parameters within a Oneil model. It performs
//! dependency analysis, circular dependency detection, and converts AST parameters into
//! resolved model parameters.
//!
//! # Overview
//!
//! The parameter resolution process consists of several steps:
//!
//! 1. **Dependency Analysis**: Extract all internal dependencies between parameters
//! 2. **Topological Resolution**: Resolve parameters in dependency order
//! 3. **Circular Dependency Detection**: Detect and report circular dependencies
//! 4. **Expression Resolution**: Resolve parameter values and limits
//!
//! # Key Concepts
//!
//! - **Internal Dependencies**: Dependencies on parameters defined within the same model
//! - **External Dependencies**: Dependencies on parameters from other models (handled elsewhere)
//! - **Circular Dependencies**: When parameter A depends on parameter B, which depends on A
//!

use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_ir::{
    parameter::{Parameter, ParameterCollection, ParameterValue, PiecewiseExpr},
    reference::Identifier,
};

use crate::{
    error::{self, ParameterResolutionError},
    loader::resolver::{
        ModelInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
        trace_level::resolve_trace_level, unit::resolve_unit,
    },
    util::{Stack, builder::ParameterCollectionBuilder, info::InfoMap},
};

/// Resolves a collection of AST parameters into resolved model parameters.
///
/// This function performs the complete parameter resolution process:
/// - Analyzes dependencies between parameters
/// - Detects circular dependencies
/// - Resolves parameter values and limits
/// - Handles both simple and piecewise parameter values
///
/// # Arguments
///
/// * `parameters` - Vector of AST parameters to resolve
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about available models
///
/// # Returns
///
/// A tuple containing:
/// - `ParameterCollection`: Successfully resolved parameters
/// - `HashMap<Identifier, Vec<ParameterResolutionError>>`: Resolution errors by parameter identifier
///
/// # Errors
///
/// The function may return errors for:
/// - Circular dependencies between parameters
/// - Undefined variable references
/// - Invalid expressions in parameter values or limits
/// - Missing submodel references
pub fn resolve_parameters(
    parameters: Vec<ast::Parameter>,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> (
    ParameterCollection,
    HashMap<Identifier, Vec<ParameterResolutionError>>,
) {
    // TODO: verify that no duplicate parameters are defined

    let parameter_map: HashMap<Identifier, ast::Parameter> = parameters
        .into_iter()
        .map(|parameter| (Identifier::new(&parameter.ident), parameter))
        .collect();

    // note that an 'internal dependency' is a dependency on a parameter
    // that is defined within the current model
    let dependencies = get_all_parameter_internal_dependencies(&parameter_map);

    let resolved_parameters = ParameterCollectionBuilder::new();
    let visited = HashSet::new();

    let (resolved_parameters, _visited) = parameter_map.keys().fold(
        (resolved_parameters, visited),
        |(resolved_parameters, visited), parameter_identifier| {
            let mut parameter_stack = Stack::new();

            resolve_parameter(
                parameter_identifier,
                &parameter_map,
                &dependencies,
                submodel_info,
                model_info,
                &mut parameter_stack,
                resolved_parameters,
                visited,
            )
        },
    );

    let resolved_parameters = resolved_parameters.try_into();

    match resolved_parameters {
        Ok(resolved_parameters) => (resolved_parameters, HashMap::new()),
        Err((resolved_parameters, resolution_errors)) => (resolved_parameters, resolution_errors),
    }
}

/// Analyzes all parameters to extract their internal dependencies.
///
/// Creates a mapping from parameter identifier to the set of other parameters
/// it depends on within the same model.
///
/// # Arguments
///
/// * `parameter_map` - Map of parameter identifiers to AST parameters
///
/// # Returns
///
/// A map from parameter identifier to its set of internal dependencies
fn get_all_parameter_internal_dependencies<'a>(
    parameter_map: &'a HashMap<Identifier, ast::Parameter>,
) -> HashMap<&'a Identifier, HashSet<Identifier>> {
    let dependencies = HashMap::new();

    parameter_map
        .keys()
        .fold(dependencies, |mut dependencies, identifier| {
            let parameter = parameter_map
                .get(identifier)
                .expect("parameter should exist");

            let param_dependencies = get_parameter_internal_dependencies(parameter);

            dependencies.insert(identifier, param_dependencies);

            dependencies
        })
}

/// Extracts internal dependencies from a single parameter.
///
/// Analyzes the parameter's value and limits to find references to other
/// parameters within the same model.
///
/// # Arguments
///
/// * `parameter` - The AST parameter to analyze
///
/// # Returns
///
/// A set of parameter identifiers that this parameter depends on
fn get_parameter_internal_dependencies(parameter: &ast::Parameter) -> HashSet<Identifier> {
    let dependencies = HashSet::new();

    let dependencies = match &parameter.limits {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            let dependencies = get_expr_internal_dependencies(min, dependencies);
            let dependencies = get_expr_internal_dependencies(max, dependencies);
            dependencies
        }
        Some(ast::parameter::Limits::Discrete { values }) => {
            values.iter().fold(dependencies, |dependencies, expr| {
                let dependencies = get_expr_internal_dependencies(expr, dependencies);
                dependencies
            })
        }
        None => dependencies,
    };

    let dependencies = match &parameter.value {
        ast::parameter::ParameterValue::Simple(expr, _) => {
            get_expr_internal_dependencies(expr, dependencies)
        }
        ast::parameter::ParameterValue::Piecewise(piecewise, _) => {
            piecewise
                .parts
                .iter()
                .fold(dependencies, |dependencies, part| {
                    let dependencies = get_expr_internal_dependencies(&part.if_expr, dependencies);
                    let dependencies = get_expr_internal_dependencies(&part.expr, dependencies);
                    dependencies
                })
        }
    };

    dependencies
}

/// Extracts internal dependencies from an expression.
///
/// Recursively traverses the expression tree to find variable references
/// that correspond to parameters within the same model.
///
/// # Arguments
///
/// * `expr` - The expression to analyze
/// * `dependencies` - Accumulated set of dependencies
///
/// # Returns
///
/// Updated set of dependencies including any found in this expression
fn get_expr_internal_dependencies(
    expr: &ast::Expr,
    mut dependencies: HashSet<Identifier>,
) -> HashSet<Identifier> {
    match expr {
        ast::Expr::BinaryOp { op: _, left, right } => {
            let dependencies = get_expr_internal_dependencies(&left, dependencies);
            let dependencies = get_expr_internal_dependencies(&right, dependencies);
            dependencies
        }

        ast::Expr::UnaryOp { op: _, expr } => get_expr_internal_dependencies(expr, dependencies),

        ast::Expr::FunctionCall { name: _, args } => {
            args.iter().fold(dependencies, |dependencies, arg| {
                let dependencies = get_expr_internal_dependencies(arg, dependencies);
                dependencies
            })
        }

        ast::Expr::Variable(variable) => match variable {
            ast::expression::Variable::Identifier(identifier) => {
                let identifier = Identifier::new(identifier);
                dependencies.insert(identifier);
                dependencies
            }

            ast::expression::Variable::Accessor {
                parent: _,
                component: _,
            } => {
                // an accessor implies that the dependency is on a parameter
                // outside of the current model, so it doesn't count as an
                // internal dependency
                dependencies
            }
        },
        ast::Expr::Literal(_) => dependencies,
    }
}

/// Resolves a single parameter with dependency tracking.
///
/// This function handles the recursive resolution of a parameter, including:
/// - Circular dependency detection
/// - Dependency resolution order
/// - Parameter value and limit resolution
///
/// # Arguments
///
/// * `parameter_identifier` - Identifier of the parameter to resolve
/// * `parameter_map` - Map of all available parameters
/// * `dependencies` - Dependency graph for all parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about available models
/// * `parameter_stack` - Stack for tracking resolution order (for circular dependency detection)
/// * `resolved_parameters` - Builder for collecting resolved parameters
/// * `visited` - Set of already visited parameters
///
/// # Returns
///
/// A tuple containing:
/// - Updated parameter collection builder
/// - Updated visited set
/// - Result indicating success or resolution errors
fn resolve_parameter(
    parameter_identifier: &Identifier,
    parameter_map: &HashMap<Identifier, oneil_ast::Parameter>,
    dependencies: &HashMap<&Identifier, HashSet<Identifier>>,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
    parameter_stack: &mut Stack<Identifier>,
    mut resolved_parameters: ParameterCollectionBuilder,
    mut visited: HashSet<Identifier>,
) -> (ParameterCollectionBuilder, HashSet<Identifier>) {
    // check that the parameter exists
    if !parameter_map.contains_key(parameter_identifier) {
        // This is technically a resolution error. However, this error will
        // be caught later when the variable is resolved. In order to avoid
        // duplicate errors, we return Ok(()) and let the variable resolution
        // handle the "not found" error
        return (resolved_parameters, visited);
    }

    assert!(
        dependencies.contains_key(parameter_identifier),
        "parameter dependencies for '{:?}' not found",
        parameter_identifier
    );

    // check for circular dependencies
    if let Some(circular_dependency) =
        parameter_stack.find_circular_dependency(parameter_identifier)
    {
        resolved_parameters.add_error_list(
            parameter_identifier.clone(),
            vec![ParameterResolutionError::circular_dependency(
                circular_dependency,
            )],
        );
        return (resolved_parameters, visited);
    }

    // check if the parameter has already been visited
    if visited.contains(parameter_identifier) {
        return (resolved_parameters, visited);
    }
    visited.insert(parameter_identifier.clone());

    // resolve the parameter dependencies
    let parameter_dependencies = dependencies.get(parameter_identifier).unwrap();

    // add the parameter to the stack
    parameter_stack.push(parameter_identifier.clone());

    // resolve the parameter dependencies
    (resolved_parameters, visited) = parameter_dependencies.iter().fold(
        (resolved_parameters, visited),
        |(resolved_parameters, visited), dependency| {
            resolve_parameter(
                dependency,
                parameter_map,
                dependencies,
                submodel_info,
                model_info,
                parameter_stack,
                resolved_parameters,
                visited,
            )
        },
    );

    // remove the parameter from the stack
    parameter_stack.pop();

    // resolve the parameter
    let parameter = parameter_map.get(parameter_identifier).unwrap();

    let ast::Parameter {
        name: _, // this should be stored in documentation
        ident,
        value,
        limits,
        is_performance,
        trace_level,
        note: _, // this should be stored in documentation
    } = parameter;

    let ident = Identifier::new(ident);

    let defined_parameters = resolved_parameters.get_defined_parameters();
    let defined_parameters_with_errors = resolved_parameters.get_parameters_with_errors();
    let defined_parameters_info = InfoMap::new(
        defined_parameters.iter().collect(),
        defined_parameters_with_errors,
    );

    let value = resolve_parameter_value(value, &defined_parameters_info, submodel_info, model_info);

    let limits = resolve_limits(
        limits.as_ref(),
        &defined_parameters_info,
        submodel_info,
        model_info,
    );

    let trace_level = resolve_trace_level(trace_level);

    let resolved_parameters = match error::combine_errors(value, limits) {
        Ok((value, limits)) => {
            let parameter = Parameter::new(
                parameter_dependencies.clone(),
                ident,
                value,
                limits,
                *is_performance,
                trace_level,
            );

            resolved_parameters.add_parameter(parameter_identifier.clone(), parameter);

            resolved_parameters
        }
        Err(errors) => {
            resolved_parameters.add_error_list(parameter_identifier.clone(), errors);

            resolved_parameters
        }
    };

    (resolved_parameters, visited)
}

/// Resolves a parameter value expression.
///
/// Handles both simple and piecewise parameter values, resolving expressions
/// and converting them to the appropriate model types.
///
/// # Arguments
///
/// * `value` - The AST parameter value to resolve
/// * `defined_parameters_info` - Information about already resolved parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about available models
///
/// # Returns
///
/// A resolved parameter value or a list of resolution errors
fn resolve_parameter_value(
    value: &ast::parameter::ParameterValue,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> Result<ParameterValue, Vec<ParameterResolutionError>> {
    let local_variables = HashSet::new();

    match value {
        ast::parameter::ParameterValue::Simple(expr, unit) => {
            let expr = resolve_expr(
                expr,
                &local_variables,
                defined_parameters_info,
                submodel_info,
                model_info,
            )
            .map_err(error::convert_errors)?;

            let unit = unit.as_ref().map(resolve_unit);

            Ok(ParameterValue::simple(expr, unit))
        }

        ast::parameter::ParameterValue::Piecewise(piecewise, unit) => {
            let exprs = piecewise.parts.iter().map(|part| {
                let expr = resolve_expr(
                    &part.expr,
                    &local_variables,
                    defined_parameters_info,
                    submodel_info,
                    model_info,
                )
                .map_err(error::convert_errors);

                let if_expr = resolve_expr(
                    &part.if_expr,
                    &local_variables,
                    defined_parameters_info,
                    submodel_info,
                    model_info,
                )
                .map_err(error::convert_errors);

                let (expr, if_expr) = error::combine_errors(expr, if_expr)?;

                Ok(PiecewiseExpr::new(expr, if_expr))
            });

            let unit = unit.as_ref().map(resolve_unit);

            let exprs = error::combine_error_list(exprs)?;

            Ok(ParameterValue::piecewise(exprs, unit))
        }
    }
}

/// Resolves parameter limits.
///
/// Handles both continuous (min/max) and discrete limits, resolving expressions
/// and converting them to the appropriate model types.
///
/// # Arguments
///
/// * `limits` - Optional AST parameter limits to resolve
/// * `defined_parameters_info` - Information about already resolved parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about available models
///
/// # Returns
///
/// Resolved parameter limits or a list of resolution errors
fn resolve_limits(
    limits: Option<&ast::parameter::Limits>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> Result<oneil_ir::parameter::Limits, Vec<ParameterResolutionError>> {
    let local_variables = HashSet::new();
    match limits {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            let min = resolve_expr(
                min,
                &local_variables,
                defined_parameters_info,
                submodel_info,
                model_info,
            )
            .map_err(error::convert_errors);

            let max = resolve_expr(
                max,
                &local_variables,
                defined_parameters_info,
                submodel_info,
                model_info,
            )
            .map_err(error::convert_errors);

            let (min, max) = error::combine_errors(min, max)?;

            Ok(oneil_ir::parameter::Limits::continuous(min, max))
        }
        Some(ast::parameter::Limits::Discrete { values }) => {
            let values = values.into_iter().map(|value| {
                resolve_expr(
                    value,
                    &local_variables,
                    defined_parameters_info,
                    submodel_info,
                    model_info,
                )
                .map_err(error::convert_errors)
            });

            let values = error::combine_error_list(values)?;

            Ok(oneil_ir::parameter::Limits::discrete(values))
        }
        None => Ok(oneil_ir::parameter::Limits::default()),
    }
}

#[cfg(test)]
mod tests {
    use crate::error::VariableResolutionError;

    use super::*;
    use oneil_ast as ast;
    use oneil_ir::{
        parameter::{Limits, ParameterValue},
        reference::Identifier,
    };

    // Helper function to create a simple parameter
    fn create_simple_parameter(ident: &str, value: f64) -> ast::Parameter {
        ast::Parameter {
            name: format!("Parameter {}", ident),
            ident: ident.to_string(),
            value: ast::parameter::ParameterValue::Simple(
                ast::Expr::Literal(ast::expression::Literal::Number(value)),
                None,
            ),
            limits: None,
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
            note: None,
        }
    }

    // Helper function to create a dependent parameter
    fn create_dependent_parameter(ident: &str, dependency: &str) -> ast::Parameter {
        ast::Parameter {
            name: format!("Parameter {}", ident),
            ident: ident.to_string(),
            value: ast::parameter::ParameterValue::Simple(
                ast::Expr::BinaryOp {
                    op: ast::expression::BinaryOp::Add,
                    left: Box::new(ast::Expr::Variable(ast::expression::Variable::Identifier(
                        dependency.to_string(),
                    ))),
                    right: Box::new(ast::Expr::Literal(ast::expression::Literal::Number(1.0))),
                },
                None,
            ),
            limits: None,
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
            note: None,
        }
    }

    // Helper function to create mock model and submodel info
    fn create_mock_info<'a>() -> (SubmodelInfo<'a>, ModelInfo<'a>) {
        let submodel_info = SubmodelInfo::new(HashMap::new(), HashSet::new());
        let model_info = ModelInfo::new(HashMap::new(), HashSet::new());
        (submodel_info, model_info)
    }

    #[test]
    fn test_resolve_parameters_empty() {
        // create the parameters
        let parameters = vec![];

        // create the submodel and model info
        let (submodel_info, model_info) = create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) = resolve_parameters(parameters, &submodel_info, &model_info);

        // test the errors
        assert!(errors.is_empty());

        // test the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn test_resolve_parameters_simple() {
        // create the parameters
        let parameters = vec![
            create_simple_parameter("a", 10.0),
            create_simple_parameter("b", 20.0),
        ];

        // create the submodel and model info
        let (submodel_info, model_info) = create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) = resolve_parameters(parameters, &submodel_info, &model_info);

        // test the errors
        assert!(errors.is_empty());

        // test the resolved parameters
        assert_eq!(resolved_params.len(), 2);

        let param_a = resolved_params.get(&Identifier::new("a")).unwrap();
        let param_b = resolved_params.get(&Identifier::new("b")).unwrap();

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().is_empty());
    }

    #[test]
    fn test_resolve_parameters_with_dependencies() {
        // create the parameters
        let parameters = vec![
            create_simple_parameter("a", 10.0),
            create_dependent_parameter("b", "a"),
        ];

        // create the submodel and model info
        let (submodel_info, model_info) = create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) = resolve_parameters(parameters, &submodel_info, &model_info);

        // test the errors
        assert!(errors.is_empty());

        // test the resolved parameters
        assert_eq!(resolved_params.len(), 2);

        let param_a = resolved_params.get(&Identifier::new("a")).unwrap();
        let param_b = resolved_params.get(&Identifier::new("b")).unwrap();

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().contains(&Identifier::new("a")));
    }

    #[test]
    fn test_resolve_parameters_circular_dependency() {
        // create the parameters
        let parameters = vec![
            create_dependent_parameter("a", "b"),
            create_dependent_parameter("b", "a"),
        ];

        // create the submodel and model info
        let (submodel_info, model_info) = create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) = resolve_parameters(parameters, &submodel_info, &model_info);

        // test the errors
        assert!(!errors.is_empty());

        assert!(errors.contains_key(&Identifier::new("a")));
        assert!(errors.contains_key(&Identifier::new("b")));

        let a_errors = errors.get(&Identifier::new("a")).unwrap();
        let b_errors = errors.get(&Identifier::new("b")).unwrap();

        // test that both parameters have errors, one is a circular dependency
        // error and both have a "parameter had error" error
        assert_eq!(a_errors.len() + b_errors.len(), 3);

        // the order in which parameters are resolved is non-deterministic,
        // so we need to check for a circular dependency error in either
        let a_has_circular_dependency = a_errors
            .iter()
            .any(|e| matches!(e, ParameterResolutionError::CircularDependency(_)));
        let b_has_circular_dependency = b_errors
            .iter()
            .any(|e| matches!(e, ParameterResolutionError::CircularDependency(_)));
        assert!(a_has_circular_dependency || b_has_circular_dependency);

        assert!(a_errors.iter().any(|e| matches!(
            e,
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::ParameterHasError(ident)
            )
            if ident == &Identifier::new("b")
        )));

        assert!(b_errors.iter().any(|e| matches!(
            e,
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::ParameterHasError(ident)
            )
            if ident == &Identifier::new("a")
        )));

        // test the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn test_get_parameter_internal_dependencies_simple() {
        let parameter = create_simple_parameter("a", 10.0);
        let dependencies = get_parameter_internal_dependencies(&parameter);

        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_variable() {
        let parameter = create_dependent_parameter("a", "b");
        let dependencies = get_parameter_internal_dependencies(&parameter);

        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&Identifier::new("b")));
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_limits() {
        let parameter = ast::Parameter {
            name: "Test Parameter".to_string(),
            ident: "test".to_string(),
            value: ast::parameter::ParameterValue::Simple(
                ast::Expr::Literal(ast::expression::Literal::Number(5.0)),
                None,
            ),
            limits: Some(ast::parameter::Limits::Continuous {
                min: ast::Expr::Variable(ast::expression::Variable::Identifier(
                    "min_val".to_string(),
                )),
                max: ast::Expr::Variable(ast::expression::Variable::Identifier(
                    "max_val".to_string(),
                )),
            }),
            is_performance: false,
            trace_level: ast::parameter::TraceLevel::None,
            note: None,
        };

        let dependencies = get_parameter_internal_dependencies(&parameter);

        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&Identifier::new("min_val")));
        assert!(dependencies.contains(&Identifier::new("max_val")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_literal() {
        let expr = ast::Expr::Literal(ast::expression::Literal::Number(42.0));

        let result = get_expr_internal_dependencies(&expr, HashSet::new());

        assert!(result.is_empty());
    }

    #[test]
    fn test_get_expr_internal_dependencies_variable() {
        let expr = ast::Expr::Variable(ast::expression::Variable::Identifier(
            "test_var".to_string(),
        ));

        let result = get_expr_internal_dependencies(&expr, HashSet::new());

        assert_eq!(result.len(), 1);
        assert!(result.contains(&Identifier::new("test_var")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_binary_op() {
        let expr = ast::Expr::BinaryOp {
            op: ast::expression::BinaryOp::Add,
            left: Box::new(ast::Expr::Variable(ast::expression::Variable::Identifier(
                "a".to_string(),
            ))),
            right: Box::new(ast::Expr::Variable(ast::expression::Variable::Identifier(
                "b".to_string(),
            ))),
        };

        let result = get_expr_internal_dependencies(&expr, HashSet::new());

        assert_eq!(result.len(), 2);
        assert!(result.contains(&Identifier::new("a")));
        assert!(result.contains(&Identifier::new("b")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_function_call() {
        let expr = ast::Expr::FunctionCall {
            name: "test_func".to_string(),
            args: vec![
                ast::Expr::Variable(ast::expression::Variable::Identifier("arg1".to_string())),
                ast::Expr::Variable(ast::expression::Variable::Identifier("arg2".to_string())),
            ],
        };

        let result = get_expr_internal_dependencies(&expr, HashSet::new());

        assert_eq!(result.len(), 2);
        assert!(result.contains(&Identifier::new("arg1")));
        assert!(result.contains(&Identifier::new("arg2")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_accessor() {
        let expr = ast::Expr::Variable(ast::expression::Variable::Accessor {
            parent: "parent".to_string(),
            component: Box::new(ast::expression::Variable::Identifier(
                "component".to_string(),
            )),
        });

        let result = get_expr_internal_dependencies(&expr, HashSet::new());

        // Accessors don't count as internal dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_parameter_value_simple() {
        let value = ast::parameter::ParameterValue::Simple(
            ast::Expr::Literal(ast::expression::Literal::Number(42.0)),
            None,
        );
        let (submodel_info, model_info) = create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        let result = resolve_parameter_value(
            &value,
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        assert!(result.is_ok());
        let resolved_value = result.unwrap();
        assert!(matches!(resolved_value, ParameterValue::Simple(_, None)));
    }

    #[test]
    fn test_resolve_limits_none() {
        let (submodel_info, model_info) = create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        let result = resolve_limits(None, &defined_parameters_info, &submodel_info, &model_info);

        assert!(result.is_ok());
        let limits = result.unwrap();
        assert!(matches!(limits, Limits::Default));
    }

    #[test]
    fn test_resolve_limits_continuous() {
        let limits = ast::parameter::Limits::Continuous {
            min: ast::Expr::Literal(ast::expression::Literal::Number(0.0)),
            max: ast::Expr::Literal(ast::expression::Literal::Number(100.0)),
        };
        let (submodel_info, model_info) = create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        let result = resolve_limits(
            Some(&limits),
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        assert!(result.is_ok());
        let resolved_limits = result.unwrap();
        assert!(matches!(resolved_limits, Limits::Continuous { .. }));
    }

    #[test]
    fn test_resolve_limits_discrete() {
        let limits = ast::parameter::Limits::Discrete {
            values: vec![
                ast::Expr::Literal(ast::expression::Literal::Number(1.0)),
                ast::Expr::Literal(ast::expression::Literal::Number(2.0)),
                ast::Expr::Literal(ast::expression::Literal::Number(3.0)),
            ],
        };
        let (submodel_info, model_info) = create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        let result = resolve_limits(
            Some(&limits),
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        assert!(result.is_ok());
        let resolved_limits = result.unwrap();
        assert!(matches!(resolved_limits, Limits::Discrete { .. }));
    }
}
