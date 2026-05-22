//! Parameter resolution model for the Oneil model loader.

use std::ops::Deref;

use indexmap::{IndexMap, IndexSet};

use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::{
    labels::{ParameterLabel, SectionLabel},
    span::Span,
    symbols::ParameterName,
};

use crate::{
    ExternalResolutionContext, ResolutionContext,
    error::{self, ParameterResolutionError},
    resolver::{
        resolve_expr::{get_expr_dependencies, get_expr_internal_dependencies, resolve_expr},
        resolve_trace_level::resolve_trace_level,
        resolve_unit::resolve_unit,
        util::ParameterWithSection,
    },
};

/// Resolves a collection of AST parameters into resolved model parameters.
pub fn resolve_parameters<E>(
    parameters: Vec<ParameterWithSection<'_>>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    let mut parameter_map: IndexMap<ParameterName, (Span, ParameterWithSection<'_>)> =
        IndexMap::new();

    // collect all parameters and check for duplicates
    for decl in parameters {
        let ident = ParameterName::from(decl.parameter.ident().as_str());
        let ident_span = decl.parameter.ident().span();

        let maybe_original_parameter = parameter_map.get(&ident);
        if let Some((original_ident_span, _)) = maybe_original_parameter {
            resolution_context.add_parameter_error_to_active_model(
                ident.clone(),
                ParameterResolutionError::duplicate_parameter(
                    ident,
                    original_ident_span.clone(),
                    ident_span.clone(),
                ),
            );
        } else {
            parameter_map.insert(ident, (ident_span.clone(), decl));
        }
    }

    // Drop the ident_span from the map
    //
    // It's main purpose was for duplicate parameter reporting, which is done.
    let parameter_ast_map: IndexMap<ParameterName, ParameterWithSection<'_>> = parameter_map
        .into_iter()
        .map(|(ident, (_ident_span, decl))| (ident, decl))
        .collect();

    // note that an 'internal dependency' is a dependency on a parameter
    // that is defined within the current model
    let dependencies = get_all_parameter_internal_dependencies(&parameter_ast_map);

    let mut parameters_visited = IndexSet::new();

    for parameter_name in parameter_ast_map.keys() {
        try_resolve_identifier_as_parameter(
            parameter_name,
            &parameter_ast_map,
            &dependencies,
            &mut parameters_visited,
            resolution_context,
        );
    }
}

/// Analyzes all parameters to extract their internal dependencies.
///
/// Note that the dependencies are both parameter names and builtins, which
/// is why we use identifiers instead of parameter names.
fn get_all_parameter_internal_dependencies<'a>(
    parameter_map: &'a IndexMap<ParameterName, ParameterWithSection<'a>>,
) -> IndexMap<&'a ParameterName, IndexMap<ast::Identifier, Span>> {
    let mut dependencies = IndexMap::new();

    for identifier in parameter_map.keys() {
        let decl = parameter_map
            .get(identifier)
            .expect("parameter should exist");

        let param_dependencies = get_parameter_internal_dependencies(decl.parameter);

        dependencies.insert(identifier, param_dependencies);
    }

    dependencies
}

/// Extracts internal dependencies from a single parameter.
fn get_parameter_internal_dependencies(
    parameter: &ast::Parameter,
) -> IndexMap<ast::Identifier, Span> {
    let mut dependencies = IndexMap::new();

    let limits = parameter.limits().map(ast::Node::deref);
    match limits {
        Some(ast::Limits::Continuous { min, max }) => {
            let min_dependencies = get_expr_internal_dependencies(min);
            dependencies.extend(min_dependencies);

            let max_dependencies = get_expr_internal_dependencies(max);
            dependencies.extend(max_dependencies);
        }
        Some(ast::Limits::Discrete { values }) => {
            for expr in values {
                let expr_dependencies = get_expr_internal_dependencies(expr);
                dependencies.extend(expr_dependencies);
            }
        }
        None => {}
    }

    match parameter.value().deref() {
        ast::ParameterValue::Simple(expr, _) => {
            let expr_dependencies = get_expr_internal_dependencies(expr);
            dependencies.extend(expr_dependencies);
        }
        ast::ParameterValue::Piecewise(piecewise, _) => {
            for part in piecewise {
                let if_expr_dependencies = get_expr_internal_dependencies(part.if_expr());
                dependencies.extend(if_expr_dependencies);

                let expr_dependencies = get_expr_internal_dependencies(part.expr());
                dependencies.extend(expr_dependencies);
            }
        }
    }

    dependencies
}

