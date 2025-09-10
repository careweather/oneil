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
use oneil_ir::{self as ir, IrSpan};

use crate::{
    BuiltinRef,
    error::{self, ParameterResolutionError},
    loader::resolver::{expr::resolve_expr, trace_level::resolve_trace_level, unit::resolve_unit},
    util::{
        Stack,
        builder::ParameterBuilder,
        context::{ParameterContext, ReferenceContext},
        get_span_from_ast_span,
    },
};

pub type ParameterErrorMap = HashMap<ir::Identifier, Vec<ParameterResolutionError>>;

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
    parameters: Vec<&ast::ParameterNode>,
    builtin_ref: &impl BuiltinRef,
    context: &ReferenceContext<'_, '_>,
) -> (ir::ParameterCollection, ParameterErrorMap) {
    let mut parameter_builder = ParameterBuilder::new();

    let mut parameter_map = HashMap::new();

    for parameter in parameters {
        let ident = ir::Identifier::new(parameter.ident().as_str());
        let ident_span = get_span_from_ast_span(parameter.ident().node_span());

        let maybe_original_parameter = parameter_map.get(&ident);
        if let Some((original_ident_span, _)) = maybe_original_parameter {
            parameter_builder.add_parameter_error(
                ident.clone(),
                ParameterResolutionError::duplicate_parameter(
                    ident,
                    *original_ident_span,
                    ident_span,
                ),
            );
        } else {
            parameter_map.insert(ident, (ident_span, parameter));
        }
    }

    // split the parameter map into two maps, one for the spans and one for the
    // AST nodes
    let (parameter_span_map, parameter_ast_map): (
        HashMap<_, IrSpan>,
        HashMap<_, &ast::ParameterNode>,
    ) = parameter_map
        .into_iter()
        .map(|(ident, (span, ast))| ((ident.clone(), span), (ident, ast)))
        .unzip();

    // note that an 'internal dependency' is a dependency on a parameter
    // that is defined within the current model
    let dependencies = get_all_parameter_internal_dependencies(&parameter_ast_map);

    for (parameter_identifier, parameter_span) in parameter_span_map {
        let mut parameter_stack = Stack::new();

        let parameter_identifier_with_span =
            ir::WithSpan::new(parameter_identifier.clone(), parameter_span);

        parameter_builder = resolve_parameter(
            parameter_identifier_with_span,
            &parameter_ast_map,
            &dependencies,
            &mut parameter_stack,
            builtin_ref,
            context,
            parameter_builder,
        );
    }

    parameter_builder.into_parameter_collection_and_errors()
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
    parameter_map: &'a HashMap<ir::Identifier, &'a ast::ParameterNode>,
) -> HashMap<&'a ir::Identifier, HashSet<ir::WithSpan<ir::Identifier>>> {
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
fn get_parameter_internal_dependencies(
    parameter: &ast::Parameter,
) -> HashSet<ir::WithSpan<ir::Identifier>> {
    let dependencies = HashSet::new();

    let limits = parameter.limits().map(ast::Node::node_value);
    let dependencies = match limits {
        Some(ast::Limits::Continuous { min, max }) => {
            let dependencies = get_expr_internal_dependencies(min, dependencies);
            get_expr_internal_dependencies(max, dependencies)
        }
        Some(ast::Limits::Discrete { values }) => {
            values.iter().fold(dependencies, |dependencies, expr| {
                get_expr_internal_dependencies(expr, dependencies)
            })
        }
        None => dependencies,
    };

    match &parameter.value().node_value() {
        ast::ParameterValue::Simple(expr, _) => get_expr_internal_dependencies(expr, dependencies),
        ast::ParameterValue::Piecewise(piecewise, _) => {
            piecewise.iter().fold(dependencies, |dependencies, part| {
                let dependencies = get_expr_internal_dependencies(part.if_expr(), dependencies);
                get_expr_internal_dependencies(part.expr(), dependencies)
            })
        }
    }
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
    mut dependencies: HashSet<ir::WithSpan<ir::Identifier>>,
) -> HashSet<ir::WithSpan<ir::Identifier>> {
    match expr {
        ast::Expr::BinaryOp { op: _, left, right } => {
            let dependencies = get_expr_internal_dependencies(left, dependencies);
            get_expr_internal_dependencies(right, dependencies)
        }

        ast::Expr::UnaryOp { op: _, expr } | ast::Expr::Parenthesized { expr } => {
            get_expr_internal_dependencies(expr, dependencies)
        }

        ast::Expr::FunctionCall { name: _, args } => {
            args.iter().fold(dependencies, |dependencies, arg| {
                get_expr_internal_dependencies(arg, dependencies)
            })
        }

        ast::Expr::Variable(variable) => match variable.node_value() {
            ast::Variable::Identifier(identifier) => {
                let identifier_span = get_span_from_ast_span(identifier.node_span());
                let identifier = ir::Identifier::new(identifier.as_str());
                dependencies.insert(ir::WithSpan::new(identifier, identifier_span));
                dependencies
            }

            ast::Variable::ModelParameter {
                reference_model: _,
                parameter: _,
            } => {
                // an accessor implies that the dependency is on a parameter
                // outside of the current model, so it doesn't count as an
                // internal dependency
                dependencies
            }
        },
        ast::Expr::Literal(_) => dependencies,
        ast::Expr::ComparisonOp {
            op: _,
            left,
            right,
            rest_chained,
        } => {
            let dependencies = get_expr_internal_dependencies(left, dependencies);
            let dependencies = get_expr_internal_dependencies(right, dependencies);
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
    parameter_identifier: ir::WithSpan<ir::Identifier>,
    parameter_ast_map: &HashMap<ir::Identifier, &ast::ParameterNode>,
    dependencies: &HashMap<&ir::Identifier, HashSet<ir::WithSpan<ir::Identifier>>>,
    parameter_stack: &mut Stack<ir::Identifier>,
    builtin_ref: &impl BuiltinRef,
    context: &ReferenceContext<'_, '_>,
    mut parameter_builder: ParameterBuilder,
) -> ParameterBuilder {
    let parameter_identifier_span = parameter_identifier.span();
    let parameter_identifier = parameter_identifier.take_value();

    // check that the parameter exists
    if !parameter_ast_map.contains_key(&parameter_identifier) {
        // This is technically a resolution error. However, this error will
        // be caught later when the variable is resolved. In order to avoid
        // duplicate errors, we return Ok(()) and let the variable resolution
        // handle the "not found" error
        //
        // This also accounts for the fact that the parameter may be a builtin
        return parameter_builder;
    }

    assert!(
        dependencies.contains_key(&parameter_identifier),
        "parameter dependencies for '{parameter_identifier:?}' not found",
    );

    // check for circular dependencies
    if let Some(circular_dependency) =
        parameter_stack.find_circular_dependency(&parameter_identifier)
    {
        let reference_span = parameter_identifier_span;
        parameter_builder.add_parameter_error(
            parameter_identifier,
            ParameterResolutionError::circular_dependency(circular_dependency, reference_span),
        );
        return parameter_builder;
    }

    // check if the parameter has already been visited
    if parameter_builder.has_visited(&parameter_identifier) {
        return parameter_builder;
    }
    parameter_builder.mark_as_visited(parameter_identifier.clone());

    // resolve the parameter dependencies
    let parameter_dependencies = dependencies
        .get(&parameter_identifier)
        .expect("parameter dependencies should exist");

    // add the parameter to the stack
    parameter_stack.push(parameter_identifier.clone());

    // resolve the parameter dependencies
    for dependency in parameter_dependencies {
        parameter_builder = resolve_parameter(
            dependency.clone(),
            parameter_ast_map,
            dependencies,
            parameter_stack,
            builtin_ref,
            context,
            parameter_builder,
        );
    }

    // remove the parameter from the stack
    parameter_stack.pop();

    // resolve the parameter
    let parameter = parameter_ast_map
        .get(&parameter_identifier)
        .expect("parameter should exist");

    let parameter_context = ParameterContext::new(
        parameter_builder.get_parameters(),
        parameter_builder.get_parameter_errors(),
    );

    let ident = parameter_identifier.clone();

    let value =
        resolve_parameter_value(parameter.value(), builtin_ref, context, &parameter_context);

    let limits = resolve_limits(parameter.limits(), builtin_ref, context, &parameter_context);

    let is_performance = parameter.performance_marker().is_some();

    let trace_level = resolve_trace_level(parameter.trace_level());

    match error::combine_errors(value, limits) {
        Ok((value, limits)) => {
            // build the parameter
            let ident_span = get_span_from_ast_span(parameter.ident().node_span());
            let ident_with_span = ir::WithSpan::new(ident, ident_span);

            let parameter_dependencies = parameter_dependencies
                .iter()
                .map(|dependency| dependency.value().clone())
                .collect();

            let parameter = ir::Parameter::new(
                parameter_dependencies,
                ident_with_span,
                value,
                limits,
                is_performance,
                trace_level,
            );

            // add the parameter to the parameter builder
            parameter_builder.add_parameter(parameter_identifier, parameter);
        }
        Err(errors) => {
            // add the errors to the parameter builder
            for error in errors {
                parameter_builder.add_parameter_error(parameter_identifier.clone(), error);
            }
        }
    }

    parameter_builder
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
    value: &ast::ParameterValue,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ParameterValue, Vec<ParameterResolutionError>> {
    match value {
        ast::ParameterValue::Simple(expr, unit) => {
            let expr = resolve_expr(expr, builtin_ref, reference_context, parameter_context)
                .map_err(error::convert_errors)?;

            let unit = unit.as_ref().map(resolve_unit);

            Ok(ir::ParameterValue::simple(expr, unit))
        }

        ast::ParameterValue::Piecewise(piecewise, unit) => {
            let exprs = piecewise.iter().map(|part| {
                let expr = resolve_expr(
                    part.expr(),
                    builtin_ref,
                    reference_context,
                    parameter_context,
                )
                .map_err(error::convert_errors);

                let if_expr = resolve_expr(
                    part.if_expr(),
                    builtin_ref,
                    reference_context,
                    parameter_context,
                )
                .map_err(error::convert_errors);

                let (expr, if_expr) = error::combine_errors(expr, if_expr)?;

                Ok(ir::PiecewiseExpr::new(expr, if_expr))
            });

            let unit = unit.as_ref().map(resolve_unit);

            let exprs = error::combine_error_list(exprs)?;

            Ok(ir::ParameterValue::piecewise(exprs, unit))
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
    limits: Option<&ast::LimitsNode>,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Limits, Vec<ParameterResolutionError>> {
    match limits.map(ast::Node::node_value) {
        Some(ast::Limits::Continuous { min, max }) => {
            let min = resolve_expr(min, builtin_ref, reference_context, parameter_context)
                .map_err(error::convert_errors);

            let max = resolve_expr(max, builtin_ref, reference_context, parameter_context)
                .map_err(error::convert_errors);

            let (min, max) = error::combine_errors(min, max)?;

            Ok(ir::Limits::continuous(min, max))
        }
        Some(ast::Limits::Discrete { values }) => {
            let values = values.iter().map(|value| {
                resolve_expr(value, builtin_ref, reference_context, parameter_context)
                    .map_err(error::convert_errors)
            });

            let values = error::combine_error_list(values)?;

            Ok(ir::Limits::discrete(values))
        }
        None => Ok(ir::Limits::default()),
    }
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
    use oneil_ast as ast;
    use oneil_ir as ir;

    #[test]
    fn test_resolve_parameters_empty() {
        // create the parameters
        let parameters = vec![];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors
        assert!(errors.is_empty());

        // check the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn test_resolve_parameters_simple() {
        // create the parameters
        let param_a = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();

        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_number_value(20.0)
            .build();

        let parameters = vec![&param_a, &param_b];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors
        assert!(errors.is_empty());

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 2);

        let param_a = resolved_params
            .get(&ir::Identifier::new("a"))
            .expect("param a should exist");
        let param_b = resolved_params
            .get(&ir::Identifier::new("b"))
            .expect("param b should exist");

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().is_empty());
    }

    #[test]
    fn test_resolve_parameters_with_dependencies() {
        // create the parameters
        let param_a = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_dependent_parameter_values(["a"])
            .build();
        let parameters = vec![&param_a, &param_b];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors
        assert!(errors.is_empty());

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 2);

        let param_a = resolved_params
            .get(&ir::Identifier::new("a"))
            .expect("param a should exist");
        let param_b = resolved_params
            .get(&ir::Identifier::new("b"))
            .expect("param b should exist");

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().contains(&ir::Identifier::new("a")));
    }

    #[test]
    fn test_resolve_parameters_circular_dependency() {
        // create the parameters with circular dependency
        let param_a = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_dependent_parameter_values(["b"])
            .build();
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_dependent_parameter_values(["a"])
            .build();
        let parameters = vec![&param_a, &param_b];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors
        assert!(!errors.is_empty());

        assert!(errors.contains_key(&ir::Identifier::new("a")));
        assert!(errors.contains_key(&ir::Identifier::new("b")));

        let a_errors = errors
            .get(&ir::Identifier::new("a"))
            .expect("a errors should exist");
        let b_errors = errors
            .get(&ir::Identifier::new("b"))
            .expect("b errors should exist");

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
            if identifier == &ir::Identifier::new("b"),
        )));

        assert!(b_errors.iter().any(|e| matches!(
            e,
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::ParameterHasError {
                    identifier,
                    reference_span: _,
                }
            )
            if identifier == &ir::Identifier::new("a"),
        )));

        // check the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn test_get_parameter_internal_dependencies_simple() {
        // create a simple parameter
        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(ir::WithSpan::take_value)
            .collect();

        // check the dependencies
        assert!(dependencies.is_empty());
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_variable() {
        // create a dependent parameter
        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_dependent_parameter_values(["b"])
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(ir::WithSpan::take_value)
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&ir::Identifier::new("b")));
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_limits() {
        // create the parameter
        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("test")
            .with_number_value(5.0)
            .with_continuous_limit_vars("min_val", "max_val")
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(ir::WithSpan::take_value)
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&ir::Identifier::new("min_val")));
        assert!(dependencies.contains(&ir::Identifier::new("max_val")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_literal() {
        // create a literal expression
        let expr = test_ast::literal_number_expr_node(42.0);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_expr_internal_dependencies_variable() {
        // create a variable expression
        let variable = test_ast::identifier_variable_node("test_var");
        let expr = test_ast::variable_expr_node(variable);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ir::Identifier::new("test_var")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_binary_op() {
        // create a binary operation with variables
        let left_var = test_ast::identifier_variable_node("a");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_var = test_ast::identifier_variable_node("b");
        let right_expr = test_ast::variable_expr_node(right_var);
        let expr = test_ast::binary_op_expr_node(
            test_ast::binary_op_node(ast::BinaryOp::Add),
            left_expr,
            right_expr,
        );

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains(&ir::Identifier::new("a")));
        assert!(result.contains(&ir::Identifier::new("b")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_function_call() {
        // create a function call with variable arguments
        let arg1_var = test_ast::identifier_variable_node("arg1");
        let arg1_expr = test_ast::variable_expr_node(arg1_var);
        let arg2_var = test_ast::identifier_variable_node("arg2");
        let arg2_expr = test_ast::variable_expr_node(arg2_var);
        let expr = test_ast::function_call_expr_node(
            test_ast::identifier_node("test_func"),
            vec![arg1_expr, arg2_expr],
        );

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains(&ir::Identifier::new("arg1")));
        assert!(result.contains(&ir::Identifier::new("arg2")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_accessor() {
        // create an accessor variable
        let variable = test_ast::model_parameter_variable_node("reference_model", "parameter");
        let expr = test_ast::variable_expr_node(variable);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies - accessors don't count as internal dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_parameter_value_simple() {
        // create a simple parameter value
        let expr = test_ast::literal_number_expr_node(42.0);
        let value_node = test_ast::simple_parameter_value_node(expr);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameter value
        let result = resolve_parameter_value(
            &value_node,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        assert!(matches!(result, Ok(ir::ParameterValue::Simple(_, None))));
    }

    #[test]
    fn test_resolve_limits_none() {
        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the limits
        let result = resolve_limits(None, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        assert_eq!(result, Ok(ir::Limits::default()));
    }

    #[test]
    fn test_resolve_limits_continuous() {
        // create continuous limits with literal values
        let limits_node = test_ast::continuous_limits_node(0.0, 100.0);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the limits
        let result = resolve_limits(
            Some(&limits_node),
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        assert!(matches!(result, Ok(ir::Limits::Continuous { .. })));
    }

    #[test]
    fn test_resolve_limits_discrete() {
        // create discrete limits with literal values
        let limits_node = test_ast::discrete_limits_node([1.0, 2.0, 3.0]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the limits
        let result = resolve_limits(
            Some(&limits_node),
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        assert!(matches!(result, Ok(ir::Limits::Discrete { .. })));
    }

    #[test]
    fn test_resolve_parameters_duplicate_parameters() {
        // create the parameters with duplicate identifiers
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(20.0)
            .build();
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let parameters = vec![&param_a1, &param_a2];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors - should have one duplicate parameter error
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&ir::Identifier::new("a")));

        let a_errors = errors
            .get(&ir::Identifier::new("a"))
            .expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];
        assert_eq!(
            duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: ir::Identifier::new("a"),
                original_span: param_a1_span,
                duplicate_span: param_a2_span
            }
        );

        // check the resolved parameters - should have one parameter, the
        //     parameter that is present is left unspecified
        assert_eq!(resolved_params.len(), 1);
        assert!(resolved_params.contains_key(&ir::Identifier::new("a")));
    }

    #[test]
    fn test_resolve_parameters_multiple_duplicate_parameters() {
        // create the parameters with multiple duplicates
        let param_foo1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("foo")
            .with_number_value(10.0)
            .build();
        let param_foo1_span = get_span_from_ast_span(param_foo1.ident().node_span());
        let param_bar1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("bar")
            .with_number_value(20.0)
            .build();
        let param_bar1_span = get_span_from_ast_span(param_bar1.ident().node_span());
        let param_foo2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("foo")
            .with_number_value(30.0)
            .build();
        let param_foo2_span = get_span_from_ast_span(param_foo2.ident().node_span());
        let param_bar2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("bar")
            .with_number_value(40.0)
            .build();
        let param_bar2_span = get_span_from_ast_span(param_bar2.ident().node_span());
        let parameters = vec![&param_foo1, &param_bar1, &param_foo2, &param_bar2];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors - should have two duplicate parameter errors
        assert_eq!(errors.len(), 2);
        assert!(errors.contains_key(&ir::Identifier::new("foo")));
        assert!(errors.contains_key(&ir::Identifier::new("bar")));

        let foo_errors = errors
            .get(&ir::Identifier::new("foo"))
            .expect("foo errors should exist");
        let bar_errors = errors
            .get(&ir::Identifier::new("bar"))
            .expect("bar errors should exist");
        assert_eq!(foo_errors.len(), 1);
        assert_eq!(bar_errors.len(), 1);

        // check that both errors are duplicate parameter errors
        let foo_duplicate_error = &foo_errors[0];
        assert_eq!(
            foo_duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: ir::Identifier::new("foo"),
                original_span: param_foo1_span,
                duplicate_span: param_foo2_span
            }
        );

        let bar_duplicate_error = &bar_errors[0];
        assert_eq!(
            bar_duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: ir::Identifier::new("bar"),
                original_span: param_bar1_span,
                duplicate_span: param_bar2_span
            }
        );

        // check the resolved parameters - should have two parameters, the
        //     parameters that are present are left unspecified
        assert_eq!(resolved_params.len(), 2);
        assert!(resolved_params.contains_key(&ir::Identifier::new("foo")));
        assert!(resolved_params.contains_key(&ir::Identifier::new("bar")));
    }

    #[test]
    fn test_resolve_parameters_duplicate_parameters_with_valid_parameters() {
        // create the parameters with duplicates and valid parameters
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_number_value(20.0)
            .build();
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(30.0)
            .build();
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let param_c = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("c")
            .with_number_value(40.0)
            .build();
        let parameters = vec![&param_a1, &param_b, &param_a2, &param_c];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors - should have one duplicate parameter error
        assert_eq!(errors.len(), 1);
        assert!(errors.contains_key(&ir::Identifier::new("a")));

        let a_errors = errors
            .get(&ir::Identifier::new("a"))
            .expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];
        assert_eq!(
            duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: ir::Identifier::new("a"),
                original_span: param_a1_span,
                duplicate_span: param_a2_span
            }
        );

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 3);
        assert!(resolved_params.contains_key(&ir::Identifier::new("a")));
        assert!(resolved_params.contains_key(&ir::Identifier::new("b")));
        assert!(resolved_params.contains_key(&ir::Identifier::new("c")));
    }

    #[test]
    fn test_resolve_parameters_duplicate_parameters_with_dependencies() {
        // create the parameters with duplicates and dependencies
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_a1_span = get_span_from_ast_span(param_a1.ident().node_span());
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_dependent_parameter_values(["a"])
            .build();
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(20.0)
            .build();
        let param_a2_span = get_span_from_ast_span(param_a2.ident().node_span());
        let param_c = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("c")
            .with_dependent_parameter_values(["b"])
            .build();
        let parameters = vec![&param_a1, &param_b, &param_a2, &param_c];

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the parameters
        let (resolved_params, errors) =
            resolve_parameters(parameters, &builtin_ref, &reference_context);

        // check the errors
        assert_eq!(errors.len(), 3);

        // check the a error for "duplicate parameter"
        assert!(errors.contains_key(&ir::Identifier::new("a")));
        let a_errors = errors
            .get(&ir::Identifier::new("a"))
            .expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        let duplicate_error = &a_errors[0];
        assert_eq!(
            duplicate_error,
            &ParameterResolutionError::DuplicateParameter {
                identifier: ir::Identifier::new("a"),
                original_span: param_a1_span,
                duplicate_span: param_a2_span
            }
        );

        // check the b error for "parameter has error"
        assert!(errors.contains_key(&ir::Identifier::new("b")));
        let b_errors = errors
            .get(&ir::Identifier::new("b"))
            .expect("b errors should exist");
        assert_eq!(b_errors.len(), 1);

        let parameter_has_error = &b_errors[0];
        assert!(matches!(
            parameter_has_error,
            ParameterResolutionError::VariableResolution(VariableResolutionError::ParameterHasError { identifier, .. })
            if identifier == &ir::Identifier::new("a"),
        ));

        // check the c error for "parameter has error"
        assert!(errors.contains_key(&ir::Identifier::new("c")));
        let c_errors = errors
            .get(&ir::Identifier::new("c"))
            .expect("c errors should exist");
        assert_eq!(c_errors.len(), 1);

        let parameter_has_error = &c_errors[0];
        assert!(matches!(
            parameter_has_error,
            ParameterResolutionError::VariableResolution(VariableResolutionError::ParameterHasError { identifier, .. })
            if identifier == &ir::Identifier::new("b"),
        ));

        // check the resolved parameters - only the first "a" parameter is resolved
        assert_eq!(resolved_params.len(), 1);
        assert!(resolved_params.contains_key(&ir::Identifier::new("a")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_comparison_op() {
        // create a comparison expression with variables
        let left_var = test_ast::identifier_variable_node("a");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_var = test_ast::identifier_variable_node("b");
        let right_expr = test_ast::variable_expr_node(right_var);

        let op_node = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr_node = test_ast::comparison_op_expr_node(op_node, left_expr, right_expr, []);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains(&ir::Identifier::new("a")));
        assert!(result.contains(&ir::Identifier::new("b")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_chained_comparison_op() {
        // create a chained comparison expression: a < b < c
        let left_var = test_ast::identifier_variable_node("a");
        let left_expr = test_ast::variable_expr_node(left_var);
        let middle_var = test_ast::identifier_variable_node("b");
        let middle_expr = test_ast::variable_expr_node(middle_var);
        let right_var = test_ast::identifier_variable_node("c");
        let right_expr = test_ast::variable_expr_node(right_var);

        let op1_node = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2_node = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let rest_chained = vec![(op2_node, right_expr)];

        let expr_node =
            test_ast::comparison_op_expr_node(op1_node, left_expr, middle_expr, rest_chained);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert_eq!(result.len(), 3);
        assert!(result.contains(&ir::Identifier::new("a")));
        assert!(result.contains(&ir::Identifier::new("b")));
        assert!(result.contains(&ir::Identifier::new("c")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_comparison_op_with_literals() {
        // create a comparison expression with one variable and one literal
        let left_var = test_ast::identifier_variable_node("x");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_expr = test_ast::literal_number_expr_node(10.0);

        let op_node = test_ast::comparison_op_node(ast::ComparisonOp::GreaterThan);

        let expr_node = test_ast::comparison_op_expr_node(op_node, left_expr, right_expr, []);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies - should only contain the variable
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ir::Identifier::new("x")));
    }

    #[test]
    fn test_get_expr_internal_dependencies_comparison_op_with_complex_expressions() {
        // create a comparison expression with complex expressions on both sides
        // (a + b) < (c * d)
        let a_var = test_ast::identifier_variable_node("a");
        let a_expr = test_ast::variable_expr_node(a_var);
        let b_var = test_ast::identifier_variable_node("b");
        let b_expr = test_ast::variable_expr_node(b_var);
        let c_var = test_ast::identifier_variable_node("c");
        let c_expr = test_ast::variable_expr_node(c_var);
        let d_var = test_ast::identifier_variable_node("d");
        let d_expr = test_ast::variable_expr_node(d_var);

        // Create (a + b)
        let add_op_node = test_ast::binary_op_node(ast::BinaryOp::Add);
        let left_expr = test_ast::binary_op_expr_node(add_op_node, a_expr, b_expr);

        // Create (c * d)
        let mul_op_node = test_ast::binary_op_node(ast::BinaryOp::Mul);
        let right_expr = test_ast::binary_op_expr_node(mul_op_node, c_expr, d_expr);

        // Create the comparison
        let comp_op_node = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr_node = test_ast::comparison_op_expr_node(comp_op_node, left_expr, right_expr, []);

        // get the dependencies
        let result = get_expr_internal_dependencies(expr_node.node_value(), HashSet::new());
        let result: HashSet<_> = result.into_iter().map(ir::WithSpan::take_value).collect();

        // check the dependencies
        assert_eq!(result.len(), 4);
        assert!(result.contains(&ir::Identifier::new("a")));
        assert!(result.contains(&ir::Identifier::new("b")));
        assert!(result.contains(&ir::Identifier::new("c")));
        assert!(result.contains(&ir::Identifier::new("d")));
    }

    #[test]
    fn test_get_parameter_internal_dependencies_with_comparison_conditions() {
        // create a parameter with a piecewise value that uses comparison conditions
        // create the value expression
        let value_expr = test_ast::literal_number_expr_node(5.0);

        // create a comparison condition: x < threshold
        let x_var = test_ast::identifier_variable_node("x");
        let x_expr = test_ast::variable_expr_node(x_var);
        let threshold_var = test_ast::identifier_variable_node("threshold");
        let threshold_expr = test_ast::variable_expr_node(threshold_var);

        let comp_op_node = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let condition_expr_node =
            test_ast::comparison_op_expr_node(comp_op_node, x_expr, threshold_expr, []);

        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("test")
            .with_piecewise_values([(value_expr, condition_expr_node)])
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);
        let dependencies: HashSet<_> = dependencies
            .into_iter()
            .map(ir::WithSpan::take_value)
            .collect();

        // check the dependencies
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&ir::Identifier::new("x")));
        assert!(dependencies.contains(&ir::Identifier::new("threshold")));
    }
}
