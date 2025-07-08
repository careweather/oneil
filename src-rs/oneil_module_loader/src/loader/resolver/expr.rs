//! Expression resolution for the Oneil module loader
//!
//! This module provides functionality for resolving AST expressions into module expressions.
//! Expression resolution involves converting abstract syntax tree expressions into
//! executable module expressions while handling variable resolution, function calls,
//! and literal values.
//!
//! # Overview
//!
//! The expression resolver transforms AST expressions into module expressions that can be
//! evaluated within the Oneil module system. This includes:
//!
//! - **Binary Operations**: Arithmetic, comparison, and logical operations
//! - **Unary Operations**: Negation and logical NOT operations
//! - **Function Calls**: Built-in functions and imported functions
//! - **Variables**: Local variables, parameters, and submodel accessors
//! - **Literals**: Numbers, strings, and boolean values
//!
//! # Expression Types
//!
//! ## Binary Operations
//! Supports all standard arithmetic, comparison, and logical operations:
//! - Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
//! - Comparison: `<`, `<=`, `>`, `>=`, `==`, `!=`
//! - Logical: `&&`, `||`, `minmax`
//!
//! ## Unary Operations
//! Supports negation and logical NOT:
//! - `-` for numeric negation
//! - `!` for logical NOT
//!
//! ## Function Calls
//! Handles both built-in functions and imported functions:
//! - **Built-in**: `min`, `max`, `sin`, `cos`, `tan`, `sqrt`, `ln`, `log`, etc.
//! - **Imported**: Any function name not in the built-in list
//!
//! ## Variables
//! Resolves variables through the variable resolution system:
//! - **Local variables**: Test inputs, function parameters
//! - **Parameters**: Module parameters
//! - **Submodel accessors**: `parameter.submodel` notation
//!
//! # Error Handling
//!
//! The module provides comprehensive error handling for various failure scenarios:
//! - Variable resolution errors (undefined variables, submodels with errors)
//! - Expression evaluation errors
//! - Function call errors
//!
//! All errors are collected and returned rather than causing the function to
//! fail immediately.

use std::collections::HashSet;

use oneil_ast as ast;
use oneil_module::reference::Identifier;

use crate::{
    error::{self, VariableResolutionError},
    loader::resolver::{ModuleInfo, ParameterInfo, SubmodelInfo, variable::resolve_variable},
};

/// Resolves an AST expression into a module expression.
///
/// This function transforms an abstract syntax tree expression into a module expression
/// that can be evaluated within the Oneil module system. The resolution process handles
/// variable lookup, function name resolution, and expression structure conversion.
///
/// # Arguments
///
/// * `value` - The AST expression to resolve
/// * `local_variables` - Set of local variable identifiers available in the current scope
/// * `defined_parameters_info` - Information about defined parameters and their status
/// * `submodel_info` - Information about available submodels and their paths
/// * `module_info` - Information about all available modules and their loading status
///
/// # Returns
///
/// * `Ok(oneil_module::expr::Expr)` - The resolved module expression
/// * `Err(Vec<VariableResolutionError>)` - Any variable resolution errors that occurred
///

///
/// # Error Conditions
///
/// The function may return errors in the following cases:
/// - **Variable resolution failures**: When variables cannot be resolved
/// - **Submodel access errors**: When submodel paths are invalid
/// - **Parameter errors**: When parameters have resolution errors
///
/// All errors are collected and returned rather than causing the function to fail
/// immediately.
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

/// Converts an AST binary operation to a module binary operation.
///
/// This function maps AST binary operations to their corresponding module binary operations.
/// All AST binary operations have direct equivalents in the module system.
///
/// # Arguments
///
/// * `op` - The AST binary operation to convert
///
/// # Returns
///
/// The corresponding module binary operation
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

/// Converts an AST unary operation to a module unary operation.
///
/// This function maps AST unary operations to their corresponding module unary operations.
/// Currently supports negation and logical NOT operations.
///
/// # Arguments
///
/// * `op` - The AST unary operation to convert
///
/// # Returns
///
/// The corresponding module unary operation
fn resolve_unary_op(op: &ast::expression::UnaryOp) -> oneil_module::expr::UnaryOp {
    match op {
        ast::expression::UnaryOp::Neg => oneil_module::expr::UnaryOp::Neg,
        ast::expression::UnaryOp::Not => oneil_module::expr::UnaryOp::Not,
    }
}

