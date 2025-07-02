use std::collections::{HashMap, HashSet};

use oneil_ast as ast;
use oneil_module::{
    parameter::{Parameter as ModuleParameter, ParameterCollection, ParameterValue, PiecewiseExpr},
    reference::{Identifier, ModulePath},
    test::{ModelTest, SubmodelTest},
};

use crate::util::{
    Stack,
    builder::{ModuleCollectionBuilder, ParameterCollectionBuilder},
};

pub fn resolve_submodels_and_tests(
    use_models: Vec<ast::declaration::UseModel>,
    module_path: &ModulePath,
    builder: ModuleCollectionBuilder,
) -> (
    HashMap<Identifier, ModulePath>,
    Vec<(ModulePath, Vec<ast::declaration::ModelInput>)>,
    ModuleCollectionBuilder,
) {
    use_models.into_iter().fold(
        (HashMap::new(), Vec::new(), builder),
        |(mut submodels, mut submodel_tests, mut builder), use_model| {
            // get the use model path
            let use_model_path = module_path.get_sibling_path(&use_model.model_name);
            let use_model_path = ModulePath::new(use_model_path);

            // resolve the use model path
            let resolved_use_model_path =
                resolve_module_path(use_model_path.clone(), &use_model.subcomponents, &builder);

            // insert the use model path into the submodels map if it was resolved successfully
            // otherwise, add the error to the builder
            match resolved_use_model_path {
                Ok(resolved_use_model_path) => {
                    let submodel_name = use_model.as_name.as_ref().unwrap_or(
                        use_model
                            .subcomponents
                            .last()
                            .unwrap_or(&use_model.model_name),
                    );

                    submodels.insert(
                        Identifier::new(submodel_name),
                        resolved_use_model_path.clone(),
                    );

                    // store the inputs for the submodel tests
                    // (the inputs are stored in their AST form for now and converted to
                    // the model input type once all the submodels have been resolved)
                    let inputs = use_model.inputs.unwrap_or_default();
                    submodel_tests.push((resolved_use_model_path, inputs));
                }
                Err(error) => {
                    builder.add_error(use_model_path, error);
                    todo!("make this more accurate")
                }
            }

            (submodels, submodel_tests, builder)
        },
    )
}