/// Tries to resolve a single identifier as a parameter.
///
/// If the parameter is not found, this will immediately return. In the
/// case that the identifier is a builtin, it is considered to be already resolved.
/// Otherwise, the error will show up later when attempting to resolve the identifier as
/// a "parameter not found" error.
///
/// `parameters_visited` is used for two purposes:
///
/// 1. **Cascade suppression**: dependencies are resolved before dependents so
///    that `ParameterHasError` propagates correctly. If `a` has an expression
///    error, `b = a + 1` gets suppressed rather than emitting a confusing
///    secondary "undefined parameter `a`" at validation time.
///
/// 2. **Recursion guard**: once a parameter starts being resolved its name is
///    inserted, so a recursive call to the same parameter returns immediately
///    rather than looping. Parameter *cycle* diagnostics are emitted by the
///    post-build SCC pass in [`oneil_analysis::validate_instance_graph`], which
///    sees the fully composed (post-design) graph rather than this file-time
///    template.
fn try_resolve_identifier_as_parameter<E>(
    parameter_name: &ParameterName,
    parameter_ast_map: &IndexMap<ParameterName, ParameterWithSection<'_>>,
    dependencies: &IndexMap<&ParameterName, IndexMap<ast::Identifier, Span>>,
    parameters_visited: &mut IndexSet<ParameterName>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    // check that the parameter exists
    let Some(parameter_decl) = parameter_ast_map.get(parameter_name) else {
        // This is technically a resolution error. However, this error will
        // be caught later when the variable is resolved. In order to avoid
        // duplicate errors, we return Ok(()) and let the variable resolution
        // handle the "not found" error
        //
        // This also accounts for the fact that the parameter may be a builtin
        return;
    };

    let parameter_identifier_span = parameter_decl.parameter.ident().span();

    assert!(
        dependencies.contains_key(&parameter_name),
        "parameter dependencies for '{parameter_name:?}' not found",
    );

    if parameters_visited.contains(parameter_name) {
        return;
    }
    parameters_visited.insert(parameter_name.clone());

    let parameter_dependencies = dependencies
        .get(&parameter_name)
        .expect("parameter dependencies should exist");

    for dependency_identifier in parameter_dependencies.keys() {
        let dependency_name = ParameterName::from(dependency_identifier.as_str());
        try_resolve_identifier_as_parameter(
            &dependency_name,
            parameter_ast_map,
            dependencies,
            parameters_visited,
            resolution_context,
        );
    }

    // resolve the parameter
    let section_label: Option<SectionLabel> = parameter_decl
        .section_label
        .map(|node| node.deref().clone());

    let label = ParameterLabel::from(parameter_decl.parameter.label().as_str());

    let render_name = parameter_decl
        .parameter
        .render_name()
        .map(|n| n.deref().clone());

    let value = resolve_parameter_value(parameter_decl.parameter.value(), resolution_context);

    let limits = resolve_limits(parameter_decl.parameter.limits(), resolution_context);

    let is_performance = parameter_decl.parameter.performance_marker().is_some();

    let trace_level = resolve_trace_level(parameter_decl.parameter.trace_level());

    let note = parameter_decl
        .parameter
        .note()
        .map(|n| ir::Note::new(n.value().to_string()));

    match error::combine_errors(value, limits) {
        Ok((value, limits)) => {
            // build the parameter
            let parameter_dependencies = get_parameter_dependencies(&value, &limits);

            let parameter = ir::Parameter::new(
                parameter_dependencies,
                parameter_name.clone(),
                parameter_identifier_span.clone(),
                parameter_decl.parameter.span().clone(),
                label,
                render_name,
                section_label,
                value,
                limits,
                is_performance,
                trace_level,
                note,
            );

            // add the parameter to the parameter builder
            resolution_context.add_parameter_to_active_model(parameter_name.clone(), parameter);
        }
        Err(errors) => {
            // add the errors to the parameter builder
            for error in errors {
                resolution_context
                    .add_parameter_error_to_active_model(parameter_name.clone(), error);
            }
        }
    }
}

