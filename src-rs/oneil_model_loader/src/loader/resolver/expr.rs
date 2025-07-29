//! Expression resolution for the Oneil model loader
//!
//! This module provides functionality for resolving AST expressions into model expressions.
//! Expression resolution involves converting abstract syntax tree expressions into
//! executable model expressions while handling variable resolution, function calls,
//! and literal values.
//!
//! # Overview
//!
//! The expression resolver transforms AST expressions into model expressions that can be
//! evaluated within the Oneil model system. This includes:
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
//! - **Parameters**: Model parameters
//! - **Submodel accessors**: `parameter.submodel` notation
//!
//! # Error Handling
//!
//! The model provides comprehensive error handling for various failure scenarios:
//! - Variable resolution errors (undefined variables, submodels with errors)
//! - Expression evaluation errors
//! - Function call errors
//!
//! All errors are collected and returned rather than causing the function to
//! fail immediately.

use std::collections::HashSet;

use oneil_ast as ast;
use oneil_ir::{expr::Expr, reference::Identifier, span::WithSpan};

use crate::{
    error::{self, VariableResolutionError},
    loader::resolver::{ModelInfo, ParameterInfo, SubmodelInfo, variable::resolve_variable},
    util::get_span_from_ast_span,
};

/// Resolves an AST expression into a model expression.
///
/// This function transforms an abstract syntax tree expression into a model expression
/// that can be evaluated within the Oneil model system. The resolution process handles
/// variable lookup, function name resolution, and expression structure conversion.
///
/// # Arguments
///
/// * `value` - The AST expression to resolve
/// * `local_variables` - Set of local variable identifiers available in the current scope
/// * `defined_parameters_info` - Information about defined parameters and their status
/// * `submodel_info` - Information about available submodels and their paths
/// * `model_info` - Information about all available models and their loading status
///
/// # Returns
///
/// * `Ok(oneil_ir::expr::Expr)` - The resolved model expression
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
    value: &ast::expression::ExprNode,
    local_variables: &HashSet<Identifier>,
    defined_parameters_info: &ParameterInfo,
    submodel_info: &SubmodelInfo,
    model_info: &ModelInfo,
) -> Result<oneil_ir::expr::ExprWithSpan, Vec<VariableResolutionError>> {
    let value_span = get_span_from_ast_span(value.node_span());
    match value.node_value() {
        ast::Expr::BinaryOp { op, left, right } => {
            let left = resolve_expr(
                left,
                local_variables,
                defined_parameters_info,
                submodel_info,
                model_info,
            );
            let right = resolve_expr(
                right,
                local_variables,
                defined_parameters_info,
                submodel_info,
                model_info,
            );
            let op_with_span = resolve_binary_op(op);

            let (left, right) = error::combine_errors(left, right)?;

            let expr = Expr::binary_op(op_with_span, left, right);
            Ok(WithSpan::new(expr, value_span))
        }
        ast::Expr::UnaryOp { op, expr } => {
            let expr = resolve_expr(
                expr,
                local_variables,
                defined_parameters_info,
                submodel_info,
                model_info,
            );
            let op_with_span = resolve_unary_op(op);

            match expr {
                Ok(expr) => Ok(WithSpan::new(
                    Expr::unary_op(op_with_span, expr),
                    value_span,
                )),
                Err(errors) => Err(errors),
            }
        }
        ast::Expr::FunctionCall { name, args } => {
            let name_with_span = resolve_function_name(name);
            let args = args.iter().map(|arg| {
                resolve_expr(
                    arg,
                    local_variables,
                    defined_parameters_info,
                    submodel_info,
                    model_info,
                )
            });

            let args = error::combine_error_list(args)?;

            let expr = Expr::function_call(name_with_span, args);
            Ok(WithSpan::new(expr, value_span))
        }
        ast::Expr::Variable(variable) => resolve_variable(
            variable,
            local_variables,
            defined_parameters_info,
            submodel_info,
            model_info,
        )
        .map_err(|error| vec![error]),
        ast::Expr::Literal(literal) => {
            let literal = resolve_literal(literal);
            let expr = Expr::literal(literal);
            Ok(WithSpan::new(expr, value_span))
        }
        ast::Expr::Parenthesized { expr } => resolve_expr(
            expr,
            local_variables,
            defined_parameters_info,
            submodel_info,
            model_info,
        ),
    }
}