pub fn resolve_parameters(
    parameters: Vec<ast::Parameter>,
    submodels: &HashMap<Identifier, ModulePath>,
    builder: ModuleCollectionBuilder,
) -> Result<ParameterCollection, ()> {
    // TODO: verify that no duplicate parameters are defined

    let parameter_map: HashMap<Identifier, ast::Parameter> = parameters
        .into_iter()
        .map(|parameter| (Identifier::new(&parameter.name), parameter))
        .collect();

    // note that an 'internal dependency' is a dependency on a parameter
    // that is defined within the current module
    let dependencies = get_all_parameter_internal_dependencies(&parameter_map);

    // TODO: resolve the parameters (resolve dependencies first, think evaluation order)
    let mut resolved_parameters = ParameterCollectionBuilder::new();
    let mut visited = HashSet::new();
    for parameter_identifier in parameter_map.keys() {
        if visited.contains(parameter_identifier) {
            continue;
        }

        let mut parameter_stack = Stack::new();

        (resolved_parameters, visited) = resolve_parameter(
            parameter_identifier,
            &parameter_map,
            &dependencies,
            submodels,
            &builder,
            &mut parameter_stack,
            resolved_parameters,
            visited,
        );
    }

    resolved_parameters.try_into()
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
    submodels: &HashMap<Identifier, ModulePath>,
    builder: &ModuleCollectionBuilder,
    parameter_stack: &mut Stack<Identifier>,
    mut resolved_parameters: ParameterCollectionBuilder,
    mut visited: HashSet<Identifier>,
) -> (ParameterCollectionBuilder, HashSet<Identifier>) {
    // TODO: this is actually not true - if a parameter references an undefined value,
    //       this should be marked as an error
    assert!(
        parameter_map.contains_key(parameter_identifier),
        "parameter '{:?}' not found",
        parameter_identifier
    );

    assert!(
        dependencies.contains_key(parameter_identifier),
        "parameter dependencies for '{:?}' not found",
        parameter_identifier
    );

    // check for circular dependencies
    if parameter_stack.contains(parameter_identifier) {
        resolved_parameters.add_error(parameter_identifier.clone(), todo!("circular dependency"));
        return (resolved_parameters, visited);
    }

    // check if the parameter has already been visited
    if visited.contains(parameter_identifier) {
        return (resolved_parameters, visited);
    }

    // resolve the parameter dependencies
    let parameter_dependencies = dependencies.get(parameter_identifier).unwrap();

    // add the parameter to the stack
    parameter_stack.push(parameter_identifier.clone());

    // resolve the parameter dependencies
    for dependency in parameter_dependencies {
        // TODO: don't bail immediately, bail after all the dependencies have been resolved
        (resolved_parameters, visited) = resolve_parameter(
            dependency,
            parameter_map,
            dependencies,
            submodels,
            builder,
            parameter_stack,
            resolved_parameters,
            visited,
        );
    }

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
    let value =
        convert_parameter_value_to_module_expr(value, &resolved_parameters, submodels, builder);
    let limits =
        convert_limits_to_module_limits(limits.as_ref(), &resolved_parameters, submodels, builder);
    let trace_level = convert_trace_level_to_module_trace_level(trace_level);

    let parameter = ModuleParameter::new(
        parameter_dependencies.clone(),
        ident,
        value,
        limits,
        *is_performance,
        trace_level,
    );

    resolved_parameters.add_parameter(parameter_identifier.clone(), parameter);

    (resolved_parameters, visited)
}

fn convert_parameter_value_to_module_expr(
    value: &ast::parameter::ParameterValue,
    resolved_parameters: &ParameterCollectionBuilder,
    submodels: &HashMap<Identifier, ModulePath>,
    builder: &ModuleCollectionBuilder,
) -> ParameterValue {
    let local_variables = HashSet::new();

    match value {
        ast::parameter::ParameterValue::Simple(expr, unit) => {
            let expr = convert_expr_to_module_expr(
                expr,
                &local_variables,
                resolved_parameters,
                submodels,
                builder,
            );
            let unit = unit.as_ref().map(convert_unit_to_module_unit);
            ParameterValue::simple(expr, unit)
        }

        ast::parameter::ParameterValue::Piecewise(piecewise, unit) => {
            let exprs = piecewise
                .parts
                .iter()
                .map(|part| {
                    let expr = convert_expr_to_module_expr(
                        &part.expr,
                        &local_variables,
                        resolved_parameters,
                        submodels,
                        builder,
                    );
                    let if_expr = convert_expr_to_module_expr(
                        &part.if_expr,
                        &local_variables,
                        resolved_parameters,
                        submodels,
                        builder,
                    );
                    PiecewiseExpr::new(expr, if_expr)
                })
                .collect();
            let unit = unit.as_ref().map(convert_unit_to_module_unit);
            ParameterValue::piecewise(exprs, unit)
        }
    }
}

fn convert_limits_to_module_limits(
    limits: Option<&ast::parameter::Limits>,
    resolved_parameters: &ParameterCollectionBuilder,
    submodels: &HashMap<Identifier, ModulePath>,
    builder: &ModuleCollectionBuilder,
) -> oneil_module::parameter::Limits {
    let local_variables = HashSet::new();
    match limits {
        Some(ast::parameter::Limits::Continuous { min, max }) => {
            let min = convert_expr_to_module_expr(
                min,
                &local_variables,
                resolved_parameters,
                submodels,
                builder,
            );
            let max = convert_expr_to_module_expr(
                max,
                &local_variables,
                resolved_parameters,
                submodels,
                builder,
            );
            oneil_module::parameter::Limits::continuous(min, max)
        }
        Some(ast::parameter::Limits::Discrete { values }) => {
            let values = values
                .into_iter()
                .map(|value| {
                    convert_expr_to_module_expr(
                        value,
                        &local_variables,
                        resolved_parameters,
                        submodels,
                        builder,
                    )
                })
                .collect();

            oneil_module::parameter::Limits::discrete(values)
        }
        None => oneil_module::parameter::Limits::default(),
    }
}

