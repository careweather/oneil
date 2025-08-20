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
    span::{Span, WithSpan},
};

use crate::{
    BuiltinRef,
    error::{self, ParameterResolutionError},
    loader::resolver::{
        ModelInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
        trace_level::resolve_trace_level, unit::resolve_unit,
    },
    util::{Stack, builder::ParameterCollectionBuilder, get_span_from_ast_span, info::InfoMap},
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
/// * `builtin_ref` - Set of builtin variables
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
    parameters: Vec<&ast::parameter::ParameterNode>,
    builtin_ref: &impl BuiltinRef,
    submodel_info: &SubmodelInfo<'_>,
    model_info: &ModelInfo<'_>,
) -> (
    ParameterCollection,
    HashMap<Identifier, Vec<ParameterResolutionError>>,
) {
    let (parameter_map, duplicate_errors) = parameters.into_iter().fold(
        (HashMap::new(), HashMap::new()),
        |(mut parameter_map, mut duplicate_errors), parameter| {
            let ident = Identifier::new(parameter.ident().as_str());
            let ident_span = get_span_from_ast_span(parameter.ident().node_span());

            let maybe_original_parameter = parameter_map.get(&ident);
            if let Some((original_ident_span, _)) = maybe_original_parameter {
                let original_ident_span: &Span = original_ident_span; // to help type inference
                duplicate_errors.insert(
                    ident.clone(),
                    vec![ParameterResolutionError::duplicate_parameter(
                        ident,
                        original_ident_span.clone(),
                        ident_span.clone(),
                    )],
                );
            } else {
                parameter_map.insert(ident, (ident_span, parameter));
            }

            (parameter_map, duplicate_errors)
        },
    );

    // note that an 'internal dependency' is a dependency on a parameter
    // that is defined within the current model
    let dependencies = get_all_parameter_internal_dependencies(&parameter_map);

    let resolved_parameters = ParameterCollectionBuilder::new();
    let visited = HashSet::new();

    let (resolved_parameters, _visited) = parameter_map.iter().fold(
        (resolved_parameters, visited),
        |(resolved_parameters, visited), (parameter_identifier, (parameter_span, _parameter))| {
            let mut parameter_stack = Stack::new();

            let parameter_identifier_with_span =
                WithSpan::new(parameter_identifier.clone(), parameter_span.clone());

            resolve_parameter(
                parameter_identifier_with_span,
                &parameter_map,
                &dependencies,
                builtin_ref,
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
        Ok(resolved_parameters) => (resolved_parameters, duplicate_errors),
        Err((resolved_parameters, resolution_errors)) => {
            // combine the duplicate errors with the resolution errors
            let mut errors = resolution_errors;
            for (identifier, duplicate_errors) in duplicate_errors {
                errors
                    .entry(identifier)
                    .or_insert(vec![])
                    .extend(duplicate_errors);
            }

            (resolved_parameters, errors)
        }
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
    parameter_map: &'a HashMap<Identifier, (Span, &'a ast::parameter::ParameterNode)>,
) -> HashMap<&'a Identifier, HashSet<WithSpan<Identifier>>> {
    let dependencies = HashMap::new();

    parameter_map
        .keys()
        .fold(dependencies, |mut dependencies, identifier| {
            let (_, parameter) = parameter_map
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
fn get_parameter_internal_dependencies(
    parameter: &ast::Parameter,
) -> HashSet<WithSpan<Identifier>> {
    let dependencies = HashSet::new();

    let limits = parameter.limits().map(|l| l.node_value());
    let dependencies = match limits {
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

    let dependencies = match &parameter.value().node_value() {
        ast::parameter::ParameterValue::Simple(expr, _) => {
            get_expr_internal_dependencies(expr, dependencies)
        }
        ast::parameter::ParameterValue::Piecewise(piecewise, _) => {
            piecewise.iter().fold(dependencies, |dependencies, part| {
                let dependencies = get_expr_internal_dependencies(&part.if_expr(), dependencies);
                let dependencies = get_expr_internal_dependencies(&part.expr(), dependencies);
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
    mut dependencies: HashSet<WithSpan<Identifier>>,
) -> HashSet<WithSpan<Identifier>> {
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

        ast::Expr::Variable(variable) => match variable.node_value() {
            ast::expression::Variable::Identifier(identifier) => {
                let identifier_span = get_span_from_ast_span(identifier.node_span());
                let identifier = Identifier::new(identifier.as_str());
                dependencies.insert(WithSpan::new(identifier, identifier_span));
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
        ast::Expr::Parenthesized { expr } => get_expr_internal_dependencies(expr, dependencies),
        ast::Expr::ComparisonOp {
            op: _,
            left,
            right,
            rest_chained,
        } => {
            let dependencies = get_expr_internal_dependencies(&left, dependencies);
            let dependencies = get_expr_internal_dependencies(&right, dependencies);
            // Handle chained comparisons
            rest_chained
                .iter()
                .fold(dependencies, |dependencies, (_, expr)| {
                    get_expr_internal_dependencies(expr, dependencies)
                })
        }
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
/// * `builtin_ref` - Set of builtin variables
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
    parameter_identifier: WithSpan<Identifier>,
    parameter_map: &HashMap<Identifier, (Span, &ast::parameter::ParameterNode)>,
    dependencies: &HashMap<&Identifier, HashSet<WithSpan<Identifier>>>,
    builtin_ref: &impl BuiltinRef,
    submodel_info: &SubmodelInfo<'_>,
    model_info: &ModelInfo<'_>,
    parameter_stack: &mut Stack<Identifier>,
    mut resolved_parameters: ParameterCollectionBuilder,
    mut visited: HashSet<Identifier>,
) -> (ParameterCollectionBuilder, HashSet<Identifier>) {
    let parameter_identifier_span = parameter_identifier.span().clone();
    let parameter_identifier = parameter_identifier.take_value();

    // check that the parameter exists
    if !parameter_map.contains_key(&parameter_identifier) {
        // This is technically a resolution error. However, this error will
        // be caught later when the variable is resolved. In order to avoid
        // duplicate errors, we return Ok(()) and let the variable resolution
        // handle the "not found" error
        //
        // This also accounts for the fact that the parameter may be a builtin
        return (resolved_parameters, visited);
    }

    assert!(
        dependencies.contains_key(&parameter_identifier),
        "parameter dependencies for '{:?}' not found",
        parameter_identifier
    );

    // check for circular dependencies
    if let Some(circular_dependency) =
        parameter_stack.find_circular_dependency(&parameter_identifier)
    {
        let reference_span = parameter_identifier_span;
        resolved_parameters.add_error_list(
            parameter_identifier.clone(),
            vec![ParameterResolutionError::circular_dependency(
                circular_dependency,
                reference_span,
            )],
        );
        return (resolved_parameters, visited);
    }

    // check if the parameter has already been visited
    if visited.contains(&parameter_identifier) {
        return (resolved_parameters, visited);
    }
    visited.insert(parameter_identifier.clone());

    // resolve the parameter dependencies
    let parameter_dependencies = dependencies.get(&parameter_identifier).unwrap();

    // add the parameter to the stack
    parameter_stack.push(parameter_identifier.clone());

    // resolve the parameter dependencies
    (resolved_parameters, visited) = parameter_dependencies.iter().fold(
        (resolved_parameters, visited),
        |(resolved_parameters, visited), dependency| {
            resolve_parameter(
                dependency.clone(),
                parameter_map,
                dependencies,
                builtin_ref,
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
    let (_, parameter) = parameter_map.get(&parameter_identifier).unwrap();

    let ident = parameter_identifier.clone();

    let defined_parameters = resolved_parameters.get_defined_parameters();
    let defined_parameters_with_errors = resolved_parameters.get_parameters_with_errors();
    let defined_parameters_info = InfoMap::new(
        defined_parameters.iter().collect(),
        defined_parameters_with_errors,
    );

    let value = resolve_parameter_value(
        &parameter.value(),
        builtin_ref,
        &defined_parameters_info,
        submodel_info,
        model_info,
    );

    let limits = resolve_limits(
        parameter.limits(),
        builtin_ref,
        &defined_parameters_info,
        submodel_info,
        model_info,
    );

    let is_performance = parameter.performance_marker().is_some();

    let trace_level = resolve_trace_level(parameter.trace_level());

    let resolved_parameters = match error::combine_errors(value, limits) {
        Ok((value, limits)) => {
            let ident_span = get_span_from_ast_span(parameter.ident().node_span());
            let ident_with_span = oneil_ir::span::WithSpan::new(ident, ident_span);

            let parameter_dependencies = parameter_dependencies
                .iter()
                .map(|dependency| dependency.value().clone())
                .collect();

            let parameter = Parameter::new(
                parameter_dependencies,
                ident_with_span,
                value,
                limits,
                is_performance,
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
    builtin_ref: &impl BuiltinRef,
    defined_parameters_info: &ParameterInfo<'_>,
    submodel_info: &SubmodelInfo<'_>,
    model_info: &ModelInfo<'_>,
) -> Result<ParameterValue, Vec<ParameterResolutionError>> {
    match value {
        ast::parameter::ParameterValue::Simple(expr, unit) => {
            let expr = resolve_expr(
                expr,
                builtin_ref,
                defined_parameters_info,
                submodel_info,
                model_info,
            )
            .map_err(error::convert_errors)?;

            let unit = unit.as_ref().map(resolve_unit);

            Ok(ParameterValue::simple(expr, unit))
        }

        ast::parameter::ParameterValue::Piecewise(piecewise, unit) => {
            let exprs = piecewise.iter().map(|part| {
                let expr = resolve_expr(
                    &part.expr(),
                    builtin_ref,
                    defined_parameters_info,
                    submodel_info,
                    model_info,
                )
                .map_err(error::convert_errors);

                let if_expr = resolve_expr(
                    &part.if_expr(),
                    builtin_ref,
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
    limits: Option<&ast::parameter::LimitsNode>,
    builtin_ref: &impl BuiltinRef,
    defined_parameters_info: &ParameterInfo<'_>,
    submodel_info: &SubmodelInfo<'_>,
    model_info: &ModelInfo<'_>,
) -> Result<oneil_ir::parameter::Limits, Vec<ParameterResolutionError>> {
    match limits.map(|l| l.node_value()) {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            let min = resolve_expr(
                min,
                builtin_ref,
                defined_parameters_info,
                submodel_info,
                model_info,
            )
            .map_err(error::convert_errors);

            let max = resolve_expr(
                max,
                builtin_ref,
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
                    builtin_ref,
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

    mod helper {
        use crate::test::TestBuiltinRef;

        use super::*;

        /// Helper function to create a test span
        pub fn test_span(start: usize, end: usize) -> ast::Span {
            ast::Span::new(start, end - start, 0)
        }

        /// Helper function to create a literal expression node
        pub fn create_literal_expr_node(
            literal: ast::expression::Literal,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let literal_node = ast::node::Node::new(test_span(start, end), literal);
            let expr = ast::expression::Expr::Literal(literal_node);
            ast::node::Node::new(test_span(start, end), expr)
        }

        /// Helper function to create a variable expression node
        pub fn create_variable_expr_node(
            variable: ast::expression::VariableNode,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr = ast::expression::Expr::Variable(variable);
            ast::node::Node::new(test_span(start, end), expr)
        }

        /// Helper function to create a binary operation expression node
        pub fn create_binary_op_expr_node(
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

        /// Helper function to create a function call expression node
        pub fn create_function_call_expr_node(
            name: &str,
            args: Vec<ast::expression::ExprNode>,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            let name_node = ast::node::Node::new(test_span(start, start + name.len()), identifier);
            let expr = ast::expression::Expr::FunctionCall {
                name: name_node,
                args,
            };
            ast::node::Node::new(test_span(start, end), expr)
        }

        /// Helper function to create a simple identifier variable
        pub fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            let identifier_node = ast::node::Node::new(test_span(0, name.len()), identifier);
            let variable = ast::expression::Variable::Identifier(identifier_node);
            ast::node::Node::new(test_span(0, name.len()), variable)
        }

        /// Helper function to create a simple parameter with a literal value
        pub fn create_simple_parameter(ident: &str, value: f64) -> ast::parameter::ParameterNode {
            let label = ast::naming::Label::new(format!("Parameter {}", ident));
            let label_node = ast::node::Node::new(test_span(0, 0), label);
            let identifier = ast::naming::Identifier::new(ident.to_string());
            let ident_node = ast::node::Node::new(test_span(0, ident.len()), identifier);

            let literal = ast::expression::Literal::Number(value);
            let literal_node = ast::node::Node::new(test_span(0, 1), literal);
            let expr = ast::expression::Expr::Literal(literal_node);
            let expr_node = ast::node::Node::new(test_span(0, 1), expr);
            let value_node = ast::node::Node::new(
                test_span(0, 1),
                ast::parameter::ParameterValue::Simple(expr_node, None),
            );

            let parameter =
                ast::Parameter::new(label_node, ident_node, value_node, None, None, None, None);
            ast::node::Node::new(test_span(0, ident.len()), parameter)
        }

        /// Helper function to create a parameter with a specific span for testing duplicates
        pub fn create_simple_parameter_with_span(
            ident: &str,
            value: f64,
            start: usize,
            end: usize,
        ) -> ast::parameter::ParameterNode {
            let label = ast::naming::Label::new(format!("Parameter {}", ident));
            let label_node = ast::node::Node::new(test_span(start, start), label);
            let identifier = ast::naming::Identifier::new(ident.to_string());
            let ident_node =
                ast::node::Node::new(test_span(start, start + ident.len()), identifier);

            let literal = ast::expression::Literal::Number(value);
            let literal_node = ast::node::Node::new(test_span(start, start + 1), literal);
            let expr = ast::expression::Expr::Literal(literal_node);
            let expr_node = ast::node::Node::new(test_span(start, start + 1), expr);
            let value_node = ast::node::Node::new(
                test_span(start, start + 1),
                ast::parameter::ParameterValue::Simple(expr_node, None),
            );

            let parameter =
                ast::Parameter::new(label_node, ident_node, value_node, None, None, None, None);
            ast::node::Node::new(test_span(start, end), parameter)
        }

        /// Helper function to create a dependent parameter that references another parameter
        pub fn create_dependent_parameter(
            ident: &str,
            dependency: &str,
        ) -> ast::parameter::ParameterNode {
            let label = ast::naming::Label::new(format!("Parameter {}", ident));
            let label_node = ast::node::Node::new(test_span(0, 0), label);
            let identifier = ast::naming::Identifier::new(ident.to_string());
            let ident_node = ast::node::Node::new(test_span(0, ident.len()), identifier);

            let dep_identifier = ast::naming::Identifier::new(dependency.to_string());
            let dep_ident_node =
                ast::node::Node::new(test_span(0, dependency.len()), dep_identifier);
            let dep_variable = ast::expression::Variable::Identifier(dep_ident_node);
            let dep_var_node = ast::node::Node::new(test_span(0, dependency.len()), dep_variable);
            let dep_expr = ast::expression::Expr::Variable(dep_var_node);
            let dep_expr_node = ast::node::Node::new(test_span(0, dependency.len()), dep_expr);

            let literal = ast::expression::Literal::Number(1.0);
            let literal_node = ast::node::Node::new(test_span(0, 1), literal);
            let literal_expr = ast::expression::Expr::Literal(literal_node);
            let literal_expr_node = ast::node::Node::new(test_span(0, 1), literal_expr);

            let op = ast::expression::BinaryOp::Add;
            let op_node = ast::node::Node::new(test_span(0, 1), op);
            let binary_expr = ast::expression::Expr::BinaryOp {
                left: Box::new(dep_expr_node),
                op: op_node,
                right: Box::new(literal_expr_node),
            };
            let binary_expr_node = ast::node::Node::new(test_span(0, 3), binary_expr);
            let value_node = ast::node::Node::new(
                test_span(0, 3),
                ast::parameter::ParameterValue::Simple(binary_expr_node, None),
            );

            let parameter =
                ast::Parameter::new(label_node, ident_node, value_node, None, None, None, None);
            ast::node::Node::new(test_span(0, ident.len()), parameter)
        }

        /// Helper function to create mock model and submodel info for testing
        pub fn create_mock_info<'a>() -> (SubmodelInfo<'a>, ModelInfo<'a>) {
            let submodel_info = SubmodelInfo::new(HashMap::new(), HashSet::new());
            let model_info = ModelInfo::new(HashMap::new(), HashSet::new());
            (submodel_info, model_info)
        }

        /// Helper function to create empty builtin variables
        pub fn create_empty_builtin_ref() -> TestBuiltinRef {
            TestBuiltinRef::new()
        }
    }

    #[test]
    fn test_resolve_parameters_empty() {
        // create the parameters
        let parameters = vec![];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn test_resolve_parameters_simple() {
        // create the parameters
        let param_a = helper::create_simple_parameter("a", 10.0);
        let param_b = helper::create_simple_parameter("b", 20.0);
        let parameters = vec![&param_a, &param_b];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 2);

        let param_a = resolved_params.get(&Identifier::new("a")).unwrap();
        let param_b = resolved_params.get(&Identifier::new("b")).unwrap();

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().is_empty());
    }

    #[test]
    fn test_resolve_parameters_with_dependencies() {
        // create the parameters
        let param_a = helper::create_simple_parameter("a", 10.0);
        let param_b = helper::create_dependent_parameter("b", "a");
        let parameters = vec![&param_a, &param_b];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 2);

        let param_a = resolved_params.get(&Identifier::new("a")).unwrap();
        let param_b = resolved_params.get(&Identifier::new("b")).unwrap();

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().contains(&Identifier::new("a")));
    }

    #[test]
    fn test_resolve_parameters_circular_dependency() {
        // create the parameters with circular dependency
        let param_a = helper::create_dependent_parameter("a", "b");
        let param_b = helper::create_dependent_parameter("b", "a");
        let parameters = vec![&param_a, &param_b];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors
        assert!(!errors.is_empty());

        assert!(errors.contains_key(&Identifier::new("a")));
        assert!(errors.contains_key(&Identifier::new("b")));

        let a_errors = errors.get(&Identifier::new("a")).unwrap();
        let b_errors = errors.get(&Identifier::new("b")).unwrap();

        // check that both parameters have errors, one is a circular dependency
        // error and both have a "parameter had error" error
        assert_eq!(a_errors.len() + b_errors.len(), 3);

        // the order in which parameters are resolved is non-deterministic,
        // so we need to check for a circular dependency error in either
        let a_has_circular_dependency = a_errors
            .iter()
            .any(|e| matches!(e, ParameterResolutionError::CircularDependency { .. }));
        let b_has_circular_dependency = b_errors
            .iter()
            .any(|e| matches!(e, ParameterResolutionError::CircularDependency { .. }));
        assert!(a_has_circular_dependency || b_has_circular_dependency);

        assert!(a_errors.iter().any(|e| matches!(
            e,
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::ParameterHasError {
                    identifier,
                    reference_span: _,
                }
            )
            if identifier == &Identifier::new("b"),
        )));

        assert!(b_errors.iter().any(|e| matches!(
            e,
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::ParameterHasError {
                    identifier,
                    reference_span: _,
                }
            )
            if identifier == &Identifier::new("a"),
        )));

        // check the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn test_get_parameter_internal_dependencies_simple() {
        // create a simple parameter
        let parameter = helper::create_simple_parameter("a", 10.0);

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(|dep| dep.take_value())
            .collect();

        // check the dependencies
        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_variable() {
        // create a dependent parameter
        let parameter = helper::create_dependent_parameter("a", "b");

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(|dep| dep.take_value())
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&Identifier::new("b")));
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_limits() {
        // create a parameter with limits that reference other parameters
        let label = ast::naming::Label::new("Test Parameter".to_string());
        let label_node = ast::node::Node::new(helper::test_span(0, 0), label);
        let identifier = ast::naming::Identifier::new("test".to_string());
        let ident_node = ast::node::Node::new(helper::test_span(0, 4), identifier);

        // create the value expression
        let value_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 0, 1);
        let value_node = ast::node::Node::new(
            helper::test_span(0, 1),
            ast::parameter::ParameterValue::Simple(value_expr, None),
        );

        // create the limits with variable references
        let min_var = helper::create_identifier_variable("min_val");
        let min_expr = helper::create_variable_expr_node(min_var, 0, 7);
        let max_var = helper::create_identifier_variable("max_val");
        let max_expr = helper::create_variable_expr_node(max_var, 0, 7);
        let limits = ast::parameter::Limits::Continuous {
            min: min_expr,
            max: max_expr,
        };
        let limits_node = ast::node::Node::new(helper::test_span(0, 7), limits);

        let parameter = ast::Parameter::new(
            label_node,
            ident_node,
            value_node,
            Some(limits_node),
            None,
            None,
            None,
        );

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(|dep| dep.take_value())
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&Identifier::new("min_val")));
        assert!(dependencies.contains(&Identifier::new("max_val")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_literal() {
        // create a literal expression
        let expr = helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 0, 4);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_expr_internal_dependencies_variable() {
        // create a variable expression
        let variable = helper::create_identifier_variable("test_var");
        let expr = helper::create_variable_expr_node(variable, 0, 8);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert_eq!(result.len(), 1);
        assert!(result.contains(&Identifier::new("test_var")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_binary_op() {
        // create a binary operation with variables
        let left_var = helper::create_identifier_variable("a");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 1);
        let right_var = helper::create_identifier_variable("b");
        let right_expr = helper::create_variable_expr_node(right_var, 4, 5);
        let expr = helper::create_binary_op_expr_node(
            left_expr,
            ast::expression::BinaryOp::Add,
            right_expr,
            0,
            5,
        );

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains(&Identifier::new("a")));
        assert!(result.contains(&Identifier::new("b")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_function_call() {
        // create a function call with variable arguments
        let arg1_var = helper::create_identifier_variable("arg1");
        let arg1_expr = helper::create_variable_expr_node(arg1_var, 9, 13);
        let arg2_var = helper::create_identifier_variable("arg2");
        let arg2_expr = helper::create_variable_expr_node(arg2_var, 15, 19);
        let expr =
            helper::create_function_call_expr_node("test_func", vec![arg1_expr, arg2_expr], 0, 19);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains(&Identifier::new("arg1")));
        assert!(result.contains(&Identifier::new("arg2")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_accessor() {
        // create an accessor variable
        let parent_identifier = ast::naming::Identifier::new("parent".to_string());
        let parent_node = ast::node::Node::new(helper::test_span(0, 6), parent_identifier);
        let component_identifier = ast::naming::Identifier::new("component".to_string());
        let component_node = ast::node::Node::new(helper::test_span(0, 9), component_identifier);
        let component_variable = ast::expression::Variable::Identifier(component_node);
        let component_var_node = ast::node::Node::new(helper::test_span(0, 9), component_variable);
        let accessor_variable = ast::expression::Variable::Accessor {
            parent: parent_node,
            component: Box::new(component_var_node),
        };
        let accessor_var_node = ast::node::Node::new(helper::test_span(0, 9), accessor_variable);
        let expr = helper::create_variable_expr_node(accessor_var_node, 0, 9);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies - accessors don't count as internal dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_parameter_value_simple() {
        // create a simple parameter value
        let expr = helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 0, 4);
        let value = ast::parameter::ParameterValue::Simple(expr, None);
        let value_node = ast::node::Node::new(helper::test_span(0, 4), value);
        let (submodel_info, model_info) = helper::create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        // resolve the parameter value
        let result = resolve_parameter_value(
            &value_node,
            &helper::create_empty_builtin_ref(),
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        assert!(result.is_ok());
        let resolved_value = result.unwrap();
        assert!(matches!(resolved_value, ParameterValue::Simple(_, None)));
    }

    #[test]
    fn test_resolve_limits_none() {
        // create the context
        let (submodel_info, model_info) = helper::create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        // resolve the limits
        let result = resolve_limits(
            None,
            &helper::create_empty_builtin_ref(),
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        assert!(result.is_ok());
        let limits = result.unwrap();
        assert!(matches!(limits, Limits::Default));
    }

    #[test]
    fn test_resolve_limits_continuous() {
        // create continuous limits with literal values
        let min_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(0.0), 0, 1);
        let max_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(100.0), 0, 3);
        let limits = ast::parameter::Limits::Continuous {
            min: min_expr,
            max: max_expr,
        };
        let limits_node = ast::node::Node::new(helper::test_span(0, 3), limits);
        let (submodel_info, model_info) = helper::create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        // resolve the limits
        let result = resolve_limits(
            Some(&limits_node),
            &helper::create_empty_builtin_ref(),
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        assert!(result.is_ok());
        let resolved_limits = result.unwrap();
        assert!(matches!(resolved_limits, Limits::Continuous { .. }));
    }

    #[test]
    fn test_resolve_limits_discrete() {
        // create discrete limits with literal values
        let value1 = helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 0, 1);
        let value2 = helper::create_literal_expr_node(ast::expression::Literal::Number(2.0), 0, 1);
        let value3 = helper::create_literal_expr_node(ast::expression::Literal::Number(3.0), 0, 1);
        let limits = ast::parameter::Limits::Discrete {
            values: vec![value1, value2, value3],
        };
        let limits_node = ast::node::Node::new(helper::test_span(0, 3), limits);
        let (submodel_info, model_info) = helper::create_mock_info();
        let defined_parameters_info = ParameterInfo::new(HashMap::new(), HashSet::new());

        // resolve the limits
        let result = resolve_limits(
            Some(&limits_node),
            &helper::create_empty_builtin_ref(),
            &defined_parameters_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        assert!(result.is_ok());
        let resolved_limits = result.unwrap();
        assert!(matches!(resolved_limits, Limits::Discrete { .. }));
    }

    #[test]
    fn test_resolve_parameters_duplicate_parameters() {
        // create the parameters with duplicate identifiers
        let param_a1 = helper::create_simple_parameter_with_span("a", 10.0, 0, 10);
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_a2 = helper::create_simple_parameter_with_span("a", 20.0, 15, 25);
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let parameters = vec![&param_a1, &param_a2];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors - should have one duplicate parameter error
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&Identifier::new("a")));

        let a_errors = errors.get(&Identifier::new("a")).unwrap();
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];
        assert_eq!(
            duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: Identifier::new("a"),
                original_span: param_a1_span.clone(),
                duplicate_span: param_a2_span.clone()
            }
        );

        // check the resolved parameters - should have one parameter, the
        //     parameter that is present is left unspecified
        assert_eq!(resolved_params.len(), 1);
        assert!(resolved_params.contains_key(&Identifier::new("a")));
    }

    #[test]
    fn test_resolve_parameters_multiple_duplicate_parameters() {
        // create the parameters with multiple duplicates
        let param_a1 = helper::create_simple_parameter_with_span("a", 10.0, 0, 10);
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_b1 = helper::create_simple_parameter_with_span("b", 20.0, 15, 25);
        let param_b1_span = get_span_from_ast_span(param_b1.ident().node_span());
        let param_a2 = helper::create_simple_parameter_with_span("a", 30.0, 30, 40);
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let param_b2 = helper::create_simple_parameter_with_span("b", 40.0, 45, 55);
        let param_b2_span = get_span_from_ast_span(param_b2.ident().node_span());
        let parameters = vec![&param_a1, &param_b1, &param_a2, &param_b2];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors - should have two duplicate parameter errors
        assert_eq!(errors.len(), 2);
        assert!(errors.contains_key(&Identifier::new("a")));
        assert!(errors.contains_key(&Identifier::new("b")));

        let a_errors = errors.get(&Identifier::new("a")).unwrap();
        let b_errors = errors.get(&Identifier::new("b")).unwrap();
        assert_eq!(a_errors.len(), 1);
        assert_eq!(b_errors.len(), 1);

        // check that both errors are duplicate parameter errors
        let a_duplicate_error = &a_errors[0];
        assert_eq!(
            a_duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: Identifier::new("a"),
                original_span: param_a1_span.clone(),
                duplicate_span: param_a2_span.clone()
            }
        );

        let b_duplicate_error = &b_errors[0];
        assert_eq!(
            b_duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: Identifier::new("b"),
                original_span: param_b1_span.clone(),
                duplicate_span: param_b2_span.clone()
            }
        );

        // check the resolved parameters - should have two parameters, the
        //     parameters that are present are left unspecified
        assert_eq!(resolved_params.len(), 2);
        assert!(resolved_params.contains_key(&Identifier::new("a")));
        assert!(resolved_params.contains_key(&Identifier::new("b")));
    }

    #[test]
    fn test_resolve_parameters_duplicate_parameters_with_valid_parameters() {
        // create the parameters with duplicates and valid parameters
        let param_a1 = helper::create_simple_parameter_with_span("a", 10.0, 0, 10);
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_b = helper::create_simple_parameter("b", 20.0);
        let param_a2 = helper::create_simple_parameter_with_span("a", 30.0, 15, 25);
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let param_c = helper::create_simple_parameter("c", 40.0);
        let parameters = vec![&param_a1, &param_b, &param_a2, &param_c];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors - should have one duplicate parameter error
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&Identifier::new("a")));

        let a_errors = errors.get(&Identifier::new("a")).unwrap();
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];
        assert_eq!(
            duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: Identifier::new("a"),
                original_span: param_a1_span.clone(),
                duplicate_span: param_a2_span.clone()
            }
        );

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 3);
        assert!(resolved_params.contains_key(&Identifier::new("a")));
        assert!(resolved_params.contains_key(&Identifier::new("b")));
        assert!(resolved_params.contains_key(&Identifier::new("c")));
    }

    #[test]
    fn test_resolve_parameters_duplicate_parameters_with_dependencies() {
        // create the parameters with duplicates and dependencies
        let param_a1 = helper::create_simple_parameter_with_span("a", 10.0, 0, 10);
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_b = helper::create_dependent_parameter("b", "a");
        let param_a2 = helper::create_simple_parameter_with_span("a", 20.0, 15, 25);
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let param_c = helper::create_dependent_parameter("c", "b");
        let parameters = vec![&param_a1, &param_b, &param_a2, &param_c];

        // create the builtin variables
        let builtin_ref = helper::create_empty_builtin_ref();

        // create the submodel and model info
        let (submodel_info, model_info) = helper::create_mock_info();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &submodel_info, &model_info);

        // check the errors - should have one duplicate parameter error
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&Identifier::new("a")));

        let a_errors = errors.get(&Identifier::new("a")).unwrap();
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];
        assert_eq!(
            duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: Identifier::new("a"),
                original_span: param_a1_span.clone(),
                duplicate_span: param_a2_span.clone()
            }
        );

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 3);
        assert!(resolved_params.contains_key(&Identifier::new("a")));
        assert!(resolved_params.contains_key(&Identifier::new("b")));
        assert!(resolved_params.contains_key(&Identifier::new("c")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_comparison_op() {
        // create a comparison expression with variables
        let left_var = helper::create_identifier_variable("a");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 1);
        let right_var = helper::create_identifier_variable("b");
        let right_expr = helper::create_variable_expr_node(right_var, 4, 5);

        let op = ast::expression::ComparisonOp::LessThan;
        let op_node = ast::node::Node::new(helper::test_span(2, 3), op);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op_node,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            rest_chained: vec![],
        };
        let expr_node = ast::node::Node::new(helper::test_span(0, 5), expr);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains(&Identifier::new("a")));
        assert!(result.contains(&Identifier::new("b")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_chained_comparison_op() {
        // create a chained comparison expression: a < b < c
        let left_var = helper::create_identifier_variable("a");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 1);
        let middle_var = helper::create_identifier_variable("b");
        let middle_expr = helper::create_variable_expr_node(middle_var, 4, 5);
        let right_var = helper::create_identifier_variable("c");
        let right_expr = helper::create_variable_expr_node(right_var, 8, 9);

        let op1 = ast::expression::ComparisonOp::LessThan;
        let op1_node = ast::node::Node::new(helper::test_span(2, 3), op1);
        let op2 = ast::expression::ComparisonOp::LessThan;
        let op2_node = ast::node::Node::new(helper::test_span(6, 7), op2);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op1_node,
            left: Box::new(left_expr),
            right: Box::new(middle_expr),
            rest_chained: vec![(op2_node, right_expr)],
        };
        let expr_node = ast::node::Node::new(helper::test_span(0, 9), expr);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert_eq!(result.len(), 3);
        assert!(result.contains(&Identifier::new("a")));
        assert!(result.contains(&Identifier::new("b")));
        assert!(result.contains(&Identifier::new("c")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_comparison_op_with_literals() {
        // create a comparison expression with one variable and one literal
        let left_var = helper::create_identifier_variable("x");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 1);
        let right_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(10.0), 4, 6);

        let op = ast::expression::ComparisonOp::GreaterThan;
        let op_node = ast::node::Node::new(helper::test_span(2, 3), op);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op_node,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            rest_chained: vec![],
        };
        let expr_node = ast::node::Node::new(helper::test_span(0, 6), expr);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies - should only contain the variable
        assert_eq!(result.len(), 1);
        assert!(result.contains(&Identifier::new("x")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_comparison_op_with_complex_expressions() {
        // create a comparison expression with complex expressions on both sides
        // (a + b) < (c * d)
        let a_var = helper::create_identifier_variable("a");
        let a_expr = helper::create_variable_expr_node(a_var, 1, 2);
        let b_var = helper::create_identifier_variable("b");
        let b_expr = helper::create_variable_expr_node(b_var, 5, 6);
        let c_var = helper::create_identifier_variable("c");
        let c_expr = helper::create_variable_expr_node(c_var, 11, 12);
        let d_var = helper::create_identifier_variable("d");
        let d_expr = helper::create_variable_expr_node(d_var, 15, 16);

        // Create (a + b)
        let add_op = ast::expression::BinaryOp::Add;
        let add_op_node = ast::node::Node::new(helper::test_span(3, 4), add_op);
        let left_binary = ast::expression::Expr::BinaryOp {
            left: Box::new(a_expr),
            op: add_op_node,
            right: Box::new(b_expr),
        };
        let left_expr = ast::node::Node::new(helper::test_span(0, 7), left_binary);

        // Create (c * d)
        let mul_op = ast::expression::BinaryOp::Mul;
        let mul_op_node = ast::node::Node::new(helper::test_span(13, 14), mul_op);
        let right_binary = ast::expression::Expr::BinaryOp {
            left: Box::new(c_expr),
            op: mul_op_node,
            right: Box::new(d_expr),
        };
        let right_expr = ast::node::Node::new(helper::test_span(10, 17), right_binary);

        // Create the comparison
        let comp_op = ast::expression::ComparisonOp::LessThan;
        let comp_op_node = ast::node::Node::new(helper::test_span(8, 9), comp_op);

        let expr = ast::expression::Expr::ComparisonOp {
            op: comp_op_node,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            rest_chained: vec![],
        };
        let expr_node = ast::node::Node::new(helper::test_span(0, 17), expr);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(|dep| dep.take_value()).collect();

        // check the dependencies
        assert_eq!(result.len(), 4);
        assert!(result.contains(&Identifier::new("a")));
        assert!(result.contains(&Identifier::new("b")));
        assert!(result.contains(&Identifier::new("c")));
        assert!(result.contains(&Identifier::new("d")));
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_comparison_conditions() {
        // create a parameter with a piecewise value that uses comparison conditions
        let label = ast::naming::Label::new("Test Parameter".to_string());
        let label_node = ast::node::Node::new(helper::test_span(0, 0), label);
        let identifier = ast::naming::Identifier::new("test".to_string());
        let ident_node = ast::node::Node::new(helper::test_span(0, 4), identifier);

        // create the value expression
        let value_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 0, 1);

        // create a comparison condition: x < threshold
        let x_var = helper::create_identifier_variable("x");
        let x_expr = helper::create_variable_expr_node(x_var, 0, 1);
        let threshold_var = helper::create_identifier_variable("threshold");
        let threshold_expr = helper::create_variable_expr_node(threshold_var, 4, 13);

        let comp_op = ast::expression::ComparisonOp::LessThan;
        let comp_op_node = ast::node::Node::new(helper::test_span(2, 3), comp_op);

        let condition_expr = ast::expression::Expr::ComparisonOp {
            op: comp_op_node,
            left: Box::new(x_expr),
            right: Box::new(threshold_expr),
            rest_chained: vec![],
        };
        let condition_expr_node = ast::node::Node::new(helper::test_span(0, 13), condition_expr);

        // create the piecewise part
        let piecewise_part = ast::parameter::PiecewisePart::new(value_expr, condition_expr_node);
        let piecewise_part_node = ast::node::Node::new(helper::test_span(0, 13), piecewise_part);

        let value_node = ast::node::Node::new(
            helper::test_span(0, 13),
            ast::parameter::ParameterValue::Piecewise(vec![piecewise_part_node], None),
        );

        let parameter =
            ast::Parameter::new(label_node, ident_node, value_node, None, None, None, None);

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(|dep| dep.take_value())
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&Identifier::new("x")));
        assert!(dependencies.contains(&Identifier::new("threshold")));
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_chained_comparison_limits() {
        // create a parameter with limits that use chained comparisons
        let label = ast::naming::Label::new("Test Parameter".to_string());
        let label_node = ast::node::Node::new(helper::test_span(0, 0), label);
        let identifier = ast::naming::Identifier::new("test".to_string());
        let ident_node = ast::node::Node::new(helper::test_span(0, 4), identifier);

        // create the value expression
        let value_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 0, 1);
        let value_node = ast::node::Node::new(
            helper::test_span(0, 1),
            ast::parameter::ParameterValue::Simple(value_expr, None),
        );

        // create limits with chained comparison: min_val < x < max_val
        let min_var = helper::create_identifier_variable("min_val");
        let min_expr = helper::create_variable_expr_node(min_var, 0, 7);
        let x_var = helper::create_identifier_variable("x");
        let x_expr = helper::create_variable_expr_node(x_var, 4, 5);
        let max_var = helper::create_identifier_variable("max_val");
        let max_expr = helper::create_variable_expr_node(max_var, 8, 15);

        let op1 = ast::expression::ComparisonOp::LessThan;
        let op1_node = ast::node::Node::new(helper::test_span(2, 3), op1);
        let op2 = ast::expression::ComparisonOp::LessThan;
        let op2_node = ast::node::Node::new(helper::test_span(6, 7), op2);

        let limits_expr = ast::expression::Expr::ComparisonOp {
            op: op1_node,
            left: Box::new(min_expr),
            right: Box::new(x_expr),
            rest_chained: vec![(op2_node, max_expr)],
        };
        let limits_expr_node = ast::node::Node::new(helper::test_span(0, 15), limits_expr);

        let limits = ast::parameter::Limits::Continuous {
            min: limits_expr_node.clone(),
            max: limits_expr_node,
        };
        let limits_node = ast::node::Node::new(helper::test_span(0, 15), limits);

        let parameter = ast::Parameter::new(
            label_node,
            ident_node,
            value_node,
            Some(limits_node),
            None,
            None,
            None,
        );

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(|dep| dep.take_value())
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains(&Identifier::new("min_val")));
        assert!(dependencies.contains(&Identifier::new("x")));
        assert!(dependencies.contains(&Identifier::new("max_val")));
    }
}
