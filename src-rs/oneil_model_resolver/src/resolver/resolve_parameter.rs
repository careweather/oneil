//! Parameter resolution model for the Oneil model loader.

use std::{collections::HashMap, collections::HashSet, ops::Deref};

use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    BuiltinRef,
    error::{self, ParameterResolutionError},
    resolver::{
        resolve_expr::{get_expr_internal_dependencies, resolve_expr},
        resolve_trace_level::resolve_trace_level,
        resolve_unit::resolve_unit,
    },
    util::{
        Stack,
        builder::ParameterBuilder,
        context::{ParameterContext, ReferenceContext},
    },
};

pub type ParameterMap = HashMap<ir::ParameterName, ir::Parameter>;
pub type ParameterErrorMap = HashMap<ir::ParameterName, Vec<ParameterResolutionError>>;

/// Resolves a collection of AST parameters into resolved model parameters.
pub fn resolve_parameters(
    parameters: Vec<&ast::ParameterNode>,
    builtin_ref: &impl BuiltinRef,
    context: &ReferenceContext<'_, '_>,
) -> (ParameterMap, ParameterErrorMap) {
    let mut parameter_builder = ParameterBuilder::new();

    let mut parameter_map = HashMap::new();

    for parameter in parameters {
        let ident = ir::ParameterName::new(parameter.ident().as_str().to_string());
        let ident_span = parameter.ident().span();

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

    // Convert parameter nodes into a map
    let parameter_ast_map: HashMap<_, &ast::ParameterNode> = parameter_map
        .into_iter()
        .map(|(ident, (_, ast))| (ident, ast))
        .collect();

    // note that an 'internal dependency' is a dependency on a parameter
    // that is defined within the current model
    let dependencies = get_all_parameter_internal_dependencies(&parameter_ast_map);

    for parameter_identifier in parameter_ast_map.keys() {
        let mut parameter_stack = Stack::new();

        parameter_builder = resolve_parameter(
            parameter_identifier.clone(),
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
fn get_all_parameter_internal_dependencies<'a>(
    parameter_map: &'a HashMap<ir::ParameterName, &'a ast::ParameterNode>,
) -> HashMap<&'a ir::ParameterName, HashMap<ir::ParameterName, Span>> {
    let mut dependencies = HashMap::new();

    for identifier in parameter_map.keys() {
        let parameter = parameter_map
            .get(identifier)
            .expect("parameter should exist");

        let param_dependencies = get_parameter_internal_dependencies(parameter);

        dependencies.insert(identifier, param_dependencies);
    }

    dependencies
}

/// Extracts internal dependencies from a single parameter.
fn get_parameter_internal_dependencies(
    parameter: &ast::Parameter,
) -> HashMap<ir::ParameterName, Span> {
    let dependencies = HashMap::new();

    let limits = parameter.limits().map(ast::Node::deref);
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

    match parameter.value().deref() {
        ast::ParameterValue::Simple(expr, _) => get_expr_internal_dependencies(expr, dependencies),
        ast::ParameterValue::Piecewise(piecewise, _) => {
            piecewise.iter().fold(dependencies, |dependencies, part| {
                let dependencies = get_expr_internal_dependencies(part.if_expr(), dependencies);
                get_expr_internal_dependencies(part.expr(), dependencies)
            })
        }
    }
}

/// Resolves a single parameter with dependency tracking.
fn resolve_parameter(
    parameter_identifier: ir::ParameterName,
    // context
    parameter_ast_map: &HashMap<ir::ParameterName, &ast::ParameterNode>,
    dependencies: &HashMap<&ir::ParameterName, HashMap<ir::ParameterName, Span>>,
    parameter_stack: &mut Stack<ir::ParameterName>,
    builtin_ref: &impl BuiltinRef,
    context: &ReferenceContext<'_, '_>,
    // builder
    mut parameter_builder: ParameterBuilder,
) -> ParameterBuilder {
    // check that the parameter exists
    let Some(param) = parameter_ast_map.get(&parameter_identifier) else {
        // This is technically a resolution error. However, this error will
        // be caught later when the variable is resolved. In order to avoid
        // duplicate errors, we return Ok(()) and let the variable resolution
        // handle the "not found" error
        //
        // This also accounts for the fact that the parameter may be a builtin
        return parameter_builder;
    };

    let parameter_identifier_span = param.ident().span();

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
    for dependency_identifier in parameter_dependencies.keys() {
        parameter_builder = resolve_parameter(
            dependency_identifier.clone(),
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

    let label = ir::Label::new(parameter.label().as_str().to_string());

    let value =
        resolve_parameter_value(parameter.value(), builtin_ref, context, &parameter_context);

    let limits = resolve_limits(parameter.limits(), builtin_ref, context, &parameter_context);

    let is_performance = parameter.performance_marker().is_some();

    let trace_level = resolve_trace_level(parameter.trace_level());

    match error::combine_errors(value, limits) {
        Ok((value, limits)) => {
            // build the parameter
            let parameter_dependencies = parameter_dependencies.clone();

            let parameter = ir::Parameter::new(
                parameter_dependencies,
                ident,
                parameter_identifier_span,
                parameter.span(),
                label,
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
fn resolve_limits(
    limits: Option<&ast::LimitsNode>,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Limits, Vec<ParameterResolutionError>> {
    match limits.map(|limits| (&**limits, limits.span())) {
        Some((ast::Limits::Continuous { min, max }, span)) => {
            let min = resolve_expr(min, builtin_ref, reference_context, parameter_context)
                .map_err(error::convert_errors);

            let max = resolve_expr(max, builtin_ref, reference_context, parameter_context)
                .map_err(error::convert_errors);

            let (min, max) = error::combine_errors(min, max)?;

            Ok(ir::Limits::continuous(min, max, span))
        }
        Some((ast::Limits::Discrete { values }, span)) => {
            let values = values.iter().map(|value| {
                resolve_expr(value, builtin_ref, reference_context, parameter_context)
                    .map_err(error::convert_errors)
            });

            let values = error::combine_error_list(values)?;

            Ok(ir::Limits::discrete(values, span))
        }
        None => Ok(ir::Limits::default()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
    fn resolve_parameters_empty() {
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
    fn resolve_parameters_simple() {
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
            .get(&ir::ParameterName::new("a".to_string()))
            .expect("param a should exist");
        let param_b = resolved_params
            .get(&ir::ParameterName::new("b".to_string()))
            .expect("param b should exist");

        assert!(param_a.dependencies().is_empty());
        assert!(param_b.dependencies().is_empty());
    }

    #[test]
    fn resolve_parameters_with_dependencies() {
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
            .get(&ir::ParameterName::new("a".to_string()))
            .expect("param a should exist");
        let param_b = resolved_params
            .get(&ir::ParameterName::new("b".to_string()))
            .expect("param b should exist");

        assert!(param_a.dependencies().is_empty());
        assert!(
            param_b
                .dependencies()
                .contains_key(&ir::ParameterName::new("a".to_string()))
        );
    }

    #[test]
    fn resolve_parameters_circular_dependency() {
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

        assert!(errors.contains_key(&ir::ParameterName::new("a".to_string())));
        assert!(errors.contains_key(&ir::ParameterName::new("b".to_string())));

        let a_errors = errors
            .get(&ir::ParameterName::new("a".to_string()))
            .expect("a errors should exist");
        let b_errors = errors
            .get(&ir::ParameterName::new("b".to_string()))
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
                    parameter_name,
                    reference_span: _,
                }
            )
            if parameter_name.as_str() == "b",
        )));

        assert!(b_errors.iter().any(|e| matches!(
            e,
            ParameterResolutionError::VariableResolution(
                VariableResolutionError::ParameterHasError {
                    parameter_name,
                    reference_span: _,
                }
            )
            if parameter_name.as_str() == "a",
        )));

        // check the resolved parameters
        assert!(resolved_params.is_empty());
    }

    #[test]
    fn get_parameter_internal_dependencies_simple() {
        // create a simple parameter
        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);

        // check the dependencies
        assert!(dependencies.is_empty());
    }

    #[test]
    fn get_parameter_internal_dependencies_with_variable() {
        // create a dependent parameter
        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_dependent_parameter_values(["b"])
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);

        // check the dependencies
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains_key(&ir::ParameterName::new("b".to_string())));
    }

    #[test]
    fn get_parameter_internal_dependencies_with_limits() {
        // create the parameter
        let parameter = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("test")
            .with_number_value(5.0)
            .with_continuous_limit_vars("min_val", "max_val")
            .build();

        // get the dependencies
        let dependencies = get_parameter_internal_dependencies(&parameter);

        // check the dependencies
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains_key(&ir::ParameterName::new("min_val".to_string())));
        assert!(dependencies.contains_key(&ir::ParameterName::new("max_val".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_literal() {
        // create a literal expression
        let expr = test_ast::literal_number_expr_node(42.0);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr, HashMap::new());

        // check the dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn get_expr_internal_dependencies_variable() {
        // create a variable expression
        let variable = test_ast::identifier_variable_node("test_var");
        let expr = test_ast::variable_expr_node(variable);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr, HashMap::new());

        // check the dependencies
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&ir::ParameterName::new("test_var".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_binary_op() {
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
        let result = get_expr_internal_dependencies(&expr, HashMap::new());

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ir::ParameterName::new("a".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("b".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_function_call() {
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
        let result = get_expr_internal_dependencies(&expr, HashMap::new());

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ir::ParameterName::new("arg1".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("arg2".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_accessor() {
        // create an accessor variable
        let variable = test_ast::model_parameter_variable_node("reference_model", "parameter");
        let expr = test_ast::variable_expr_node(variable);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr, HashMap::new());

        // check the dependencies - accessors don't count as internal dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_parameter_value_simple() {
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
    fn resolve_limits_none() {
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
    fn resolve_limits_continuous() {
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
    fn resolve_limits_discrete() {
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
    fn resolve_parameters_duplicate_parameters() {
        // create the parameters with duplicate identifiers
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(20.0)
            .build();
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

        let a_errors = errors
            .get(&ir::ParameterName::new("a".to_string()))
            .expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];

        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = duplicate_error
        else {
            panic!("duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "a");

        // check the resolved parameters - should have one parameter, the
        //     parameter that is present is left unspecified
        assert_eq!(resolved_params.len(), 1);
        assert!(resolved_params.contains_key(&ir::ParameterName::new("a".to_string())));
    }

    #[test]
    fn resolve_parameters_multiple_duplicate_parameters() {
        // create the parameters with multiple duplicates
        let param_foo1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("foo")
            .with_number_value(10.0)
            .build();
        let param_bar1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("bar")
            .with_number_value(20.0)
            .build();
        let param_foo2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("foo")
            .with_number_value(30.0)
            .build();
        let param_bar2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("bar")
            .with_number_value(40.0)
            .build();
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

        let foo_errors = errors
            .get(&ir::ParameterName::new("foo".to_string()))
            .expect("foo errors should exist");
        let bar_errors = errors
            .get(&ir::ParameterName::new("bar".to_string()))
            .expect("bar errors should exist");
        assert_eq!(foo_errors.len(), 1);
        assert_eq!(bar_errors.len(), 1);

        // check that both errors are duplicate parameter errors
        let foo_duplicate_error = &foo_errors[0];
        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } =
            foo_duplicate_error
        else {
            panic!("foo duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "foo");

        let bar_duplicate_error = &bar_errors[0];
        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } =
            bar_duplicate_error
        else {
            panic!("bar duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "bar");

        // check the resolved parameters - should have two parameters, the
        //     parameters that are present are left unspecified
        assert_eq!(resolved_params.len(), 2);
        assert!(resolved_params.contains_key(&ir::ParameterName::new("foo".to_string())));
        assert!(resolved_params.contains_key(&ir::ParameterName::new("bar".to_string())));
    }

    #[test]
    fn resolve_parameters_duplicate_parameters_with_valid_parameters() {
        // create the parameters with duplicates and valid parameters
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_number_value(20.0)
            .build();
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(30.0)
            .build();
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

        let a_errors = errors
            .get(&ir::ParameterName::new("a".to_string()))
            .expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        // check that the error is a duplicate parameter error
        let duplicate_error = &a_errors[0];
        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = duplicate_error
        else {
            panic!("duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "a");

        // check the resolved parameters
        assert_eq!(resolved_params.len(), 3);
        assert!(resolved_params.contains_key(&ir::ParameterName::new("a".to_string())));
        assert!(resolved_params.contains_key(&ir::ParameterName::new("b".to_string())));
        assert!(resolved_params.contains_key(&ir::ParameterName::new("c".to_string())));
    }

    #[test]
    fn resolve_parameters_duplicate_parameters_with_dependencies() {
        // create the parameters with duplicates and dependencies
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_dependent_parameter_values(["a"])
            .build();
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(20.0)
            .build();
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
        let a_errors = errors
            .get(&ir::ParameterName::new("a".to_string()))
            .expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        let duplicate_error = &a_errors[0];
        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = duplicate_error
        else {
            panic!("duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "a");

        // check the b error for "parameter has error"
        assert!(errors.contains_key(&ir::ParameterName::new("b".to_string())));
        let b_errors = errors
            .get(&ir::ParameterName::new("b".to_string()))
            .expect("b errors should exist");
        assert_eq!(b_errors.len(), 1);

        let parameter_has_error = &b_errors[0];
        let ParameterResolutionError::VariableResolution(
            VariableResolutionError::ParameterHasError { parameter_name, .. },
        ) = parameter_has_error
        else {
            panic!("parameter has error should be a parameter has error");
        };
        assert_eq!(parameter_name.as_str(), "a");

        // check the c error for "parameter has error"
        let c_errors = errors
            .get(&ir::ParameterName::new("c".to_string()))
            .expect("c errors should exist");
        assert_eq!(c_errors.len(), 1);

        let parameter_has_error = &c_errors[0];
        let ParameterResolutionError::VariableResolution(
            VariableResolutionError::ParameterHasError { parameter_name, .. },
        ) = parameter_has_error
        else {
            panic!("parameter has error should be a parameter has error");
        };
        assert_eq!(parameter_name.as_str(), "b");

        // check the resolved parameters - only the first "a" parameter is resolved
        assert_eq!(resolved_params.len(), 1);
        assert!(resolved_params.contains_key(&ir::ParameterName::new("a".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_comparison_op() {
        // create a comparison expression with variables
        let left_var = test_ast::identifier_variable_node("a");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_var = test_ast::identifier_variable_node("b");
        let right_expr = test_ast::variable_expr_node(right_var);

        let op_node = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr_node = test_ast::comparison_op_expr_node(op_node, left_expr, right_expr, []);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr_node, HashMap::new());

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ir::ParameterName::new("a".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("b".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_chained_comparison_op() {
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
        let result = get_expr_internal_dependencies(&expr_node, HashMap::new());

        // check the dependencies
        assert_eq!(result.len(), 3);
        assert!(result.contains_key(&ir::ParameterName::new("a".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("b".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("c".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_comparison_op_with_literals() {
        // create a comparison expression with one variable and one literal
        let left_var = test_ast::identifier_variable_node("x");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_expr = test_ast::literal_number_expr_node(10.0);

        let op_node = test_ast::comparison_op_node(ast::ComparisonOp::GreaterThan);

        let expr_node = test_ast::comparison_op_expr_node(op_node, left_expr, right_expr, []);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr_node, HashMap::new());

        // check the dependencies - should only contain the variable
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&ir::ParameterName::new("x".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_comparison_op_with_complex_expressions() {
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
        let result = get_expr_internal_dependencies(&expr_node, HashMap::new());

        // check the dependencies
        assert_eq!(result.len(), 4);
        assert!(result.contains_key(&ir::ParameterName::new("a".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("b".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("c".to_string())));
        assert!(result.contains_key(&ir::ParameterName::new("d".to_string())));
    }

    #[test]
    fn get_parameter_internal_dependencies_with_comparison_conditions() {
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

        // check the dependencies
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains_key(&ir::ParameterName::new("x".to_string())));
        assert!(dependencies.contains_key(&ir::ParameterName::new("threshold".to_string())));
    }
}