fn convert_trace_level_to_module_trace_level(
    trace_level: &ast::parameter::TraceLevel,
) -> oneil_module::parameter::TraceLevel {
    match trace_level {
        ast::parameter::TraceLevel::None => oneil_module::parameter::TraceLevel::None,
        ast::parameter::TraceLevel::Trace => oneil_module::parameter::TraceLevel::Trace,
        ast::parameter::TraceLevel::Debug => oneil_module::parameter::TraceLevel::Debug,
    }
}

fn convert_expr_to_module_expr(
    value: &ast::Expr,
    local_variables: &HashSet<Identifier>,
    resolved_parameters: &ParameterCollectionBuilder,
    submodels: &HashMap<Identifier, ModulePath>,
    builder: &ModuleCollectionBuilder,
) -> oneil_module::expr::Expr {
    match value {
        ast::Expr::BinaryOp { op, left, right } => {
            let left = convert_expr_to_module_expr(
                left,
                local_variables,
                resolved_parameters,
                submodels,
                builder,
            );
            let right = convert_expr_to_module_expr(
                right,
                local_variables,
                resolved_parameters,
                submodels,
                builder,
            );
            let op = convert_binary_op_to_module_op(op);
            oneil_module::expr::Expr::binary_op(op, left, right)
        }
        ast::Expr::UnaryOp { op, expr } => {
            let expr = convert_expr_to_module_expr(
                expr,
                local_variables,
                resolved_parameters,
                submodels,
                builder,
            );
            let op = convert_unary_op_to_module_op(op);
            oneil_module::expr::Expr::unary_op(op, expr)
        }
        ast::Expr::FunctionCall { name, args } => {
            let name = convert_function_name_to_module_function_name(name);
            let args = args
                .iter()
                .map(|arg| {
                    convert_expr_to_module_expr(
                        arg,
                        local_variables,
                        resolved_parameters,
                        submodels,
                        builder,
                    )
                })
                .collect();
            oneil_module::expr::Expr::function_call(name, args)
        }
        ast::Expr::Variable(variable) => resolve_variable(
            variable,
            local_variables,
            resolved_parameters,
            submodels,
            builder,
        ),
        ast::Expr::Literal(literal) => {
            let literal = convert_literal_to_module_literal(literal);
            oneil_module::expr::Expr::literal(literal)
        }
    }
}

fn convert_binary_op_to_module_op(op: &ast::expression::BinaryOp) -> oneil_module::expr::BinaryOp {
    match op {
        ast::expression::BinaryOp::Add => oneil_module::expr::BinaryOp::Add,
        ast::expression::BinaryOp::Sub => oneil_module::expr::BinaryOp::Sub,
        ast::expression::BinaryOp::TrueSub => oneil_module::expr::BinaryOp::TrueSub,
        ast::expression::BinaryOp::Mul => oneil_module::expr::BinaryOp::Mul,
        ast::expression::BinaryOp::Div => oneil_module::expr::BinaryOp::Div,
        ast::expression::BinaryOp::TrueDiv => oneil_module::expr::BinaryOp::TrueDiv,
        ast::expression::BinaryOp::Mod => oneil_module::expr::BinaryOp::Mod,
        ast::expression::BinaryOp::Pow => oneil_module::expr::BinaryOp::Pow,
        ast::expression::BinaryOp::LessThan => oneil_module::expr::BinaryOp::LessThan,
        ast::expression::BinaryOp::LessThanEq => oneil_module::expr::BinaryOp::LessThanEq,
        ast::expression::BinaryOp::GreaterThan => oneil_module::expr::BinaryOp::GreaterThan,
        ast::expression::BinaryOp::GreaterThanEq => oneil_module::expr::BinaryOp::GreaterThanEq,
        ast::expression::BinaryOp::Eq => oneil_module::expr::BinaryOp::Eq,
        ast::expression::BinaryOp::NotEq => oneil_module::expr::BinaryOp::NotEq,
        ast::expression::BinaryOp::And => oneil_module::expr::BinaryOp::And,
        ast::expression::BinaryOp::Or => oneil_module::expr::BinaryOp::Or,
        ast::expression::BinaryOp::MinMax => oneil_module::expr::BinaryOp::MinMax,
    }
}

