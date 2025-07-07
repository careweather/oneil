use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_module::{
    parameter::{Parameter, ParameterCollection, ParameterValue, PiecewiseExpr},
    reference::Identifier,
};

use crate::{
    error::{self, ParameterResolutionError},
    loader::resolver::{
        ModuleInfo, ParameterInfo, SubmodelInfo, expr::resolve_expr,
        trace_level::resolve_trace_level, unit::resolve_unit,
    },
    util::{Stack, builder::ParameterCollectionBuilder, info::InfoMap},
};

pub fn resolve_parameters(
    parameters: Vec<ast::Parameter>,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
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
    // that is defined within the current module
    let dependencies = get_all_parameter_internal_dependencies(&parameter_map);

    let resolved_parameters = ParameterCollectionBuilder::new();
    let visited = HashSet::new();

    let (resolved_parameters, _visited) = parameter_map.keys().fold(
        (resolved_parameters, visited),
        |(resolved_parameters, visited), parameter_identifier| {
            let mut parameter_stack = Stack::new();

            let (mut resolved_parameters, visited, result) = resolve_parameter(
                parameter_identifier,
                &parameter_map,
                &dependencies,
                submodel_info,
                module_info,
                &mut parameter_stack,
                resolved_parameters,
                visited,
            );

            match result {
                Ok(()) => (resolved_parameters, visited),
                Err(errors) => {
                    resolved_parameters.add_error_list(parameter_identifier.clone(), errors);
                    (resolved_parameters, visited)
                }
            }
        },
    );

    let resolved_parameters = resolved_parameters.try_into();

    match resolved_parameters {
        Ok(resolved_parameters) => (resolved_parameters, HashMap::new()),
        Err((resolved_parameters, resolution_errors)) => (resolved_parameters, resolution_errors),
    }
}

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
                // outside of the current module, so it doesn't count as an
                // internal dependency
                dependencies
            }
        },
        ast::Expr::Literal(_) => dependencies,
    }
}

fn resolve_parameter(
    parameter_identifier: &Identifier,
    parameter_map: &HashMap<Identifier, oneil_ast::Parameter>,
    dependencies: &HashMap<&Identifier, HashSet<Identifier>>,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
    parameter_stack: &mut Stack<Identifier>,
    mut resolved_parameters: ParameterCollectionBuilder,
    mut visited: HashSet<Identifier>,
) -> (
    ParameterCollectionBuilder,
    HashSet<Identifier>,
    // this may be a resolution error source that does not yet have an identifier
    // resolution errors with an identifier are stored in the parameter collection builder
    Result<(), Vec<ParameterResolutionError>>,
) {
    // check that the parameter exists
    if !parameter_map.contains_key(parameter_identifier) {
        return (
            resolved_parameters,
            visited,
            // This is technically a resolution error. However, this error will
            // be caught later when the variable is resolved. In order to avoid
            // duplicate errors, we return Ok(()) and let the variable resolution
            // handle the "not found" error
            Ok(()),
        );
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
        return (
            resolved_parameters,
            visited,
            Err(vec![ParameterResolutionError::circular_dependency(
                circular_dependency,
            )]),
        );
    }

    // check if the parameter has already been visited
    if visited.contains(parameter_identifier) {
        return (resolved_parameters, visited, Ok(()));
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
            let (mut resolved_parameters, visited, result) = resolve_parameter(
                dependency,
                parameter_map,
                dependencies,
                submodel_info,
                module_info,
                parameter_stack,
                resolved_parameters,
                visited,
            );
            match result {
                Ok(()) => (resolved_parameters, visited),
                Err(errors) => {
                    resolved_parameters.add_error_list(parameter_identifier.clone(), errors);
                    (resolved_parameters, visited)
                }
            }
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

    let value =
        resolve_parameter_value(value, &defined_parameters_info, submodel_info, module_info);

    let limits = resolve_limits(
        limits.as_ref(),
        &defined_parameters_info,
        submodel_info,
        module_info,
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

    (resolved_parameters, visited, Ok(()))
}

fn resolve_parameter_value(
    value: &ast::parameter::ParameterValue,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> Result<ParameterValue, Vec<ParameterResolutionError>> {
    let local_variables = HashSet::new();

    match value {
        ast::parameter::ParameterValue::Simple(expr, unit) => {
            let expr = resolve_expr(
                expr,
                &local_variables,
                defined_parameters_info,
                submodel_info,
                module_info,
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
                    module_info,
                )
                .map_err(error::convert_errors);

                let if_expr = resolve_expr(
                    &part.if_expr,
                    &local_variables,
                    defined_parameters_info,
                    submodel_info,
                    module_info,
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

fn resolve_limits(
    limits: Option<&ast::parameter::Limits>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> Result<oneil_module::parameter::Limits, Vec<ParameterResolutionError>> {
    let local_variables = HashSet::new();
    match limits {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            let min = resolve_expr(
                min,
                &local_variables,
                defined_parameters_info,
                submodel_info,
                module_info,
            )
            .map_err(error::convert_errors);

            let max = resolve_expr(
                max,
                &local_variables,
                defined_parameters_info,
                submodel_info,
                module_info,
            )
            .map_err(error::convert_errors);

            let (min, max) = error::combine_errors(min, max)?;

            Ok(oneil_module::parameter::Limits::continuous(min, max))
        }
        Some(ast::parameter::Limits::Discrete { values }) => {
            let values = values.into_iter().map(|value| {
                resolve_expr(
                    value,
                    &local_variables,
                    defined_parameters_info,
                    submodel_info,
                    module_info,
                )
                .map_err(error::convert_errors)
            });

            let values = error::combine_error_list(values)?;

            Ok(oneil_module::parameter::Limits::discrete(values))
        }
        None => Ok(oneil_module::parameter::Limits::default()),
    }
}