/// Resolves a function name to a module function name.
///
/// This function determines whether a function name refers to a built-in function
/// or an imported function. Built-in functions are mapped to their corresponding
/// enum variants, while other names are treated as imported functions.
///
/// # Arguments
///
/// * `name` - The function name to resolve
///
/// # Returns
///
/// A module function name representing either a built-in or imported function
///
/// # Built-in Functions
///
/// The following functions are recognized as built-in:
/// - **Mathematical**: `min`, `max`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`
/// - **Logarithmic**: `sqrt`, `ln`, `log`, `log10`
/// - **Rounding**: `floor`, `ceiling`
/// - **Utility**: `extent`, `range`, `abs`, `sign`, `mid`, `strip`, `mnmx`
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

/// Converts an AST literal to a module literal.
///
/// This function maps AST literals to their corresponding module literals.
/// Supports numbers, strings, and boolean values.
///
/// # Arguments
///
/// * `literal` - The AST literal to convert
///
/// # Returns
///
/// The corresponding module literal
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

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast as ast;
    use oneil_module::{
        expr::{BinaryOp, FunctionName, Literal, UnaryOp},
        reference::Identifier,
    };
    use std::collections::{HashMap, HashSet};

    /// Helper function to create basic test data structures for tests that
    /// don't rely on any context.
    fn create_empty_context() -> (
        HashSet<Identifier>,
        ParameterInfo<'static>,
        SubmodelInfo<'static>,
        ModuleInfo<'static>,
    ) {
        let local_vars = HashSet::new();
        let param_info = ParameterInfo::new(HashMap::new(), HashSet::new());
        let submodel_info = SubmodelInfo::new(HashMap::new(), HashSet::new());
        let module_info = ModuleInfo::new(HashMap::new(), HashSet::new());

        (local_vars, param_info, submodel_info, module_info)
    }

    #[test]
    fn test_resolve_literal_number() {
        // create the expression
        let literal = ast::Expr::Literal(ast::expression::Literal::Number(42.0));

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::Literal { value }) => {
                assert!(matches!(value, Literal::Number(42.0)));
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_literal_string() {
        // create the expression
        let literal = ast::Expr::Literal(ast::expression::Literal::String("hello".to_string()));

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::Literal { value }) => {
                assert_eq!(value, Literal::String("hello".to_string()));
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_literal_boolean() {
        // create the expression
        let literal = ast::Expr::Literal(ast::expression::Literal::Boolean(true));

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::Literal { value }) => {
                assert_eq!(value, Literal::Boolean(true));
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_binary_op() {
        // create the expression
        let expr = ast::Expr::BinaryOp {
            op: ast::expression::BinaryOp::Add,
            left: Box::new(ast::Expr::Literal(ast::expression::Literal::Number(1.0))),
            right: Box::new(ast::Expr::Literal(ast::expression::Literal::Number(2.0))),
        };

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::BinaryOp { op, left, right }) => {
                assert_eq!(op, BinaryOp::Add);
                assert_eq!(
                    *left,
                    oneil_module::expr::Expr::Literal {
                        value: Literal::Number(1.0)
                    }
                );
                assert_eq!(
                    *right,
                    oneil_module::expr::Expr::Literal {
                        value: Literal::Number(2.0)
                    }
                );
            }
            _ => panic!("Expected binary operation, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_unary_op() {
        // create the expression
        let expr = ast::Expr::UnaryOp {
            op: ast::expression::UnaryOp::Neg,
            expr: Box::new(ast::Expr::Literal(ast::expression::Literal::Number(5.0))),
        };

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::UnaryOp { op, expr }) => {
                assert_eq!(op, UnaryOp::Neg);
                assert_eq!(
                    *expr,
                    oneil_module::expr::Expr::Literal {
                        value: Literal::Number(5.0),
                    }
                );
            }
            _ => panic!("Expected unary operation, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_function_call_builtin() {
        // create the expression
        let expr = ast::Expr::FunctionCall {
            name: "sin".to_string(),
            args: vec![ast::Expr::Literal(ast::expression::Literal::Number(3.14))],
        };

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::FunctionCall { name, args }) => {
                assert_eq!(name, FunctionName::sin());
                assert_eq!(args.len(), 1);
                assert_eq!(
                    args[0],
                    oneil_module::expr::Expr::Literal {
                        value: Literal::Number(3.14),
                    }
                );
            }
            _ => panic!("Expected function call, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_function_call_imported() {
        // create the expression
        let expr = ast::Expr::FunctionCall {
            name: "custom_function".to_string(),
            args: vec![ast::Expr::Literal(ast::expression::Literal::Number(42.0))],
        };

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::FunctionCall { name, args }) => {
                assert_eq!(name, FunctionName::imported("custom_function".to_string()));
                assert_eq!(args.len(), 1);
                assert_eq!(
                    args[0],
                    oneil_module::expr::Expr::Literal {
                        value: Literal::Number(42.0),
                    }
                );
            }
            _ => panic!("Expected function call, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_variable_local() {
        // create the expression
        let expr = ast::Expr::Variable(ast::expression::Variable::Identifier("x".to_string()));

        // create the context
        let (_, param_info, submodel_info, module_info) = create_empty_context();
        let local_vars = HashSet::from([Identifier::new("x")]);

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Local(ident))) => {
                assert_eq!(ident, Identifier::new("x"));
            }
            _ => panic!("Expected local variable, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_variable_parameter() {
        // create the expression
        let expr = ast::Expr::Variable(ast::expression::Variable::Identifier("param".to_string()));

        // create the context
        let mut param_map = HashMap::new();
        let param_id = Identifier::new("param");
        let parameter = oneil_module::parameter::Parameter::new(
            HashSet::new(),
            param_id.clone(),
            oneil_module::parameter::ParameterValue::simple(
                oneil_module::expr::Expr::literal(oneil_module::expr::Literal::number(42.0)),
                None,
            ),
            oneil_module::parameter::Limits::default(),
            false,
            oneil_module::debug_info::TraceLevel::None,
        );
        param_map.insert(&param_id, &parameter);
        let param_info = ParameterInfo::new(param_map, HashSet::new());
        let (local_vars, _, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::Variable(oneil_module::expr::Variable::Parameter(
                ident,
            ))) => {
                assert_eq!(ident, Identifier::new("param"));
            }
            _ => panic!("Expected parameter variable, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_variable_undefined() {
        // create the expression
        let expr = ast::Expr::Variable(ast::expression::Variable::Identifier(
            "undefined".to_string(),
        ));

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter(None, ident) => {
                        assert_eq!(ident, &Identifier::new("undefined"));
                    }
                    _ => panic!("Expected undefined parameter error, got {:?}", errors[0]),
                }
            }
            _ => panic!("Expected error, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_complex_expression() {
        // create the expression: (1 + 2) * sin(3.14)
        let inner_binary = ast::Expr::BinaryOp {
            op: ast::expression::BinaryOp::Add,
            left: Box::new(ast::Expr::Literal(ast::expression::Literal::Number(1.0))),
            right: Box::new(ast::Expr::Literal(ast::expression::Literal::Number(2.0))),
        };

        let func_call = ast::Expr::FunctionCall {
            name: "sin".to_string(),
            args: vec![ast::Expr::Literal(ast::expression::Literal::Number(3.14))],
        };

        let expr = ast::Expr::BinaryOp {
            op: ast::expression::BinaryOp::Mul,
            left: Box::new(inner_binary),
            right: Box::new(func_call),
        };

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Ok(oneil_module::expr::Expr::BinaryOp { op, left, right }) => {
                assert_eq!(op, BinaryOp::Mul);

                // check left side (1 + 2)
                match *left {
                    oneil_module::expr::Expr::BinaryOp {
                        op: left_op,
                        left: left_left,
                        right: left_right,
                    } => {
                        assert_eq!(left_op, BinaryOp::Add);
                        assert_eq!(
                            *left_left,
                            oneil_module::expr::Expr::Literal {
                                value: Literal::Number(1.0)
                            }
                        );
                        assert_eq!(
                            *left_right,
                            oneil_module::expr::Expr::Literal {
                                value: Literal::Number(2.0)
                            }
                        );
                    }
                    _ => panic!("Expected binary operation on left, got {:?}", left),
                }

                // check right side (sin(3.14))
                match *right {
                    oneil_module::expr::Expr::FunctionCall { name, args } => {
                        assert_eq!(name, FunctionName::sin());
                        assert_eq!(args.len(), 1);
                        assert_eq!(
                            args[0],
                            oneil_module::expr::Expr::Literal {
                                value: Literal::Number(3.14)
                            }
                        );
                    }
                    _ => panic!("Expected function call on right, got {:?}", right),
                }
            }
            _ => panic!("Expected binary operation, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_binary_op_all_operations() {
        // create the operations
        let operations = vec![
            (ast::expression::BinaryOp::Add, BinaryOp::Add),
            (ast::expression::BinaryOp::Sub, BinaryOp::Sub),
            (ast::expression::BinaryOp::TrueSub, BinaryOp::TrueSub),
            (ast::expression::BinaryOp::Mul, BinaryOp::Mul),
            (ast::expression::BinaryOp::Div, BinaryOp::Div),
            (ast::expression::BinaryOp::TrueDiv, BinaryOp::TrueDiv),
            (ast::expression::BinaryOp::Mod, BinaryOp::Mod),
            (ast::expression::BinaryOp::Pow, BinaryOp::Pow),
            (ast::expression::BinaryOp::LessThan, BinaryOp::LessThan),
            (ast::expression::BinaryOp::LessThanEq, BinaryOp::LessThanEq),
            (
                ast::expression::BinaryOp::GreaterThan,
                BinaryOp::GreaterThan,
            ),
            (
                ast::expression::BinaryOp::GreaterThanEq,
                BinaryOp::GreaterThanEq,
            ),
            (ast::expression::BinaryOp::Eq, BinaryOp::Eq),
            (ast::expression::BinaryOp::NotEq, BinaryOp::NotEq),
            (ast::expression::BinaryOp::And, BinaryOp::And),
            (ast::expression::BinaryOp::Or, BinaryOp::Or),
            (ast::expression::BinaryOp::MinMax, BinaryOp::MinMax),
        ];

        for (ast_op, expected_module_op) in operations {
            // resolve the binary operation
            let result = resolve_binary_op(&ast_op);

            // check the result
            assert_eq!(result, expected_module_op);
        }
    }

    #[test]
    fn test_resolve_unary_op_all_operations() {
        // create the operations
        let operations = vec![
            (ast::expression::UnaryOp::Neg, UnaryOp::Neg),
            (ast::expression::UnaryOp::Not, UnaryOp::Not),
        ];

        for (ast_op, expected_module_op) in operations {
            // resolve the unary operation
            let result = resolve_unary_op(&ast_op);

            // check the result
            assert_eq!(result, expected_module_op);
        }
    }

    #[test]
    fn test_resolve_function_name_builtin() {
        // create the builtin functions
        let builtin_functions = vec![
            ("min", FunctionName::min()),
            ("max", FunctionName::max()),
            ("sin", FunctionName::sin()),
            ("cos", FunctionName::cos()),
            ("tan", FunctionName::tan()),
            ("asin", FunctionName::asin()),
            ("acos", FunctionName::acos()),
            ("atan", FunctionName::atan()),
            ("sqrt", FunctionName::sqrt()),
            ("ln", FunctionName::ln()),
            ("log", FunctionName::log()),
            ("log10", FunctionName::log10()),
            ("floor", FunctionName::floor()),
            ("ceiling", FunctionName::ceiling()),
            ("extent", FunctionName::extent()),
            ("range", FunctionName::range()),
            ("abs", FunctionName::abs()),
            ("sign", FunctionName::sign()),
            ("mid", FunctionName::mid()),
            ("strip", FunctionName::strip()),
            ("mnmx", FunctionName::minmax()),
        ];

        // resolve the function names
        for (func_name, expected_func_builtin) in builtin_functions {
            let result = resolve_function_name(func_name);

            // check the result
            assert_eq!(result, expected_func_builtin);
        }
    }

    #[test]
    fn test_resolve_function_name_imported() {
        // create the function names
        let imported_functions = vec![
            "custom_function",
            "my_special_function",
            "external_lib_function",
            "user_defined_function",
        ];

        // resolve the function names
        for func_name in imported_functions {
            let result = resolve_function_name(func_name);

            // check the result
            match result {
                FunctionName::Imported(name) => {
                    assert_eq!(name, func_name);
                }
                _ => panic!(
                    "Expected imported function for '{}', got {:?}",
                    func_name, result
                ),
            }
        }
    }

    #[test]
    fn test_resolve_literal_all_types() {
        // Test number
        let ast_number = ast::expression::Literal::Number(42.5);
        let module_number = resolve_literal(&ast_number);
        assert_eq!(module_number, Literal::Number(42.5));

        // Test string
        let ast_string = ast::expression::Literal::String("test string".to_string());
        let module_string = resolve_literal(&ast_string);
        assert_eq!(module_string, Literal::String("test string".to_string()));

        // Test boolean
        let ast_bool = ast::expression::Literal::Boolean(false);
        let module_bool = resolve_literal(&ast_bool);
        assert_eq!(module_bool, Literal::Boolean(false));
    }

    #[test]
    fn test_resolve_expression_with_errors() {
        // create the expression
        let expr = ast::Expr::BinaryOp {
            op: ast::expression::BinaryOp::Add,
            left: Box::new(ast::Expr::Variable(ast::expression::Variable::Identifier(
                "undefined1".to_string(),
            ))),
            right: Box::new(ast::Expr::Variable(ast::expression::Variable::Identifier(
                "undefined2".to_string(),
            ))),
        };

        // create the context
        let (local_vars, param_info, submodel_info, module_info) = create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &expr,
            &local_vars,
            &param_info,
            &submodel_info,
            &module_info,
        );

        // check the result
        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 2);

                let error_identifiers: Vec<_> = errors
                    .iter()
                    .filter_map(|e| {
                        if let VariableResolutionError::UndefinedParameter(None, ident) = e {
                            Some(ident.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                assert!(error_identifiers.contains(&Identifier::new("undefined1")));
                assert!(error_identifiers.contains(&Identifier::new("undefined2")));
            }
            _ => panic!("Expected error, got {:?}", result),
        }
    }
}