/// Resolves a parameter value expression.
pub(super) fn resolve_parameter_value<E>(
    value: &ast::ParameterValue,
    resolution_context: &ResolutionContext<'_, E>,
) -> Result<ir::ParameterValue, Vec<ParameterResolutionError>>
where
    E: ExternalResolutionContext,
{
    match value {
        ast::ParameterValue::Simple(expr, unit) => {
            let expr = resolve_expr(expr, resolution_context).map_err(error::convert_errors)?;

            let unit = unit
                .as_ref()
                .map(|u| resolve_unit(u, resolution_context))
                .transpose()
                .map_err(error::convert_errors)?;

            Ok(ir::ParameterValue::simple(expr, unit))
        }

        ast::ParameterValue::Piecewise(piecewise, unit) => {
            let exprs = piecewise.iter().map(|part| {
                let expr =
                    resolve_expr(part.expr(), resolution_context).map_err(error::convert_errors);

                let if_expr =
                    resolve_expr(part.if_expr(), resolution_context).map_err(error::convert_errors);

                let (expr, if_expr) = error::combine_errors(expr, if_expr)?;

                Ok(ir::PiecewiseExpr::new(expr, if_expr))
            });

            let unit = unit
                .as_ref()
                .map(|u| resolve_unit(u, resolution_context))
                .transpose()
                .map_err(error::convert_errors)?;

            let exprs = error::combine_error_list(exprs)?;

            Ok(ir::ParameterValue::piecewise(exprs, unit))
        }
    }
}

/// Resolves parameter limits.
pub(super) fn resolve_limits<E>(
    limits: Option<&ast::LimitsNode>,
    resolution_context: &ResolutionContext<'_, E>,
) -> Result<ir::Limits, Vec<ParameterResolutionError>>
where
    E: ExternalResolutionContext,
{
    match limits.map(|limits| (&**limits, limits.span())) {
        Some((ast::Limits::Continuous { min, max }, span)) => {
            let min = resolve_expr(min, resolution_context).map_err(error::convert_errors);

            let max = resolve_expr(max, resolution_context).map_err(error::convert_errors);

            let (min, max) = error::combine_errors(min, max)?;

            Ok(ir::Limits::continuous(min, max, span.clone()))
        }
        Some((ast::Limits::Discrete { values }, span)) => {
            let values = values.iter().map(|value| {
                resolve_expr(value, resolution_context).map_err(error::convert_errors)
            });

            let values = error::combine_error_list(values)?;

            Ok(ir::Limits::discrete(values, span.clone()))
        }
        None => Ok(ir::Limits::default()),
    }
}