/// Converts an AST binary operation to a model binary operation.
///
/// This function maps AST binary operations to their corresponding model binary operations.
/// All AST binary operations have direct equivalents in the model system.
///
/// # Arguments
///
/// * `op` - The AST binary operation to convert
///
/// # Returns
///
/// The corresponding model binary operation
fn resolve_binary_op(op: &ast::expression::BinaryOpNode) -> WithSpan<oneil_ir::expr::BinaryOp> {
    let op_value = match op.node_value() {
        ast::expression::BinaryOp::Add => oneil_ir::expr::BinaryOp::Add,
        ast::expression::BinaryOp::Sub => oneil_ir::expr::BinaryOp::Sub,
        ast::expression::BinaryOp::TrueSub => oneil_ir::expr::BinaryOp::TrueSub,
        ast::expression::BinaryOp::Mul => oneil_ir::expr::BinaryOp::Mul,
        ast::expression::BinaryOp::Div => oneil_ir::expr::BinaryOp::Div,
        ast::expression::BinaryOp::TrueDiv => oneil_ir::expr::BinaryOp::TrueDiv,
        ast::expression::BinaryOp::Mod => oneil_ir::expr::BinaryOp::Mod,
        ast::expression::BinaryOp::Pow => oneil_ir::expr::BinaryOp::Pow,
        ast::expression::BinaryOp::LessThan => oneil_ir::expr::BinaryOp::LessThan,
        ast::expression::BinaryOp::LessThanEq => oneil_ir::expr::BinaryOp::LessThanEq,
        ast::expression::BinaryOp::GreaterThan => oneil_ir::expr::BinaryOp::GreaterThan,
        ast::expression::BinaryOp::GreaterThanEq => oneil_ir::expr::BinaryOp::GreaterThanEq,
        ast::expression::BinaryOp::Eq => oneil_ir::expr::BinaryOp::Eq,
        ast::expression::BinaryOp::NotEq => oneil_ir::expr::BinaryOp::NotEq,
        ast::expression::BinaryOp::And => oneil_ir::expr::BinaryOp::And,
        ast::expression::BinaryOp::Or => oneil_ir::expr::BinaryOp::Or,
        ast::expression::BinaryOp::MinMax => oneil_ir::expr::BinaryOp::MinMax,
    };
    let op_span = get_span_from_ast_span(op.node_span());
    WithSpan::new(op_value, op_span)
}

/// Converts an AST unary operation to a model unary operation.
///
/// This function maps AST unary operations to their corresponding model unary operations.
/// Currently supports negation and logical NOT operations.
///
/// # Arguments
///
/// * `op` - The AST unary operation to convert
///
/// # Returns
///
/// The corresponding model unary operation
fn resolve_unary_op(op: &ast::expression::UnaryOpNode) -> WithSpan<oneil_ir::expr::UnaryOp> {
    let op_value = match op.node_value() {
        ast::expression::UnaryOp::Neg => oneil_ir::expr::UnaryOp::Neg,
        ast::expression::UnaryOp::Not => oneil_ir::expr::UnaryOp::Not,
    };
    let op_span = get_span_from_ast_span(op.node_span());
    WithSpan::new(op_value, op_span)
}

/// Resolves a function name to a model function name.
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
/// A model function name representing either a built-in or imported function
///
/// # Built-in Functions
///
/// The following functions are recognized as built-in:
/// - **Mathematical**: `min`, `max`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`
/// - **Logarithmic**: `sqrt`, `ln`, `log`, `log10`
/// - **Rounding**: `floor`, `ceiling`
/// - **Utility**: `extent`, `range`, `abs`, `sign`, `mid`, `strip`, `mnmx`
fn resolve_function_name(
    name: &ast::naming::IdentifierNode,
) -> WithSpan<oneil_ir::expr::FunctionName> {
    let span = get_span_from_ast_span(name.node_span());
    let name = match name.as_str() {
        "min" => oneil_ir::expr::FunctionName::min(),
        "max" => oneil_ir::expr::FunctionName::max(),
        "sin" => oneil_ir::expr::FunctionName::sin(),
        "cos" => oneil_ir::expr::FunctionName::cos(),
        "tan" => oneil_ir::expr::FunctionName::tan(),
        "asin" => oneil_ir::expr::FunctionName::asin(),
        "acos" => oneil_ir::expr::FunctionName::acos(),
        "atan" => oneil_ir::expr::FunctionName::atan(),
        "sqrt" => oneil_ir::expr::FunctionName::sqrt(),
        "ln" => oneil_ir::expr::FunctionName::ln(),
        "log" => oneil_ir::expr::FunctionName::log(),
        "log10" => oneil_ir::expr::FunctionName::log10(),
        "floor" => oneil_ir::expr::FunctionName::floor(),
        "ceiling" => oneil_ir::expr::FunctionName::ceiling(),
        "extent" => oneil_ir::expr::FunctionName::extent(),
        "range" => oneil_ir::expr::FunctionName::range(),
        "abs" => oneil_ir::expr::FunctionName::abs(),
        "sign" => oneil_ir::expr::FunctionName::sign(),
        "mid" => oneil_ir::expr::FunctionName::mid(),
        "strip" => oneil_ir::expr::FunctionName::strip(),
        "mnmx" => oneil_ir::expr::FunctionName::minmax(),
        name => oneil_ir::expr::FunctionName::imported(name.to_string()),
    };
    WithSpan::new(name, span)
}

