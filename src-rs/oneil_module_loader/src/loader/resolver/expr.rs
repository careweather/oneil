use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::reference::Identifier;

use crate::{
    error::{self, VariableResolutionError},
    loader::resolver::{ModuleInfo, ParameterInfo, SubmodelInfo, variable::resolve_variable},
};

pub fn resolve_expr(
    value: &ast::Expr,
    local_variables: &HashSet<Identifier>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    module_info: &ModuleInfo,
) -> Result<oneil_module::expr::Expr, Vec<VariableResolutionError>> {
    match value {
        ast::Expr::BinaryOp { op, left, right } => {
            let left = resolve_expr(
                left,
                local_variables,
                defined_parameters_info,
                submodel_info,
                module_info,
            );
            let right = resolve_expr(
                right,
                local_variables,
                defined_parameters_info,
                submodel_info,
                module_info,
            );
            let op = resolve_binary_op(op);

            let (left, right) = error::combine_errors(left, right)?;

            Ok(oneil_module::expr::Expr::binary_op(op, left, right))
        }
        ast::Expr::UnaryOp { op, expr } => {
            let expr = resolve_expr(
                expr,
                local_variables,
                defined_parameters_info,
                submodel_info,
                module_info,
            );
            let op = resolve_unary_op(op);

            match expr {
                Ok(expr) => Ok(oneil_module::expr::Expr::unary_op(op, expr)),
                Err(errors) => Err(errors),
            }
        }
        ast::Expr::FunctionCall { name, args } => {
            let name = resolve_function_name(name);
            let args = args.iter().map(|arg| {
                resolve_expr(
                    arg,
                    local_variables,
                    defined_parameters_info,
                    submodel_info,
                    module_info,
                )
            });

            let args = error::combine_error_list(args)?;

            Ok(oneil_module::expr::Expr::function_call(name, args))
        }
        ast::Expr::Variable(variable) => resolve_variable(
            variable,
            local_variables,
            defined_parameters_info,
            submodel_info,
            module_info,
        )
        .map_err(|error| vec![error]),
        ast::Expr::Literal(literal) => {
            let literal = resolve_literal(literal);
            Ok(oneil_module::expr::Expr::literal(literal))
        }
    }
}

fn resolve_binary_op(op: &ast::expression::BinaryOp) -> oneil_module::expr::BinaryOp {
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

fn resolve_unary_op(op: &ast::expression::UnaryOp) -> oneil_module::expr::UnaryOp {
    match op {
        ast::expression::UnaryOp::Neg => oneil_module::expr::UnaryOp::Neg,
        ast::expression::UnaryOp::Not => oneil_module::expr::UnaryOp::Not,
    }
}

fn resolve_function_name(name: &str) -> oneil_module::expr::FunctionName {
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

fn resolve_literal(literal: &ast::expression::Literal) -> oneil_module::expr::Literal {
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
