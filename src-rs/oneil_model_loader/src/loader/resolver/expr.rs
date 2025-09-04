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

use oneil_ast as ast;
use oneil_ir::{self as ir, IrSpan};

use crate::{
    BuiltinRef,
    error::{self, VariableResolutionError},
    loader::resolver::variable::resolve_variable,
    util::{
        context::{ParameterContext, ReferenceContext},
        get_span_from_ast_span,
    },
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
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters and their status
/// * `submodel_info` - Information about available submodels and their paths
/// * `model_info` - Information about all available models and their loading status
///
/// # Returns
///
/// * `Ok(oneil_ir::expr::ExprWithSpan)` - The resolved model expression
/// * `Err(Vec<VariableResolutionError>)` - Any variable resolution errors that occurred
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
    value: &ast::ExprNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    let value_span = get_span_from_ast_span(value.node_span());
    match value.node_value() {
        ast::Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } => resolve_comparison_expression(
            op,
            left,
            right,
            rest_chained,
            value_span,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::BinaryOp { op, left, right } => resolve_binary_expression(
            op,
            left,
            right,
            value_span,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::UnaryOp { op, expr } => resolve_unary_expression(
            op,
            expr,
            value_span,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::FunctionCall { name, args } => resolve_function_call_expression(
            name,
            args,
            value_span,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::Variable(variable) => {
            resolve_variable_expression(variable, builtin_ref, reference_context, parameter_context)
        }
        ast::Expr::Literal(literal) => Ok(resolve_literal_expression(literal, value_span)),
        ast::Expr::Parenthesized { expr } => resolve_parenthesized_expression(
            expr,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
    }
}

/// Resolves a comparison expression with optional chained comparisons.
///
/// # Arguments
///
/// * `op` - The comparison operator
/// * `left` - The left operand expression
/// * `right` - The right operand expression
/// * `rest_chained` - Additional chained comparison operations
/// * `value_span` - The span of the entire expression
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// The resolved comparison expression or error collection
fn resolve_comparison_expression(
    op: &ast::ComparisonOpNode,
    left: &ast::ExprNode,
    right: &ast::ExprNode,
    rest_chained: &[(ast::ComparisonOpNode, ast::ExprNode)],
    value_span: IrSpan,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    let left = resolve_expr(left, builtin_ref, reference_context, parameter_context);
    let right = resolve_expr(right, builtin_ref, reference_context, parameter_context);
    let op_with_span = resolve_comparison_op(op);

    // Resolve the chained comparisons
    let rest_chained = rest_chained.iter().map(|(op, expr)| {
        let expr = resolve_expr(expr, builtin_ref, reference_context, parameter_context);
        let op_with_span = resolve_comparison_op(op);
        expr.map(|expr| (op_with_span, expr))
    });

    let left_right_result = error::combine_errors(left, right);
    let rest_chained_result = error::combine_error_list(rest_chained);
    let ((left, right), rest_chained) =
        error::combine_errors(left_right_result, rest_chained_result)?;

    let expr = ir::Expr::comparison_op(op_with_span, left, right, rest_chained);
    Ok(ir::WithSpan::new(expr, value_span))
}

/// Resolves a binary operation expression.
///
/// # Arguments
///
/// * `op` - The binary operator
/// * `left` - The left operand expression
/// * `right` - The right operand expression
/// * `value_span` - The span of the entire expression
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// The resolved binary expression or error collection
fn resolve_binary_expression(
    op: &ast::BinaryOpNode,
    left: &ast::ExprNode,
    right: &ast::ExprNode,
    value_span: IrSpan,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    let left = resolve_expr(left, builtin_ref, reference_context, parameter_context);
    let right = resolve_expr(right, builtin_ref, reference_context, parameter_context);
    let op_with_span = resolve_binary_op(op);

    let (left, right) = error::combine_errors(left, right)?;

    let expr = ir::Expr::binary_op(op_with_span, left, right);
    Ok(ir::WithSpan::new(expr, value_span))
}

/// Resolves a unary operation expression.
///
/// # Arguments
///
/// * `op` - The unary operator
/// * `expr` - The operand expression
/// * `value_span` - The span of the entire expression
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// The resolved unary expression or error collection
fn resolve_unary_expression(
    op: &ast::UnaryOpNode,
    expr: &ast::ExprNode,
    value_span: IrSpan,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    let expr = resolve_expr(expr, builtin_ref, reference_context, parameter_context);
    let op_with_span = resolve_unary_op(op);

    match expr {
        Ok(expr) => Ok(ir::WithSpan::new(
            ir::Expr::unary_op(op_with_span, expr),
            value_span,
        )),
        Err(errors) => Err(errors),
    }
}

/// Resolves a function call expression.
///
/// # Arguments
///
/// * `name` - The function name identifier
/// * `args` - The function arguments
/// * `value_span` - The span of the entire expression
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// The resolved function call expression or error collection
fn resolve_function_call_expression(
    name: &ast::IdentifierNode,
    args: &[ast::ExprNode],
    value_span: IrSpan,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    let name_with_span = resolve_function_name(name, builtin_ref);
    let args = args
        .iter()
        .map(|arg| resolve_expr(arg, builtin_ref, reference_context, parameter_context));

    let args = error::combine_error_list(args)?;

    let expr = ir::Expr::function_call(name_with_span, args);
    Ok(ir::WithSpan::new(expr, value_span))
}

/// Resolves a variable expression.
///
/// # Arguments
///
/// * `variable` - The variable node to resolve
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// The resolved variable expression or error collection
fn resolve_variable_expression(
    variable: &ast::VariableNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    resolve_variable(variable, builtin_ref, reference_context, parameter_context)
        .map_err(|error| vec![error])
}

/// Resolves a literal expression.
///
/// # Arguments
///
/// * `literal` - The literal node to resolve
/// * `value_span` - The span of the entire expression
///
/// # Returns
///
/// The resolved literal expression
fn resolve_literal_expression(
    literal: &ast::LiteralNode,
    value_span: IrSpan,
) -> ir::WithSpan<ir::Expr> {
    let literal = resolve_literal(literal);
    let expr = ir::Expr::literal(literal);
    ir::WithSpan::new(expr, value_span)
}

/// Resolves a parenthesized expression.
///
/// # Arguments
///
/// * `expr` - The expression inside parentheses
/// * `builtin_ref` - Reference to built-in functions and variables
/// * `defined_parameters_info` - Information about defined parameters
/// * `submodel_info` - Information about available submodels
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// The resolved expression (parentheses are removed during resolution)
fn resolve_parenthesized_expression(
    expr: &ast::ExprNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::ExprWithSpan, Vec<VariableResolutionError>> {
    resolve_expr(expr, builtin_ref, reference_context, parameter_context)
}

/// Converts an AST comparison operation to a model comparison operation.
///
/// This function maps AST comparison operations to their corresponding model comparison operations.
/// All AST comparison operations have direct equivalents in the model system.
///
/// # Arguments
///
/// * `op` - The AST comparison operation to convert
///
/// # Returns
///
/// The corresponding model comparison operation
fn resolve_comparison_op(op: &ast::ComparisonOpNode) -> ir::WithSpan<ir::ComparisonOp> {
    let op_value = match op.node_value() {
        ast::ComparisonOp::LessThan => ir::ComparisonOp::LessThan,
        ast::ComparisonOp::LessThanEq => ir::ComparisonOp::LessThanEq,
        ast::ComparisonOp::GreaterThan => ir::ComparisonOp::GreaterThan,
        ast::ComparisonOp::GreaterThanEq => ir::ComparisonOp::GreaterThanEq,
        ast::ComparisonOp::Eq => ir::ComparisonOp::Eq,
        ast::ComparisonOp::NotEq => ir::ComparisonOp::NotEq,
    };
    let op_span = get_span_from_ast_span(op.node_span());
    ir::WithSpan::new(op_value, op_span)
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
fn resolve_binary_op(op: &ast::BinaryOpNode) -> ir::WithSpan<ir::BinaryOp> {
    let op_value = match op.node_value() {
        ast::BinaryOp::Add => ir::BinaryOp::Add,
        ast::BinaryOp::Sub => ir::BinaryOp::Sub,
        ast::BinaryOp::TrueSub => ir::BinaryOp::TrueSub,
        ast::BinaryOp::Mul => ir::BinaryOp::Mul,
        ast::BinaryOp::Div => ir::BinaryOp::Div,
        ast::BinaryOp::TrueDiv => ir::BinaryOp::TrueDiv,
        ast::BinaryOp::Mod => ir::BinaryOp::Mod,
        ast::BinaryOp::Pow => ir::BinaryOp::Pow,
        ast::BinaryOp::And => ir::BinaryOp::And,
        ast::BinaryOp::Or => ir::BinaryOp::Or,
        ast::BinaryOp::MinMax => ir::BinaryOp::MinMax,
    };
    let op_span = get_span_from_ast_span(op.node_span());
    ir::WithSpan::new(op_value, op_span)
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
fn resolve_unary_op(op: &ast::UnaryOpNode) -> ir::WithSpan<ir::UnaryOp> {
    let op_value = match op.node_value() {
        ast::UnaryOp::Neg => ir::UnaryOp::Neg,
        ast::UnaryOp::Not => ir::UnaryOp::Not,
    };
    let op_span = get_span_from_ast_span(op.node_span());
    ir::WithSpan::new(op_value, op_span)
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
fn resolve_function_name(
    name: &ast::IdentifierNode,
    builtin_ref: &impl BuiltinRef,
) -> ir::WithSpan<ir::FunctionName> {
    let span = get_span_from_ast_span(name.node_span());
    let name = ir::Identifier::new(name.as_str());

    let name = if builtin_ref.has_builtin_function(&name) {
        ir::FunctionName::builtin(name)
    } else {
        ir::FunctionName::imported(name)
    };

    ir::WithSpan::new(name, span)
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
fn resolve_literal(literal: &ast::LiteralNode) -> ir::Literal {
    match literal.node_value() {
        ast::Literal::Number(number) => ir::Literal::number(*number),
        ast::Literal::String(string) => ir::Literal::string(string.clone()),
        ast::Literal::Boolean(boolean) => ir::Literal::boolean(*boolean),
    }
}

#[cfg(test)]
#[cfg(never)]
mod tests {
    use super::*;
    use crate::test::{TestBuiltinRef, TestContext};

    use oneil_ast as ast;
    use oneil_ir as ir;
    use oneil_ir::span::IrSpan;
    use oneil_ir::{
        expr::{BinaryOp, FunctionName, Literal, UnaryOp},
        reference::Identifier,
    };
    use std::collections::HashSet;

    mod helper {

        use super::*;

        /// Helper function to create basic test data structures for tests that
        /// don't rely on any context.
        /// Helper function to create a test span
        pub fn test_span(start: usize, end: usize) -> ast::AstSpan {
            ast::AstSpan::new(start, end - start, 0)
        }

        /// Helper function to create a literal expression node
        pub fn create_literal_expr_node(
            literal: ast::expression::Literal,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let literal_node = ast::node::Node::new(&test_span(start, end), literal);
            let expr = ast::expression::Expr::Literal(literal_node);
            ast::node::Node::new(&test_span(start, end), expr)
        }

        /// Helper function to create a variable expression node
        pub fn create_variable_expr_node(
            variable: ast::expression::VariableNode,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr = ast::expression::Expr::Variable(variable);
            ast::node::Node::new(&test_span(start, end), expr)
        }

        /// Helper function to create a binary operation node
        pub fn create_binary_op_node(
            op: ast::expression::BinaryOp,
            start: usize,
            end: usize,
        ) -> ast::expression::BinaryOpNode {
            ast::node::Node::new(&test_span(start, end), op)
        }

        /// Helper function to create a comparison operation node
        pub fn create_comparison_op_node(
            op: ast::expression::ComparisonOp,
            start: usize,
            end: usize,
        ) -> ast::expression::ComparisonOpNode {
            ast::node::Node::new(&test_span(start, end), op)
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
            ast::node::Node::new(&test_span(start, end), expr)
        }

        /// Helper function to create a unary operation node
        pub fn create_unary_op_node(
            op: ast::expression::UnaryOp,
            start: usize,
            end: usize,
        ) -> ast::expression::UnaryOpNode {
            ast::node::Node::new(&test_span(start, end), op)
        }

        /// Helper function to create an identifier node
        pub fn create_identifier_node(
            name: &str,
            start: usize,
            end: usize,
        ) -> ast::naming::IdentifierNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            ast::node::Node::new(&test_span(start, end), identifier)
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
            ast::node::Node::new(&test_span(start, end), expr_node)
        }

        /// Helper function to create a function call expression node
        pub fn create_function_call_expr_node(
            name: ast::naming::IdentifierNode,
            args: Vec<ast::expression::ExprNode>,
            start: usize,
            end: usize,
        ) -> ast::expression::ExprNode {
            let expr = ast::expression::Expr::FunctionCall { name, args };
            ast::node::Node::new(&test_span(start, end), expr)
        }

        /// Helper function to create a simple identifier variable
        pub fn create_identifier_variable(name: &str) -> ast::expression::VariableNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            let identifier_node = ast::node::Node::new(&test_span(0, name.len()), identifier);
            let variable = ast::expression::Variable::Identifier(identifier_node);
            ast::node::Node::new(&test_span(0, name.len()), variable)
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
            ast::node::Node::new(&test_span(start, end), parenthesized_expr)
        }

        /// Helper function to create a parameter ID with span
        pub fn create_ir_id_with_span(
            name: &str,
            start: usize,
            end: usize,
        ) -> WithSpan<Identifier> {
            WithSpan::new(Identifier::new(name), Span::new(start, end))
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
            ast::node::Node::new(&test_span(start, end), literal)
        }
    }

    #[test]
    fn test_resolve_literal_number() {
        // create the expression
        let literal =
            helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 0, 4);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&literal, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(literal.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Number(42.0));
                    }
                    _ => panic!("Expected literal expression, got {result:?}"),
                }
            }
            _ => panic!("Expected literal expression, got {result:?}"),
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
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&literal, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(literal.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::String("hello".to_string()));
                    }
                    _ => panic!("Expected literal expression, got {result:?}"),
                }
            }
            _ => panic!("Expected literal expression, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_literal_boolean() {
        // create the expression
        let literal =
            helper::create_literal_expr_node(ast::expression::Literal::Boolean(true), 0, 4);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&literal, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(literal.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Boolean(true));
                    }
                    _ => panic!("Expected literal expression, got {result:?}"),
                }
            }
            _ => panic!("Expected literal expression, got {result:?}"),
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
        let expr = helper::create_binary_op_expr_node(ast_left, ast_op.clone(), ast_right, 0, 5);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Add);

                        match left.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal expression on left, got {left:?}"),
                        }

                        match right.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(2.0));
                            }
                            _ => panic!("Expected literal expression on right, got {right:?}"),
                        }
                    }
                    _ => panic!("Expected binary operation, got {result:?}"),
                }
            }
            _ => panic!("Expected binary operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_unary_op() {
        // create the expression
        let ast_inner_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 1, 4);
        let ast_op = helper::create_unary_op_node(ast::expression::UnaryOp::Neg, 0, 1);
        let expr = helper::create_unary_op_expr_node(ast_op.clone(), ast_inner_expr, 0, 4);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::expr::Expr::UnaryOp { op, expr } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &UnaryOp::Neg);

                        match expr.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(5.0));
                            }
                            _ => panic!("Expected literal expression, got {expr:?}"),
                        }
                    }
                    _ => panic!("Expected unary operation, got {result:?}"),
                }
            }
            _ => panic!("Expected unary operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_function_call_builtin() {
        // create the expression
        let ast_arg = helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 4, 8);
        let ast_name = helper::create_identifier_node("foo", 0, 3);
        let expr = helper::create_function_call_expr_node(ast_name.clone(), vec![ast_arg], 0, 8);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::expr::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(ast_name.node_span());
                        assert_eq!(name.span(), expected_name_span);
                        assert_eq!(
                            name.value(),
                            &FunctionName::imported(Identifier::new("foo"))
                        );

                        assert_eq!(args.len(), 1);

                        match &args[0].value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal argument, got {:?}", args[0]),
                        }
                    }
                    _ => panic!("Expected function call, got {result:?}"),
                }
            }
            _ => panic!("Expected function call, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_function_call_imported() {
        // create the expression
        let ast_arg =
            helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 16, 19);
        let ast_name = helper::create_identifier_node("custom_function", 0, 15);
        let expr = helper::create_function_call_expr_node(ast_name.clone(), vec![ast_arg], 0, 19);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::expr::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(ast_name.node_span());
                        assert_eq!(name.span(), expected_name_span);
                        assert_eq!(
                            name.value(),
                            &FunctionName::imported(Identifier::new("custom_function"))
                        );

                        assert_eq!(args.len(), 1);

                        match &args[0].value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(42.0));
                            }
                            _ => panic!("Expected literal argument, got {:?}", args[0]),
                        }
                    }
                    _ => panic!("Expected function call, got {result:?}"),
                }
            }
            _ => panic!("Expected function call, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_variable_builtin() {
        // create the expression
        let ast_variable = helper::create_identifier_variable("x");
        let expr = helper::create_variable_expr_node(ast_variable, 0, 1);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::expr::Expr::Variable(variable) => {
                        assert_eq!(variable, &ir::expr::Variable::Builtin(Identifier::new("x")));
                    }
                    _ => panic!("Expected variable expression, got {result:?}"),
                }
            }
            _ => panic!("Expected local variable, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_variable_parameter() {
        // create the expression
        let ast_variable = helper::create_identifier_variable("param");
        let expr = helper::create_variable_expr_node(ast_variable, 0, 5);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

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

        let context =
            TestContext::new().with_parameter_context([(param_id.take_value(), parameter)]);

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::expr::Expr::Variable(variable) => {
                        assert_eq!(
                            variable,
                            &ir::expr::Variable::Parameter(Identifier::new("param"))
                        );
                    }
                    _ => panic!("Expected variable expression, got {result:?}"),
                }
            }
            _ => panic!("Expected parameter variable, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_variable_undefined() {
        // create the expression
        let ast_variable = helper::create_identifier_variable("undefined");
        let expr = helper::create_variable_expr_node(ast_variable.clone(), 0, 9);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span,
                    } => {
                        let span = get_span_from_ast_span(ast_variable.node_span());
                        assert_eq!(reference_span, &span);
                        assert_eq!(parameter, &Identifier::new("undefined"));
                    }
                    _ => panic!("Expected undefined parameter error, got {:?}", errors[0]),
                }
            }
            _ => panic!("Expected error, got {result:?}"),
        }
    }

    #[test]
    #[allow(clippy::too_many_lines, reason = "this is a complex test")]
    fn test_resolve_complex_expression() {
        // create the expression: (1 + 2) * foo(1)
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
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 12, 16);
        let ast_func_name = helper::create_identifier_node("foo", 8, 11);
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
            func_call,
            0,
            17,
        );

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_mul_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Mul);

                        // check left side (1 + 2)
                        let expected_left_span = get_span_from_ast_span(inner_binary.node_span());
                        assert_eq!(left.span(), expected_left_span);
                        match left.value() {
                            ir::expr::Expr::BinaryOp {
                                op: left_op,
                                left: left_left,
                                right: left_right,
                            } => {
                                let expected_left_op_span =
                                    get_span_from_ast_span(ast_add_op.node_span());
                                assert_eq!(left_op.span(), expected_left_op_span);
                                assert_eq!(left_op.value(), &BinaryOp::Add);

                                let expected_left_span =
                                    get_span_from_ast_span(ast_left_1.node_span());
                                assert_eq!(left_left.span(), expected_left_span);
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
                                assert_eq!(left_right.span(), expected_right_span);
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

                        // check right side (foo(3.14))
                        match right.value() {
                            ir::expr::Expr::FunctionCall { name, args } => {
                                let expected_name_span =
                                    get_span_from_ast_span(ast_func_name.node_span());
                                assert_eq!(name.span(), expected_name_span);
                                assert_eq!(
                                    name.value(),
                                    &FunctionName::imported(Identifier::new("foo"))
                                );

                                assert_eq!(args.len(), 1);
                                let expected_arg_span =
                                    get_span_from_ast_span(ast_func_arg.node_span());
                                assert_eq!(args[0].span(), expected_arg_span);
                                match args[0].value() {
                                    ir::expr::Expr::Literal { value } => {
                                        assert_eq!(value, &Literal::Number(1.0));
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
            _ => panic!("Expected successful result, got {result:?}"),
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
            assert_eq!(result.span(), expected_span);
            assert_eq!(result.value(), &expected_ir_op);
        }
    }

    #[test]
    fn test_resolve_comparison_op_all_operations() {
        use oneil_ir::expr::ComparisonOp;

        // create the operations
        let operations = vec![
            (
                ast::expression::ComparisonOp::LessThan,
                ComparisonOp::LessThan,
            ),
            (
                ast::expression::ComparisonOp::LessThanEq,
                ComparisonOp::LessThanEq,
            ),
            (
                ast::expression::ComparisonOp::GreaterThan,
                ComparisonOp::GreaterThan,
            ),
            (
                ast::expression::ComparisonOp::GreaterThanEq,
                ComparisonOp::GreaterThanEq,
            ),
            (ast::expression::ComparisonOp::Eq, ComparisonOp::Eq),
            (ast::expression::ComparisonOp::NotEq, ComparisonOp::NotEq),
        ];

        for (ast_op, expected_ir_op) in operations {
            // create the comparison operation node
            let ast_op_node = helper::create_comparison_op_node(ast_op, 0, 1);

            // resolve the comparison operation
            let result = resolve_comparison_op(&ast_op_node);

            // check the result
            let expected_span = get_span_from_ast_span(ast_op_node.node_span());
            assert_eq!(result.span(), expected_span);
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
            assert_eq!(result.span(), expected_span);
            assert_eq!(result.value(), &expected_ir_op);
        }
    }

    #[test]
    fn test_resolve_function_name_builtin() {
        // create the builtin functions
        let builtin_functions = ["min", "max", "sin", "cos", "tan"];

        let builtin_ref = TestBuiltinRef::new().with_builtin_functions(builtin_functions);

        // resolve the function names
        for func_name in builtin_functions {
            // create the function name node
            let ast_func_name_node = helper::create_identifier_node(func_name, 0, 1);

            // resolve the function name
            let result = resolve_function_name(&ast_func_name_node, &builtin_ref);

            // check the result
            let expected_span = get_span_from_ast_span(ast_func_name_node.node_span());
            let expected_func_builtin = FunctionName::builtin(Identifier::new(func_name));
            assert_eq!(result.span(), expected_span);
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

        let builtin_ref = TestBuiltinRef::new();

        // resolve the function names
        for func_name in imported_functions {
            // create the function name node
            let ast_func_name_node = helper::create_identifier_node(func_name, 0, 1);

            // resolve the function name
            let result = resolve_function_name(&ast_func_name_node, &builtin_ref);

            // check the result
            let expected_span = get_span_from_ast_span(ast_func_name_node.node_span());
            assert_eq!(result.span(), expected_span);
            match result.value() {
                FunctionName::Imported(name) => {
                    assert_eq!(name.as_str(), func_name);
                }
                FunctionName::Builtin(_) => {
                    panic!("Expected imported function for '{func_name}', got {result:?}")
                }
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
        let expr =
            helper::create_binary_op_expr_node(ast_left_expr, ast_add_op, ast_right_expr, 0, 26);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 2);

                let error_identifiers: Vec<_> = errors
                    .iter()
                    .filter_map(|e| {
                        if let VariableResolutionError::UndefinedParameter {
                            model_path: None,
                            parameter,
                            reference_span: _,
                        } = e
                        {
                            Some(parameter.clone())
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
            _ => panic!("Expected error, got {result:?}"),
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

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(inner_expr.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_add_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Add);

                        let expected_left_span = get_span_from_ast_span(ast_left.node_span());
                        assert_eq!(left.span(), expected_left_span);
                        match left.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal on left side, got {:?}", left.value()),
                        }

                        let expected_right_span = get_span_from_ast_span(ast_right.node_span());
                        assert_eq!(right.span(), expected_right_span);
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
            _ => panic!("Expected successful result, got {result:?}"),
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
            inner_parenthesized,
            ast_mul_op.clone(),
            ast_outer_right.clone(),
            0,
            13,
        );

        // create the context
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::BinaryOp { op, left, right } => {
                        let expected_op_span = get_span_from_ast_span(ast_mul_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &BinaryOp::Mul);

                        // check left side ((1 + 2))
                        let expected_left_span = get_span_from_ast_span(inner_binary.node_span());
                        assert_eq!(left.span(), expected_left_span);
                        match left.value() {
                            ir::expr::Expr::BinaryOp {
                                op: left_op,
                                left: left_left,
                                right: left_right,
                            } => {
                                let expected_left_op_span =
                                    get_span_from_ast_span(ast_add_op.node_span());
                                assert_eq!(left_op.span(), expected_left_op_span);
                                assert_eq!(left_op.value(), &BinaryOp::Add);

                                let expected_left_left_span =
                                    get_span_from_ast_span(ast_inner_left.node_span());
                                assert_eq!(left_left.span(), expected_left_left_span);
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
                                assert_eq!(left_right.span(), expected_left_right_span);
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
                        assert_eq!(right.span(), expected_right_span);
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
            _ => panic!("Expected successful result, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_single_literal_multiple_parentheses() {
        // Test a single literal wrapped in multiple parentheses: ((42))
        let ast_inner_literal =
            helper::create_literal_expr_node(ast::expression::Literal::Number(42.0), 2, 4);
        let first_parentheses =
            helper::create_parenthesized_expr_node(ast_inner_literal.clone(), 1, 5);
        let expr = helper::create_parenthesized_expr_node(first_parentheses, 0, 6);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(ast_inner_literal.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Number(42.0));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected successful result, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_single_literal_deep_nested_parentheses() {
        // Test a single literal with deeply nested parentheses: (((1)))
        let inner_literal =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 3, 7);
        let third_level = helper::create_parenthesized_expr_node(inner_literal.clone(), 2, 8);
        let second_level = helper::create_parenthesized_expr_node(third_level, 1, 9);
        let expr = helper::create_parenthesized_expr_node(second_level, 0, 10);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(inner_literal.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::Literal { value } => {
                        assert_eq!(value, &Literal::Number(1.0));
                    }
                    _ => panic!("Expected literal expression, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected literal expression, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_parenthesized_function_call() {
        // Test a parenthesized function call: (foo(3.14))
        let func_arg =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 5, 9);
        let func_name = helper::create_identifier_node("foo", 1, 4);
        let func_call = helper::create_function_call_expr_node(
            func_name.clone(),
            vec![func_arg.clone()],
            1,
            10,
        );
        let expr = helper::create_parenthesized_expr_node(func_call.clone(), 0, 11);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(func_call.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(func_name.node_span());
                        assert_eq!(name.span(), expected_name_span);
                        assert_eq!(
                            name.value(),
                            &FunctionName::imported(Identifier::new("foo"))
                        );

                        assert_eq!(args.len(), 1);
                        let expected_arg_span = get_span_from_ast_span(func_arg.node_span());
                        assert_eq!(args[0].span(), expected_arg_span);
                        match args[0].value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal argument, got {:?}", args[0].value()),
                        }
                    }
                    _ => panic!("Expected function call, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected function call, got {result:?}"),
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

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(unary_expr.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::UnaryOp { op, expr } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &UnaryOp::Neg);

                        let expected_expr_span = get_span_from_ast_span(inner_expr.node_span());
                        assert_eq!(expr.span(), expected_expr_span);
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
            _ => panic!("Expected unary operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression() {
        // Test a simple comparison expression: x < 5
        let left_var = helper::create_identifier_variable("x");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 1);
        let right_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 4, 5);
        let op = helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 2, 3);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op.clone(),
            left: Box::new(left_expr.clone()),
            right: Box::new(right_expr.clone()),
            rest_chained: vec![],
        };
        let expr_node = ast::node::Node::new(&helper::test_span(0, 5), expr);

        // create the context
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        // resolve the expression
        let result = resolve_expr(&expr_node, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr_node.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::ComparisonOp {
                        op: ir_op,
                        left,
                        right,
                        rest_chained,
                    } => {
                        let expected_op_span = get_span_from_ast_span(op.node_span());
                        assert_eq!(ir_op.span(), expected_op_span);
                        assert_eq!(ir_op.value(), &ir::expr::ComparisonOp::LessThan);

                        let expected_left_span = get_span_from_ast_span(left_expr.node_span());
                        assert_eq!(left.span(), expected_left_span);
                        match left.value() {
                            ir::expr::Expr::Variable(variable) => {
                                assert_eq!(
                                    variable,
                                    &ir::expr::Variable::Builtin(Identifier::new("x"))
                                );
                            }
                            _ => panic!("Expected variable expression, got {:?}", left.value()),
                        }

                        let expected_right_span = get_span_from_ast_span(right_expr.node_span());
                        assert_eq!(right.span(), expected_right_span);
                        match right.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(5.0));
                            }
                            _ => panic!("Expected literal expression, got {:?}", right.value()),
                        }

                        assert_eq!(rest_chained.len(), 0);
                    }
                    _ => panic!("Expected comparison operation, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected comparison operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_chained_comparison_expression() {
        // Test a chained comparison expression: 1 < x < 10
        let left_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 0, 1);
        let middle_var = helper::create_identifier_variable("x");
        let middle_expr = helper::create_variable_expr_node(middle_var, 4, 5);
        let right_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(10.0), 8, 10);

        let op1 = helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 2, 3);
        let op2 = helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 6, 7);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op1.clone(),
            left: Box::new(left_expr),
            right: Box::new(middle_expr),
            rest_chained: vec![(op2.clone(), right_expr.clone())],
        };
        let expr_node = ast::node::Node::new(&helper::test_span(0, 10), expr);

        // create the context and builtin ref
        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        // resolve the expression
        let result = resolve_expr(&expr_node, &builtin_ref, &context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr_node.node_span());
                assert_eq!(result.span(), expected_span);
                match result.value() {
                    ir::expr::Expr::ComparisonOp {
                        op,
                        left,
                        right,
                        rest_chained,
                    } => {
                        let expected_op_span = get_span_from_ast_span(op1.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &ir::expr::ComparisonOp::LessThan);

                        match left.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(1.0));
                            }
                            _ => panic!("Expected literal expression, got {:?}", left.value()),
                        }

                        match right.value() {
                            ir::expr::Expr::Variable(variable) => {
                                assert_eq!(
                                    variable,
                                    &ir::expr::Variable::Builtin(Identifier::new("x"))
                                );
                            }
                            _ => panic!("Expected variable expression, got {:?}", right.value()),
                        }

                        assert_eq!(rest_chained.len(), 1);
                        let (chained_op, chained_expr) = &rest_chained[0];
                        let expected_chained_op_span = get_span_from_ast_span(op2.node_span());
                        assert_eq!(chained_op.span(), expected_chained_op_span);
                        assert_eq!(chained_op.value(), &ir::expr::ComparisonOp::LessThan);

                        let expected_chained_expr_span =
                            get_span_from_ast_span(right_expr.node_span());
                        assert_eq!(chained_expr.span(), expected_chained_expr_span);
                        match chained_expr.value() {
                            ir::expr::Expr::Literal { value } => {
                                assert_eq!(value, &Literal::Number(10.0));
                            }
                            _ => panic!(
                                "Expected literal expression in chained comparison, got {:?}",
                                chained_expr.value()
                            ),
                        }
                    }
                    _ => panic!("Expected comparison operation, got {:?}", result.value()),
                }
            }
            _ => panic!("Expected comparison operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression_error_from_left_operand() {
        // Test error propagation from left operand of comparison expression
        let left_var = helper::create_identifier_variable("undefined_left");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 15);
        let right_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(5.0), 18, 19);
        let op = helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 16, 17);

        let expr = ast::expression::Expr::ComparisonOp {
            op,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            rest_chained: vec![],
        };
        let expr_node = ast::node::Node::new(&helper::test_span(0, 19), expr);

        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        let result = resolve_expr(&expr_node, &builtin_ref, &context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &Identifier::new("undefined_left"));
                    }
                    _ => panic!("Expected undefined parameter error, got {:?}", errors[0]),
                }
            }
            _ => panic!("Expected error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression_error_from_right_operand() {
        // Test error propagation from right operand of comparison expression
        let left_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 0, 1);
        let right_var = helper::create_identifier_variable("undefined_right");
        let right_expr = helper::create_variable_expr_node(right_var, 4, 19);
        let op =
            helper::create_comparison_op_node(ast::expression::ComparisonOp::GreaterThan, 2, 3);

        let expr = ast::expression::Expr::ComparisonOp {
            op,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            rest_chained: vec![],
        };
        let expr_node = ast::node::Node::new(&helper::test_span(0, 19), expr);

        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        let result = resolve_expr(&expr_node, &builtin_ref, &context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &Identifier::new("undefined_right"));
                    }
                    _ => panic!("Expected undefined parameter error, got {:?}", errors[0]),
                }
            }
            _ => panic!("Expected error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression_error_from_chained_operand() {
        // Test error propagation from chained comparison operand
        let left_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(1.0), 0, 1);
        let middle_expr =
            helper::create_literal_expr_node(ast::expression::Literal::Number(2.0), 4, 5);
        let chained_var = helper::create_identifier_variable("undefined_chained");
        let chained_expr = helper::create_variable_expr_node(chained_var, 8, 23);

        let op1 = helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 2, 3);
        let op2 = helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 6, 7);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op1,
            left: Box::new(left_expr),
            right: Box::new(middle_expr),
            rest_chained: vec![(op2, chained_expr)],
        };
        let expr_node = ast::node::Node::new(&helper::test_span(0, 23), expr);

        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        let result = resolve_expr(&expr_node, &builtin_ref, &context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &Identifier::new("undefined_chained"));
                    }
                    _ => panic!("Expected undefined parameter error, got {:?}", errors[0]),
                }
            }
            _ => panic!("Expected error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression_multiple_errors() {
        // Test multiple errors from different parts of comparison expression
        let left_var = helper::create_identifier_variable("undefined_left");
        let left_expr = helper::create_variable_expr_node(left_var, 0, 15);
        let right_var = helper::create_identifier_variable("undefined_right");
        let right_expr = helper::create_variable_expr_node(right_var, 18, 33);
        let chained_var = helper::create_identifier_variable("undefined_chained");
        let chained_expr = helper::create_variable_expr_node(chained_var, 36, 51);

        let op1 =
            helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 16, 17);
        let op2 =
            helper::create_comparison_op_node(ast::expression::ComparisonOp::LessThan, 34, 35);

        let expr = ast::expression::Expr::ComparisonOp {
            op: op1,
            left: Box::new(left_expr),
            right: Box::new(right_expr),
            rest_chained: vec![(op2, chained_expr)],
        };
        let expr_node = ast::node::Node::new(&helper::test_span(0, 51), expr);

        let context = TestContext::new();
        let builtin_ref = TestBuiltinRef::new();

        let result = resolve_expr(&expr_node, &builtin_ref, &context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 3);

                let error_identifiers: Vec<_> = errors
                    .iter()
                    .filter_map(|e| {
                        if let VariableResolutionError::UndefinedParameter {
                            model_path: None,
                            parameter,
                            reference_span: _,
                        } = e
                        {
                            Some(parameter.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                assert!(error_identifiers.contains(&Identifier::new("undefined_left")));
                assert!(error_identifiers.contains(&Identifier::new("undefined_right")));
                assert!(error_identifiers.contains(&Identifier::new("undefined_chained")));
            }
            _ => panic!("Expected multiple errors, got {result:?}"),
        }
    }
}