/// Converts an AST literal to a model literal.
///
/// This function maps AST literals to their corresponding model literals.
/// Supports numbers, strings, and boolean values.
///
/// # Arguments
///
/// * `literal` - The AST literal to convert
///
/// # Returns
///
/// The corresponding model literal
fn resolve_literal(literal: &ast::expression::LiteralNode) -> oneil_ir::expr::Literal {
    match literal.node_value() {
        ast::expression::Literal::Number(number) => oneil_ir::expr::Literal::number(*number),
        ast::expression::Literal::String(string) => oneil_ir::expr::Literal::string(string.clone()),
        ast::expression::Literal::Boolean(boolean) => oneil_ir::expr::Literal::boolean(*boolean),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast as ast;
    use oneil_ir as ir;
    use oneil_ir::span::Span;
    use oneil_ir::{
        expr::{BinaryOp, FunctionName, Literal, UnaryOp},
        reference::Identifier,
    };
    use std::collections::{HashMap, HashSet};

    mod helper {
        use super::*;

        /// Helper function to create basic test data structures for tests that
        /// don't rely on any context.
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

        /// Helper function to create a binary operation node
        pub fn create_binary_op_node(
            op: ast::expression::BinaryOp,
            start: usize,
            end: usize,
        ) -> ast::expression::BinaryOpNode {
            let op_node = ast::node::Node::new(test_span(start, end), op);
            op_node
        }

        /// Helper function to create a binary operation expression node
        pub fn create_binary_op_expr_node(
            left: ast::expression::ExprNode,
            op: ast::expression::BinaryOpNode,
            right: ast::expression::ExprNode,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr = ast::expression::Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
            ast::node::Node::new(test_span(start, end), expr)
        }

        /// Helper function to create a unary operation node
        pub fn create_unary_op_node(
            op: ast::expression::UnaryOp,
            start: usize,
            end: usize,
        ) -> ast::expression::UnaryOpNode {
            let op_node = ast::node::Node::new(test_span(start, end), op);
            op_node
        }

        /// Helper function to create an identifier node
        pub fn create_identifier_node(
            name: &str,
            start: usize,
            end: usize,
        ) -> ast::naming::IdentifierNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            ast::node::Node::new(test_span(start, end), identifier)
        }

        /// Helper function to create a unary operation expression node
        pub fn create_unary_op_expr_node(
            op: ast::expression::UnaryOpNode,
            expr: ast::expression::ExprNode,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr_node = ast::expression::Expr::UnaryOp {
                op,
                expr: Box::new(expr),
            };
            ast::node::Node::new(test_span(start, end), expr_node)
        }

        /// Helper function to create a function call expression node
        pub fn create_function_call_expr_node(
            name: ast::naming::IdentifierNode,
            args: Vec<ast::expression::ExprNode>,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr = ast::expression::Expr::FunctionCall { name, args };
            ast::node::Node::new(test_span(start, end), expr)
        }

        /// Helper function to create a simple identifier variable
        pub fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            let identifier_node = ast::node::Node::new(test_span(0, name.len()), identifier);
            let variable = ast::expression::Variable::Identifier(identifier_node);
            ast::node::Node::new(test_span(0, name.len()), variable)
        }

        /// Helper function to create a parenthesized expression node
        pub fn create_parenthesized_expr_node(
            expr: ast::expression::ExprNode,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let parenthesized_expr = ast::expression::Expr::Parenthesized {
                expr: Box::new(expr),
            };
            ast::node::Node::new(test_span(start, end), parenthesized_expr)
        }

        pub fn create_empty_context() -> (
            HashSet<Identifier>,
            ParameterInfo<'static>,
            SubmodelInfo<'static>,
            ModelInfo<'static>,
        ) {
            let local_vars = HashSet::new();
            let param_info = ParameterInfo::new(HashMap::new(), HashSet::new());
            let submodel_info = SubmodelInfo::new(HashMap::new(), HashSet::new());
            let model_info = ModelInfo::new(HashMap::new(), HashSet::new());

            (local_vars, param_info, submodel_info, model_info)
        }

        /// Helper function to create a parameter ID with span
        pub fn create_ir_id_with_span(
            name: &str,
            start: usize,
            end: usize,
        ) -> WithSpan<Identifier> {
            WithSpan::new(Identifier::new(name.to_string()), Span::new(start, end))
        }

        /// Helper function to create a parameter value with span
        pub fn create_ir_expr_literal(
            literal: f64,
            start: usize,
            end: usize,
        ) -> ir::expr::ExprWithSpan {
            let literal = ir::expr::Literal::number(literal);
            WithSpan::new(
                ir::expr::Expr::Literal { value: literal },
                Span::new(start, end),
            )
        }

        /// Helper function to create a literal node
        pub fn create_literal_node(
            literal: ast::expression::Literal,
            start: usize,
            end: usize,
        ) -> ast::expression::LiteralNode {
            ast::node::Node::new(test_span(start, end), literal)
        }
    }

    #[test]
    fn test_resolve_literal_number() {
        // create the expression
        let literal =
            helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 0, 4);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &local_vars,
            &param_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(literal.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Number(42.0));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result),
                }
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_literal_string() {
        // create the expression
        let literal = helper::create_literal_expr_node(
            ast::expression::Literal::String("hello".to_string()),
            0,
            5,
        );

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &local_vars,
            &param_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(literal.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::String("hello".to_string()));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result),
                }
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_literal_boolean() {
        // create the expression
        let literal =
            helper::create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &local_vars,
            &param_info,
            &submodel_info,
            &model_info,
        );

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(literal.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Boolean(true));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result),
                }
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_binary_op() {
        // create the expression
        let ast_left =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 0, 1);
        let ast_op = helper::create_binary_op_node(ast::expression::BinaryOp::Add, 1, 2);
        let ast_right =
            helper::create_literal_expr_node(ast::expression::Literal::Number(2.0), 4, 5);
        let expr = helper::create_binary_op_expr_node(
            ast_left.clone(),
            ast_op.clone(),
            ast_right.clone(),
            0,
            5,
        );

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), &expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Add);

                        match left.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal expression on left, got {:?}", left),
                        }

                        match right.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(2.0));
                            }
                            _ => panic!("Expected literal expression on right, got {:?}", right),
                        }
                    }
                    _ => panic!("Expected binary operation, got {:?}", result),
                }
            }
            _ => panic!("Expected binary operation, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_unary_op() {
        // create the expression
        let ast_inner_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 1, 4);
        let ast_op = helper::create_unary_op_node(ast::expression::UnaryOp::Neg, 0, 1);
        let expr = helper::create_unary_op_expr_node(ast_op.clone(), ast_inner_expr.clone(), 0, 4);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);

                match result.value() {
                    ir::expr::Expr::UnaryOp { op, expr } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), &expected_op_span);
                        assert_eq!(op.value(), &UnaryOp::Neg);

                        match expr.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(5.0));
                            }
                            _ => panic!("Expected literal expression, got {:?}", expr),
                        }
                    }
                    _ => panic!("Expected unary operation, got {:?}", result),
                }
            }
            _ => panic!("Expected unary operation, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_function_call_builtin() {
        // create the expression
        let ast_arg =
            helper::create_literal_expr_node(ast::expression::Literal::Number(3.14), 4, 8);
        let ast_name = helper::create_identifier_node("sin", 0, 3);
        let expr =
            helper::create_function_call_expr_node(ast_name.clone(), vec![ast_arg.clone()], 0, 8);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);

                match result.value() {
                    ir::expr::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(ast_name.node_span());
                        assert_eq!(name.span(), &expected_name_span);
                        assert_eq!(name.value(), &FunctionName::sin());

                        assert_eq!(args.len(), 1);

                        match &args[0].value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(3.14));
                            }
                            _ => panic!("Expected literal argument, got {:?}", args[0]),
                        }
                    }
                    _ => panic!("Expected function call, got {:?}", result),
                }
            }
            _ => panic!("Expected function call, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_function_call_imported() {
        // create the expression
        let ast_arg =
            helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 16, 19);
        let ast_name = helper::create_identifier_node("custom_function", 0, 15);
        let expr =
            helper::create_function_call_expr_node(ast_name.clone(), vec![ast_arg.clone()], 0, 19);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);

                match result.value() {
                    ir::expr::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(ast_name.node_span());
                        assert_eq!(name.span(), &expected_name_span);
                        assert_eq!(
                            name.value(),
                            &FunctionName::imported("custom_function".to_string())
                        );

                        assert_eq!(args.len(), 1);

                        match &args[0].value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(42.0));
                            }
                            _ => panic!("Expected literal argument, got {:?}", args[0]),
                        }
                    }
                    _ => panic!("Expected function call, got {:?}", result),
                }
            }
            _ => panic!("Expected function call, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_variable_local() {
        // create the expression
        let ast_variable = helper::create_identifier_variable("x");
        let expr = helper::create_variable_expr_node(ast_variable.clone(), 0, 1);

        // create the context
        let (_, param_info, submodel_info, model_info) = helper::create_empty_context();
        let local_vars = HashSet::from([Identifier::new("x")]);

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);

                match result.value() {
                    ir::expr::Expr::Variable(variable) => {
                        assert_eq!(variable, &ir::expr::Variable::Local(Identifier::new("x")));
                    }
                    _ => panic!("Expected variable expression, got {:?}", result),
                }
            }
            _ => panic!("Expected local variable, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_variable_parameter() {
        // create the expression
        let ast_variable = helper::create_identifier_variable("param");
        let expr = helper::create_variable_expr_node(ast_variable.clone(), 0, 5);

        // create the context
        let mut param_map = HashMap::new();
        let param_id = helper::create_ir_id_with_span("param", 0, 5);
        let param_value = helper::create_ir_expr_literal(42.0, 0, 5);
        let parameter = oneil_ir::parameter::Parameter::new(
            HashSet::new(),
            param_id.clone(),
            oneil_ir::parameter::ParameterValue::simple(param_value, None),
            oneil_ir::parameter::Limits::default(),
            false,
            oneil_ir::debug_info::TraceLevel::None,
        );
        param_map.insert(param_id.value(), &parameter);
        let param_info = ParameterInfo::new(param_map, HashSet::new());
        let (local_vars, _, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);

                match result.value() {
                    ir::expr::Expr::Variable(variable) => {
                        assert_eq!(
                            variable,
                            &ir::expr::Variable::Parameter(Identifier::new("param"))
                        );
                    }
                    _ => panic!("Expected variable expression, got {:?}", result),
                }
            }
            _ => panic!("Expected parameter variable, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_variable_undefined() {
        // create the expression
        let ast_variable = helper::create_identifier_variable("undefined");
        let expr = helper::create_variable_expr_node(ast_variable.clone(), 0, 9);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter(None, ident) => {
                        let span = get_span_from_ast_span(ast_variable.node_span());
                        assert_eq!(ident.span(), &span);
                        assert_eq!(ident.value(), &Identifier::new("undefined"));
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
        let ast_left_1 =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 1, 2);
        let ast_right_1 =
            helper::create_literal_expr_node(ast::expression::Literal::Number(2.0), 5, 6);
        let ast_add_op = helper::create_binary_op_node(ast::expression::BinaryOp::Add, 2, 3);
        let inner_binary = helper::create_binary_op_expr_node(
            ast_left_1.clone(),
            ast_add_op.clone(),
            ast_right_1.clone(),
            0,
            7,
        );

        let ast_func_arg =
            helper::create_literal_expr_node(ast::expression::Literal::Number(3.14), 12, 16);
        let ast_func_name = helper::create_identifier_node("sin", 8, 11);
        let func_call = helper::create_function_call_expr_node(
            ast_func_name.clone(),
            vec![ast_func_arg.clone()],
            8,
            17,
        );

        let ast_mul_op = helper::create_binary_op_node(ast::expression::BinaryOp::Mul, 7, 8);
        let expr = helper::create_binary_op_expr_node(
            inner_binary.clone(),
            ast_mul_op.clone(),
            func_call.clone(),
            0,
            17,
        );

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);

                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_mul_op.node_span());
                        assert_eq!(op.span(), &expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Mul);

                        // check left side (1 + 2)
                        let expected_left_span = get_span_from_ast_span(inner_binary.node_span());
                        assert_eq!(left.span(), &expected_left_span);
                        match left.value() {
                            ir::expr::Expr::BinaryOp {
                                op: left_op,
                                left: left_left,
                                right: left_right,
                            } => {
                                let expected_left_op_span =
                                    get_span_from_ast_span(ast_add_op.node_span());
                                assert_eq!(left_op.span(), &expected_left_op_span);
                                assert_eq!(left_op.value(), &BinaryOp::Add);

                                let expected_left_span =
                                    get_span_from_ast_span(ast_left_1.node_span());
                                assert_eq!(left_left.span(), &expected_left_span);
                                match left_left.value() {
                                    ir::expr::Expr::Literal { value } => {
                                        assert_eq!(value, &Literal::Number(1.0));
                                    }
                                    _ => panic!(
                                        "Expected literal on left side, got {:?}",
                                        left_left.value()
                                    ),
                                }

                                let expected_right_span =
                                    get_span_from_ast_span(ast_right_1.node_span());
                                assert_eq!(left_right.span(), &expected_right_span);
                                match left_right.value() {
                                    ir::expr::Expr::Literal { value } => {
                                        assert_eq!(value, &Literal::Number(2.0));
                                    }
                                    _ => panic!(
                                        "Expected literal on right side, got {:?}",
                                        left_right.value()
                                    ),
                                }
                            }
                            _ => panic!(
                                "Expected binary operation on left side, got {:?}",
                                left.value()
                            ),
                        }

                        // check right side (sin(3.14))
                        match right.value() {
                            ir::expr::Expr::FunctionCall { name, args } => {
                                let expected_name_span =
                                    get_span_from_ast_span(ast_func_name.node_span());
                                assert_eq!(name.span(), &expected_name_span);
                                assert_eq!(name.value(), &FunctionName::sin());

                                assert_eq!(args.len(), 1);
                                let expected_arg_span =
                                    get_span_from_ast_span(ast_func_arg.node_span());
                                assert_eq!(args[0].span(), &expected_arg_span);
                                match args[0].value() {
                                    ir::expr::Expr::Literal { value } => {
                                        assert_eq!(value, &Literal::Number(3.14));
                                    }
                                    _ => panic!(
                                        "Expected literal argument, got {:?}",
                                        args[0].value()
                                    ),
                                }
                            }
                            _ => panic!(
                                "Expected function call on right side, got {:?}",
                                right.value()
                            ),
                        }
                    }
                    _ => panic!("Expected binary operation, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected successful result, got {:?}", result),
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

        for (ast_op, expected_ir_op) in operations {
            // create the binary operation node
            let ast_op_node = helper::create_binary_op_node(ast_op, 0, 1);

            // resolve the binary operation
            let result = resolve_binary_op(&ast_op_node);

            // check the result
            let expected_span = get_span_from_ast_span(ast_op_node.node_span());
            assert_eq!(result.span(), &expected_span);
            assert_eq!(result.value(), &expected_ir_op);
        }
    }

    #[test]
    fn test_resolve_unary_op_all_operations() {
        // create the operations
        let operations = vec![
            (ast::expression::UnaryOp::Neg, UnaryOp::Neg),
            (ast::expression::UnaryOp::Not, UnaryOp::Not),
        ];

        for (ast_op, expected_ir_op) in operations {
            // create the unary operation node
            let ast_op_node = helper::create_unary_op_node(ast_op, 0, 1);

            // resolve the unary operation
            let result = resolve_unary_op(&ast_op_node);

            // check the result
            let expected_span = get_span_from_ast_span(ast_op_node.node_span());
            assert_eq!(result.span(), &expected_span);
            assert_eq!(result.value(), &expected_ir_op);
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
            // create the function name node
            let ast_func_name_node = helper::create_identifier_node(func_name, 0, 1);

            // resolve the function name
            let result = resolve_function_name(&ast_func_name_node);

            // check the result
            let expected_span = get_span_from_ast_span(ast_func_name_node.node_span());
            assert_eq!(result.span(), &expected_span);
            assert_eq!(result.value(), &expected_func_builtin);
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
            // create the function name node
            let ast_func_name_node = helper::create_identifier_node(func_name, 0, 1);

            // resolve the function name
            let result = resolve_function_name(&ast_func_name_node);

            // check the result
            let expected_span = get_span_from_ast_span(ast_func_name_node.node_span());
            assert_eq!(result.span(), &expected_span);
            match result.value() {
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
        let ast_number = helper::create_literal_node(ast::expression::Literal::Number(42.5), 0, 4);
        let ir_number = resolve_literal(&ast_number);
        assert_eq!(ir_number, Literal::Number(42.5));

        // Test string
        let ast_string = helper::create_literal_node(
            ast::expression::Literal::String("test string".to_string()),
            0,
            11,
        );
        let ir_string = resolve_literal(&ast_string);
        assert_eq!(ir_string, Literal::String("test string".to_string()));

        // Test boolean
        let ast_bool = helper::create_literal_node(ast::expression::Literal::Boolean(false), 0, 5);
        let ir_bool = resolve_literal(&ast_bool);
        assert_eq!(ir_bool, Literal::Boolean(false));
    }

    #[test]
    fn test_resolve_expression_with_errors() {
        // create the expression
        let ast_left_var = helper::create_identifier_variable("undefined1");
        let ast_left_expr = helper::create_variable_expr_node(ast_left_var.clone(), 0, 11);
        let ast_right_var = helper::create_identifier_variable("undefined2");
        let ast_right_expr = helper::create_variable_expr_node(ast_right_var.clone(), 15, 26);
        let ast_add_op = helper::create_binary_op_node(ast::expression::BinaryOp::Add, 11, 14);
        let expr = helper::create_binary_op_expr_node(
            ast_left_expr.clone(),
            ast_add_op.clone(),
            ast_right_expr.clone(),
            0,
            26,
        );

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

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

                let undefined1_span = get_span_from_ast_span(ast_left_var.node_span());
                assert!(error_identifiers.contains(&WithSpan::new(
                    Identifier::new("undefined1"),
                    undefined1_span
                )));

                let undefined2_span = get_span_from_ast_span(ast_right_var.node_span());
                assert!(error_identifiers.contains(&WithSpan::new(
                    Identifier::new("undefined2"),
                    undefined2_span
                )));
            }
            _ => panic!("Expected error, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_parenthesized_expression() {
        // Test a simple parenthesized expression: (1 + 2)
        let ast_left =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 1, 2);
        let ast_right =
            helper::create_literal_expr_node(ast::expression::Literal::Number(2.0), 5, 6);
        let ast_add_op = helper::create_binary_op_node(ast::expression::BinaryOp::Add, 2, 3);
        let inner_expr = helper::create_binary_op_expr_node(
            ast_left.clone(),
            ast_add_op.clone(),
            ast_right.clone(),
            0,
            7,
        );
        let expr = helper::create_parenthesized_expr_node(inner_expr.clone(), 0, 8);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(inner_expr.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_add_op.node_span());
                        assert_eq!(op.span(), &expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Add);

                        let expected_left_span = get_span_from_ast_span(ast_left.node_span());
                        assert_eq!(left.span(), &expected_left_span);
                        match left.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal on left side, got {:?}", left.value()),
                        }

                        let expected_right_span = get_span_from_ast_span(ast_right.node_span());
                        assert_eq!(right.span(), &expected_right_span);
                        match right.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(2.0));
                            }
                            _ => panic!("Expected literal on right side, got {:?}", right.value()),
                        }
                    }
                    _ => panic!("Expected binary operation, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected successful result, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_nested_parenthesized_expression() {
        // Test nested parentheses: ((1 + 2) * 3)
        let ast_inner_left =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 2, 3);
        let ast_inner_right =
            helper::create_literal_expr_node(ast::expression::Literal::Number(2.0), 6, 7);
        let ast_add_op = helper::create_binary_op_node(ast::expression::BinaryOp::Add, 3, 4);
        let inner_binary = helper::create_binary_op_expr_node(
            ast_inner_left.clone(),
            ast_add_op.clone(),
            ast_inner_right.clone(),
            1,
            8,
        );
        let inner_parenthesized =
            helper::create_parenthesized_expr_node(inner_binary.clone(), 1, 9);
        let ast_outer_right =
            helper::create_literal_expr_node(ast::expression::Literal::Number(3.0), 12, 13);
        let ast_mul_op = helper::create_binary_op_node(ast::expression::BinaryOp::Mul, 9, 10);
        let expr = helper::create_binary_op_expr_node(
            inner_parenthesized.clone(),
            ast_mul_op.clone(),
            ast_outer_right.clone(),
            0,
            13,
        );

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_mul_op.node_span());
                        assert_eq!(op.span(), &expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Mul);

                        // check left side ((1 + 2))
                        let expected_left_span = get_span_from_ast_span(inner_binary.node_span());
                        assert_eq!(left.span(), &expected_left_span);
                        match left.value() {
                            ir::expr::Expr::BinaryOp {
                                op: left_op,
                                left: left_left,
                                right: left_right,
                            } => {
                                let expected_left_op_span =
                                    get_span_from_ast_span(ast_add_op.node_span());
                                assert_eq!(left_op.span(), &expected_left_op_span);
                                assert_eq!(left_op.value(), &BinaryOp::Add);

                                let expected_left_left_span =
                                    get_span_from_ast_span(ast_inner_left.node_span());
                                assert_eq!(left_left.span(), &expected_left_left_span);
                                match left_left.value() {
                                    ir::expr::Expr::Literal { value } => {
                                        assert_eq!(value, &Literal::Number(1.0));
                                    }
                                    _ => panic!(
                                        "Expected literal on left side, got {:?}",
                                        left_left.value()
                                    ),
                                }

                                let expected_left_right_span =
                                    get_span_from_ast_span(ast_inner_right.node_span());
                                assert_eq!(left_right.span(), &expected_left_right_span);
                                match left_right.value() {
                                    ir::expr::Expr::Literal { value } => {
                                        assert_eq!(value, &Literal::Number(2.0));
                                    }
                                    _ => panic!(
                                        "Expected literal on right side, got {:?}",
                                        left_right.value()
                                    ),
                                }
                            }
                            _ => panic!(
                                "Expected binary operation on left side, got {:?}",
                                left.value()
                            ),
                        }

                        // check right side (3)
                        let expected_right_span =
                            get_span_from_ast_span(ast_outer_right.node_span());
                        assert_eq!(right.span(), &expected_right_span);
                        match right.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(3.0));
                            }
                            _ => panic!("Expected literal on right side, got {:?}", right.value()),
                        }
                    }
                    _ => panic!("Expected binary operation, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected successful result, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_single_literal_multiple_parentheses() {
        // Test a single literal wrapped in multiple parentheses: ((42))
        let ast_inner_literal =
            helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 2, 4);
        let first_parentheses =
            helper::create_parenthesized_expr_node(ast_inner_literal.clone(), 1, 5);
        let expr = helper::create_parenthesized_expr_node(first_parentheses.clone(), 0, 6);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(ast_inner_literal.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Number(42.0));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected successful result, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_single_literal_deep_nested_parentheses() {
        // Test a single literal with deeply nested parentheses: (((3.14)))
        let inner_literal =
            helper::create_literal_expr_node(ast::expression::Literal::Number(3.14), 3, 7);
        let third_level = helper::create_parenthesized_expr_node(inner_literal.clone(), 2, 8);
        let second_level = helper::create_parenthesized_expr_node(third_level, 1, 9);
        let expr = helper::create_parenthesized_expr_node(second_level, 0, 10);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(inner_literal.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Number(3.14));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected literal expression, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_parenthesized_function_call() {
        // Test a parenthesized function call: (sin(3.14))
        let func_arg =
            helper::create_literal_expr_node(ast::expression::Literal::Number(3.14), 5, 9);
        let func_name = helper::create_identifier_node("sin", 1, 4);
        let func_call = helper::create_function_call_expr_node(
            func_name.clone(),
            vec![func_arg.clone()],
            1,
            10,
        );
        let expr = helper::create_parenthesized_expr_node(func_call.clone(), 0, 11);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(func_call.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(func_name.node_span());
                        assert_eq!(name.span(), &expected_name_span);
                        assert_eq!(name.value(), &FunctionName::sin());

                        assert_eq!(args.len(), 1);
                        let expected_arg_span = get_span_from_ast_span(func_arg.node_span());
                        assert_eq!(args[0].span(), &expected_arg_span);
                        match args[0].value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(3.14));
                            }
                            _ => panic!("Expected literal argument, got {:?}", args[0].value()),
                        }
                    }
                    _ => panic!("Expected function call, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected function call, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_parenthesized_unary_operation() {
        // Test a parenthesized unary operation: (-5)
        let inner_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 1, 2);
        let ast_op = helper::create_unary_op_node(ast::expression::UnaryOp::Neg, 0, 1);
        let unary_expr =
            helper::create_unary_op_expr_node(ast_op.clone(), inner_expr.clone(), 0, 3);
        let expr = helper::create_parenthesized_expr_node(unary_expr.clone(), 0, 4);

        // create the context
        let (local_vars, param_info, submodel_info, model_info) = helper::create_empty_context();

        // resolve the expression
        let result = resolve_expr(&expr, &local_vars, &param_info, &submodel_info, &model_info);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(unary_expr.node_span());
                assert_eq!(result.span(), &expected_span);
                match result.value() {
                    ir::expr::Expr::UnaryOp { op, expr } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), &expected_op_span);
                        assert_eq!(op.value(), &UnaryOp::Neg);

                        let expected_expr_span = get_span_from_ast_span(inner_expr.node_span());
                        assert_eq!(expr.span(), &expected_expr_span);
                        match expr.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(5.0));
                            }
                            _ => panic!("Expected literal expression, got {:?}", expr.value()),
                        }
                    }
                    _ => panic!("Expected unary operation, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected unary operation, got {:?}", result),
        }
    }
}