pub fn get_parameter_dependencies(
    parameter_value: &ir::ParameterValue,
    parameter_limits: &ir::Limits,
) -> ir::Dependencies {
    let mut dependencies = ir::Dependencies::new();

    match parameter_limits {
        ir::Limits::Continuous {
            min,
            max,
            limit_expr_span: _,
        } => {
            let min_dependencies = get_expr_dependencies(min);
            dependencies.extend(min_dependencies);

            let max_dependencies = get_expr_dependencies(max);
            dependencies.extend(max_dependencies);
        }
        ir::Limits::Discrete {
            values,
            limit_expr_span: _,
        } => {
            for expr in values {
                let expr_dependencies = get_expr_dependencies(expr);
                dependencies.extend(expr_dependencies);
            }
        }
        ir::Limits::Default => {}
    }

    match parameter_value {
        ir::ParameterValue::Simple(expr, _) => {
            let expr_dependencies = get_expr_dependencies(expr);
            dependencies.extend(expr_dependencies);
        }
        ir::ParameterValue::Piecewise(piecewise, _) => {
            for part in piecewise {
                let if_expr_dependencies = get_expr_dependencies(part.if_expr());
                dependencies.extend(if_expr_dependencies);

                let expr_dependencies = get_expr_dependencies(part.expr());
                dependencies.extend(expr_dependencies);
            }
        }
    }

    dependencies
}