fn convert_unary_op_to_module_op(op: &ast::expression::UnaryOp) -> oneil_module::expr::UnaryOp {
    match op {
        ast::expression::UnaryOp::Neg => oneil_module::expr::UnaryOp::Neg,
        ast::expression::UnaryOp::Not => oneil_module::expr::UnaryOp::Not,
    }
}

fn convert_function_name_to_module_function_name(name: &str) -> oneil_module::expr::FunctionName {
    match name {
        "min" => oneil_module::expr::FunctionName::min(),
        "max" => oneil_module::expr::FunctionName::max(),
        "sin" => oneil_module::expr::FunctionName::sin(),
        "cos" => oneil_module::expr::FunctionName::cos(),
        "tan" => oneil_module::expr::FunctionName::tan(),
        "asin" => oneil_module::expr::FunctionName::asin(),
        "acos" => oneil_module::expr::FunctionName::acos(),
        "atan" => oneil_module::expr::FunctionName::atan(),
        "sqrt" => oneil_module::expr::FunctionName::sqrt(),
        "ln" => oneil_module::expr::FunctionName::ln(),
        "log" => oneil_module::expr::FunctionName::log(),
        "log10" => oneil_module::expr::FunctionName::log10(),
        "floor" => oneil_module::expr::FunctionName::floor(),
        "ceiling" => oneil_module::expr::FunctionName::ceiling(),
        "extent" => oneil_module::expr::FunctionName::extent(),
        "range" => oneil_module::expr::FunctionName::range(),
        "abs" => oneil_module::expr::FunctionName::abs(),
        "sign" => oneil_module::expr::FunctionName::sign(),
        "mid" => oneil_module::expr::FunctionName::mid(),
        "strip" => oneil_module::expr::FunctionName::strip(),
        "mnmx" => oneil_module::expr::FunctionName::minmax(),
        _ => oneil_module::expr::FunctionName::imported(name.to_string()),
    }
}

fn convert_literal_to_module_literal(
    literal: &ast::expression::Literal,
) -> oneil_module::expr::Literal {
    match literal {
        ast::expression::Literal::Number(number) => oneil_module::expr::Literal::number(*number),
        ast::expression::Literal::String(string) => {
            oneil_module::expr::Literal::string(string.clone())
        }
        ast::expression::Literal::Boolean(boolean) => {
            oneil_module::expr::Literal::boolean(*boolean)
        }
    }
}

fn convert_unit_to_module_unit(unit: &ast::UnitExpr) -> oneil_module::unit::CompositeUnit {
    let units = convert_unit_recursive(unit, false, Vec::new());
    oneil_module::unit::CompositeUnit::new(units)
}

fn convert_unit_recursive(
    unit: &ast::UnitExpr,
    is_inverse: bool,
    mut units: Vec<oneil_module::unit::Unit>,
) -> Vec<oneil_module::unit::Unit> {
    match unit {
        ast::UnitExpr::BinaryOp { op, left, right } => {
            let units = convert_unit_recursive(left, is_inverse, units);

            let units = match op {
                ast::unit::UnitOp::Multiply => convert_unit_recursive(right, is_inverse, units),
                ast::unit::UnitOp::Divide => convert_unit_recursive(right, !is_inverse, units),
            };
            units
        }
        ast::UnitExpr::Unit {
            identifier,
            exponent,
        } => {
            let exponent = exponent.unwrap_or(1.0);
            let exponent = if is_inverse { -exponent } else { exponent };

            let unit = oneil_module::unit::Unit::new(identifier.clone(), exponent);
            units.push(unit);
            units
        }
    }
}

pub fn resolve_model_tests(
    tests: Vec<ast::Test>,
    builder: ModuleCollectionBuilder,
) -> Vec<ModelTest> {
    todo!()
}

pub fn resolve_submodel_tests(
    submodel_tests: Vec<(ModulePath, Vec<ast::declaration::ModelInput>)>,
    builder: ModuleCollectionBuilder,
) -> Vec<SubmodelTest> {
    todo!()
}

fn resolve_module_path(
    module_path: ModulePath,
    subcomponents: &[String],
    builder: &ModuleCollectionBuilder,
) -> Result<ModulePath, ()> {
    assert!(
        builder.module_has_been_visited(&module_path),
        "module path {:?} has not been visited",
        module_path
    );

    if subcomponents.is_empty() {
        return Ok(module_path);
    }

    let module = builder.get_module(&module_path).ok_or(todo!(
        "I think the module had errors? Not sure how to handle this yet {:?}",
        module_path
    ))?;

    let submodel_name = Identifier::new(subcomponents[0]);
    let submodel_path = module
        .get_submodel(&submodel_name)
        .ok_or(todo!("resolution error"))?
        .clone();

    resolve_module_path(submodel_path, &subcomponents[1..], builder)
}

fn resolve_variable(
    variable: &ast::expression::Variable,
    local_variables: &HashSet<Identifier>,
    resolved_parameters: &ParameterCollectionBuilder,
    submodels: &HashMap<Identifier, ModulePath>,
    builder: &ModuleCollectionBuilder,
) -> oneil_module::expr::Expr {
    match variable {
        ast::expression::Variable::Identifier(identifier) => {
            let identifier = Identifier::new(identifier);
            if local_variables.contains(&identifier) {
                oneil_module::expr::Expr::local_variable(identifier)
            } else if resolved_parameters.has_parameter(&identifier) {
                oneil_module::expr::Expr::parameter_variable(identifier)
            } else {
                todo!("parameter not found {:?}", identifier)
            }
        }
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent);
            let submodel_path = submodels
                .get(&parent_identifier)
                .unwrap_or(todo!("submodel not found {:?}", parent_identifier));

            resolve_variable_recursive(submodel_path, component, builder)
        }
    }
}

fn resolve_variable_recursive(
    submodel_path: &ModulePath,
    variable: &ast::expression::Variable,
    builder: &ModuleCollectionBuilder,
) -> oneil_module::expr::Expr {
    let module = builder
        .get_module(submodel_path)
        .unwrap_or(todo!("submodel not found {:?}", submodel_path));

    match variable {
        ast::expression::Variable::Identifier(identifier) => {
            let identifier = Identifier::new(identifier);
            if module.get_parameter(&identifier).is_some() {
                oneil_module::expr::Expr::parameter_variable(identifier)
            } else {
                todo!(
                    "parameter {:?} not found in model {:?}",
                    identifier,
                    submodel_path
                )
            }
        }
        ast::expression::Variable::Accessor { parent, component } => {
            let parent_identifier = Identifier::new(parent);
            let submodel_path = module.get_submodel(&parent_identifier).unwrap_or(todo!(
                "submodel not found {:?} in model {:?}",
                parent_identifier,
                submodel_path
            ));

            resolve_variable_recursive(submodel_path, component, builder)
        }
    }
}