/// Returns dependencies introduced only by limit expressions.
pub fn get_limits_dependencies(limits: &ir::Limits) -> ir::Dependencies {
    get_parameter_dependencies(
        &ir::ParameterValue::simple(
            ir::Expr::literal(
                oneil_shared::span::Span::synthetic(),
                ir::Literal::number(0.0),
            ),
            None,
        ),
        limits,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        error::VariableResolutionError,
        resolver::ParameterWithSection,
        test::{
            external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
            test_ast, test_model_path,
        },
    };

    use super::*;
    use oneil_ast as ast;
    use oneil_ir as ir;

    fn param_name(s: &str) -> ParameterName {
        ParameterName::from(s)
    }

    #[test]
    fn resolve_parameters_empty() {
        // build the parameters
        let parameters: Vec<ParameterWithSection<'_>> = vec![];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        assert!(resolution_context.get_active_model_parameters().is_empty());

        // check the errors
        assert!(
            resolution_context
                .get_active_model_parameter_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_parameters_simple() {
        // build the parameters
        let param_a = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_number_value(20.0)
            .build();
        let parameters = vec![
            ParameterWithSection {
                parameter: &param_a,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_b,
                section_label: None,
            },
        ];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        let resolved = resolution_context.get_active_model_parameters();
        assert_eq!(resolved.len(), 2);
        let param_a = resolved
            .get(&param_name("a"))
            .expect("param a should exist");
        let param_b = resolved
            .get(&param_name("b"))
            .expect("param b should exist");
        assert!(param_a.dependencies().parameter().is_empty());
        assert!(param_b.dependencies().parameter().is_empty());

        // check the errors
        assert!(
            resolution_context
                .get_active_model_parameter_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_parameters_with_dependencies() {
        // build the parameters
        let param_a = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_b = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("b")
            .with_dependent_parameter_values(["a"])
            .build();
        let parameters = vec![
            ParameterWithSection {
                parameter: &param_a,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_b,
                section_label: None,
            },
        ];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        let resolved = resolution_context.get_active_model_parameters();
        assert_eq!(resolved.len(), 2);
        let param_a = resolved
            .get(&param_name("a"))
            .expect("param a should exist");
        let param_b = resolved
            .get(&param_name("b"))
            .expect("param b should exist");
        assert!(param_a.dependencies().parameter().is_empty());
        assert!(
            param_b
                .dependencies()
                .parameter()
                .contains_key(&param_name("a"))
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_parameter_errors()
                .is_empty()
        );
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
        assert!(dependencies.contains_key(&ast::Identifier::from("b".to_string())));
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
        assert!(dependencies.contains_key(&ast::Identifier::from("min_val".to_string())));
        assert!(dependencies.contains_key(&ast::Identifier::from("max_val".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_literal() {
        // create a literal expression
        let expr = test_ast::literal_number_expr_node(42.0);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr);

        // check the dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn get_expr_internal_dependencies_variable() {
        // create a variable expression
        let variable = test_ast::identifier_variable_node("test_var");
        let expr = test_ast::variable_expr_node(variable);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr);

        // check the dependencies
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&ast::Identifier::from("test_var".to_string())));
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
        let result = get_expr_internal_dependencies(&expr);

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ast::Identifier::from("a".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("b".to_string())));
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
        let result = get_expr_internal_dependencies(&expr);

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ast::Identifier::from("arg1".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("arg2".to_string())));
    }

    #[test]
    fn get_expr_internal_dependencies_accessor() {
        // create an accessor variable
        let variable = test_ast::model_parameter_variable_node("reference_model", "parameter");
        let expr = test_ast::variable_expr_node(variable);

        // get the dependencies
        let result = get_expr_internal_dependencies(&expr);

        // check the dependencies - accessors don't count as internal dependencies
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_parameter_value_simple() {
        // build the parameter value node
        let expr = test_ast::literal_number_expr_node(42.0);
        let value_node = test_ast::simple_parameter_value_node(expr);

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the parameter value
        let result = resolve_parameter_value(&value_node, &resolution_context);

        // check the result
        assert!(matches!(result, Ok(ir::ParameterValue::Simple(_, None))));
    }

    #[test]
    fn resolve_limits_none() {
        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the limits
        let result = resolve_limits(None, &resolution_context);

        // check the result
        assert_eq!(result, Ok(ir::Limits::default()));
    }

    #[test]
    fn resolve_limits_continuous() {
        // build the limits node and context
        let limits_node = test_ast::continuous_limits_node(0.0, 100.0);
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the limits
        let result = resolve_limits(Some(&limits_node), &resolution_context);

        // check the result
        assert!(matches!(result, Ok(ir::Limits::Continuous { .. })));
    }

    #[test]
    fn resolve_limits_discrete() {
        // build the limits node and context
        let limits_node = test_ast::discrete_limits_node([1.0, 2.0, 3.0]);
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // resolve the limits
        let result = resolve_limits(Some(&limits_node), &resolution_context);

        // check the result
        assert!(matches!(result, Ok(ir::Limits::Discrete { .. })));
    }

    #[test]
    fn resolve_parameters_duplicate_parameters() {
        // build the parameters with duplicate identifiers
        let param_a1 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(10.0)
            .build();
        let param_a2 = test_ast::ParameterNodeBuilder::new()
            .with_ident_and_label("a")
            .with_number_value(20.0)
            .build();
        let parameters = vec![
            ParameterWithSection {
                parameter: &param_a1,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_a2,
                section_label: None,
            },
        ];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        let resolved = resolution_context.get_active_model_parameters();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&param_name("a")));

        // check the errors
        let errors = resolution_context.get_active_model_parameter_errors();
        assert_eq!(errors.len(), 1);

        // check the first duplicate error
        let a_errors = errors.get(&param_name("a")).expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = &a_errors[0]
        else {
            panic!("duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "a");
    }

    #[test]
    fn resolve_parameters_multiple_duplicate_parameters() {
        // build the parameters with multiple duplicates
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
        let parameters = vec![
            ParameterWithSection {
                parameter: &param_foo1,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_bar1,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_foo2,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_bar2,
                section_label: None,
            },
        ];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        let resolved = resolution_context.get_active_model_parameters();
        assert_eq!(resolved.len(), 2);
        assert!(resolved.contains_key(&param_name("foo")));
        assert!(resolved.contains_key(&param_name("bar")));

        // check the errors
        let errors = resolution_context.get_active_model_parameter_errors();
        assert_eq!(errors.len(), 2);

        let foo_errors = errors
            .get(&param_name("foo"))
            .expect("foo errors should exist");
        let bar_errors = errors
            .get(&param_name("bar"))
            .expect("bar errors should exist");
        assert_eq!(foo_errors.len(), 1);
        assert_eq!(bar_errors.len(), 1);

        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = &foo_errors[0]
        else {
            panic!("foo duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "foo");

        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = &bar_errors[0]
        else {
            panic!("bar duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "bar");
    }

    #[test]
    fn resolve_parameters_duplicate_parameters_with_valid_parameters() {
        // build the parameters with duplicates and valid parameters
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
        let parameters = vec![
            ParameterWithSection {
                parameter: &param_a1,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_b,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_a2,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_c,
                section_label: None,
            },
        ];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        let resolved = resolution_context.get_active_model_parameters();
        assert_eq!(resolved.len(), 3);
        assert!(resolved.contains_key(&param_name("a")));
        assert!(resolved.contains_key(&param_name("b")));
        assert!(resolved.contains_key(&param_name("c")));

        // check the errors
        let errors = resolution_context.get_active_model_parameter_errors();
        assert_eq!(errors.len(), 1);

        let a_errors = errors.get(&param_name("a")).expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = &a_errors[0]
        else {
            panic!("duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "a");
    }

    #[test]
    fn resolve_parameters_duplicate_parameters_with_dependencies() {
        // build the parameters with duplicates and dependencies
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
        let parameters = vec![
            ParameterWithSection {
                parameter: &param_a1,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_b,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_a2,
                section_label: None,
            },
            ParameterWithSection {
                parameter: &param_c,
                section_label: None,
            },
        ];

        // build the context
        let active_path = test_model_path("main");
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(active_path)
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_parameters(parameters, &mut resolution_context);

        // check the resolved parameters
        let resolved = resolution_context.get_active_model_parameters();
        assert_eq!(resolved.len(), 1);
        assert!(resolved.contains_key(&param_name("a")));

        // check the errors
        let errors = resolution_context.get_active_model_parameter_errors();
        assert_eq!(errors.len(), 3);

        let a_errors = errors.get(&param_name("a")).expect("a errors should exist");
        assert_eq!(a_errors.len(), 1);

        let ParameterResolutionError::DuplicateParameter { parameter_name, .. } = &a_errors[0]
        else {
            panic!("duplicate error should be a duplicate parameter error");
        };
        assert_eq!(parameter_name.as_str(), "a");

        let b_errors = errors.get(&param_name("b")).expect("b errors should exist");
        assert_eq!(b_errors.len(), 1);

        let ParameterResolutionError::VariableResolution(
            VariableResolutionError::ParameterHasError { parameter_name, .. },
        ) = &b_errors[0]
        else {
            panic!("parameter has error should be a parameter has error");
        };
        assert_eq!(parameter_name.as_str(), "a");

        let c_errors = errors.get(&param_name("c")).expect("c errors should exist");
        assert_eq!(c_errors.len(), 1);

        let ParameterResolutionError::VariableResolution(
            VariableResolutionError::ParameterHasError { parameter_name, .. },
        ) = &c_errors[0]
        else {
            panic!("parameter has error should be a parameter has error");
        };
        assert_eq!(parameter_name.as_str(), "b");
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
        let result = get_expr_internal_dependencies(&expr_node);

        // check the dependencies
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&ast::Identifier::from("a".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("b".to_string())));
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
        let result = get_expr_internal_dependencies(&expr_node);

        // check the dependencies
        assert_eq!(result.len(), 3);
        assert!(result.contains_key(&ast::Identifier::from("a".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("b".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("c".to_string())));
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
        let result = get_expr_internal_dependencies(&expr_node);

        // check the dependencies - should only contain the variable
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&ast::Identifier::from("x".to_string())));
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
        let result = get_expr_internal_dependencies(&expr_node);

        // check the dependencies
        assert_eq!(result.len(), 4);
        assert!(result.contains_key(&ast::Identifier::from("a".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("b".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("c".to_string())));
        assert!(result.contains_key(&ast::Identifier::from("d".to_string())));
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
        assert!(dependencies.contains_key(&ast::Identifier::from("x".to_string())));
        assert!(dependencies.contains_key(&ast::Identifier::from("threshold".to_string())));
    }
}
