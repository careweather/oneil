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
mod tests {
    use super::*;
    use crate::test::{
        TestBuiltinRef,
        construct::{ParameterContextBuilder, ReferenceContextBuilder, test_ast, test_ir},
    };

    use oneil_ast as ast;
    use oneil_ir as ir;

    #[test]
    fn test_resolve_literal_number() {
        // create the expression
        let literal = test_ast::literal_number_expr_node(42.0);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::Literal { value } => {
                    assert_eq!(value, &ir::Literal::Number(42.0));
                }
                _ => panic!("Expected literal expression, got {result:?}"),
            },
            _ => panic!("Expected literal expression, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_literal_string() {
        // create the expression
        let literal = test_ast::literal_string_expr_node("hello");

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::Literal { value } => {
                    assert_eq!(value, &ir::Literal::String("hello".to_string()));
                }
                _ => panic!("Expected literal expression, got {result:?}"),
            },
            _ => panic!("Expected literal expression, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_literal_boolean() {
        // create the expression
        let literal = test_ast::literal_boolean_expr_node(true);

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(
            &literal,
            &builtin_ref,
            &reference_context,
            &parameter_context,
        );

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::Literal { value } => {
                    assert_eq!(value, &ir::Literal::Boolean(true));
                }
                _ => panic!("Expected literal expression, got {result:?}"),
            },
            _ => panic!("Expected literal expression, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_binary_op() {
        // create the expression
        let ast_left = test_ast::literal_number_expr_node(1.0);
        let ast_right = test_ast::literal_number_expr_node(2.0);
        let ast_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let expr = test_ast::binary_op_expr_node(ast_op.clone(), ast_left, ast_right);

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::BinaryOp { op, left, right } => {
                    assert_eq!(op.value(), &ir::BinaryOp::Add);

                    match left.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(1.0));
                        }
                        _ => panic!("Expected literal expression on left, got {left:?}"),
                    }

                    match right.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(2.0));
                        }
                        _ => panic!("Expected literal expression on right, got {right:?}"),
                    }
                }
                _ => panic!("Expected binary operation, got {result:?}"),
            },
            _ => panic!("Expected binary operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_unary_op() {
        // create the expression
        let ast_inner_expr = test_ast::literal_number_expr_node(5.0);
        let ast_op = test_ast::unary_op_node(ast::UnaryOp::Neg);
        let expr = test_ast::unary_op_expr_node(ast_op.clone(), ast_inner_expr);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::Expr::UnaryOp { op, expr } => {
                        let expected_op_span = get_span_from_ast_span(ast_op.node_span());
                        assert_eq!(op.span(), expected_op_span);
                        assert_eq!(op.value(), &ir::UnaryOp::Neg);

                        match expr.value() {
                            ir::Expr::Literal { value } => {
                                assert_eq!(value, &ir::Literal::Number(5.0));
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
        let ast_arg = test_ast::literal_number_expr_node(1.0);
        let ast_name = test_ast::identifier_node("foo");
        let expr = test_ast::function_call_expr_node(ast_name.clone(), vec![ast_arg]);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new().with_builtin_functions(["foo"]);

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(ast_name.node_span());
                        assert_eq!(name.span(), expected_name_span);
                        assert_eq!(
                            name.value(),
                            &ir::FunctionName::builtin(ir::Identifier::new("foo"))
                        );

                        assert_eq!(args.len(), 1);

                        match &args[0].value() {
                            ir::Expr::Literal { value } => {
                                assert_eq!(value, &ir::Literal::Number(1.0));
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
        let ast_arg = test_ast::literal_number_expr_node(42.0);
        let ast_name = test_ast::identifier_node("custom_function");
        let expr = test_ast::function_call_expr_node(ast_name.clone(), vec![ast_arg]);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => {
                let expected_span = get_span_from_ast_span(expr.node_span());
                assert_eq!(result.span(), expected_span);

                match result.value() {
                    ir::Expr::FunctionCall { name, args } => {
                        let expected_name_span = get_span_from_ast_span(ast_name.node_span());
                        assert_eq!(name.span(), expected_name_span);
                        assert_eq!(
                            name.value(),
                            &ir::FunctionName::imported(ir::Identifier::new("custom_function"))
                        );

                        assert_eq!(args.len(), 1);

                        match &args[0].value() {
                            ir::Expr::Literal { value } => {
                                assert_eq!(value, &ir::Literal::Number(42.0));
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
        let ast_variable = test_ast::identifier_variable_node("x");
        let expr = test_ast::variable_expr_node(ast_variable);

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::Variable(variable) => {
                    assert_eq!(variable, &ir::Variable::Builtin(ir::Identifier::new("x")));
                }
                _ => panic!("Expected variable expression, got {result:?}"),
            },
            _ => panic!("Expected local variable, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_variable_parameter() {
        // create the expression
        let variable_ast = test_ast::identifier_variable_node("param");
        let expr = test_ast::variable_expr_node(variable_ast);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter = test_ir::ParameterBuilder::new()
            .with_ident_str("param")
            .with_simple_number_value(42.0)
            .build();

        let parameter_context_builder =
            ParameterContextBuilder::new().with_parameter_context([parameter]);
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::Variable(variable) => {
                    assert_eq!(
                        variable,
                        &ir::Variable::Parameter(ir::Identifier::new("param"))
                    );
                }
                _ => panic!("Expected variable expression, got {result:?}"),
            },
            _ => panic!("Expected parameter variable, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_variable_undefined() {
        // create the expression
        let variable_ast = test_ast::identifier_variable_node("undefined");
        let expr = test_ast::variable_expr_node(variable_ast);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &ir::Identifier::new("undefined"));
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
        let ast_left_1 = test_ast::literal_number_expr_node(1.0);
        let ast_right_1 = test_ast::literal_number_expr_node(2.0);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let inner_binary = test_ast::binary_op_expr_node(
            ast_add_op.clone(),
            ast_left_1.clone(),
            ast_right_1.clone(),
        );

        let ast_func_arg = test_ast::literal_number_expr_node(1.0);
        let ast_func_name = test_ast::identifier_node("foo");
        let func_call =
            test_ast::function_call_expr_node(ast_func_name.clone(), vec![ast_func_arg.clone()]);

        let ast_mul_op = test_ast::binary_op_node(ast::BinaryOp::Mul);
        let expr =
            test_ast::binary_op_expr_node(ast_mul_op.clone(), inner_binary.clone(), func_call);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => {
                match result.value() {
                    ir::Expr::BinaryOp { op, left, right } => {
                        assert_eq!(op.value(), &ir::BinaryOp::Mul);

                        // check left side (1 + 2)
                        match left.value() {
                            ir::Expr::BinaryOp {
                                op: left_op,
                                left: left_left,
                                right: left_right,
                            } => {
                                assert_eq!(left_op.value(), &ir::BinaryOp::Add);

                                match left_left.value() {
                                    ir::Expr::Literal { value } => {
                                        assert_eq!(value, &ir::Literal::Number(1.0));
                                    }
                                    _ => panic!(
                                        "Expected literal on left side, got {:?}",
                                        left_left.value()
                                    ),
                                }

                                match left_right.value() {
                                    ir::Expr::Literal { value } => {
                                        assert_eq!(value, &ir::Literal::Number(2.0));
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
                            ir::Expr::FunctionCall { name, args } => {
                                assert_eq!(
                                    name.value(),
                                    &ir::FunctionName::imported(ir::Identifier::new("foo"))
                                );

                                assert_eq!(args.len(), 1);
                                match args[0].value() {
                                    ir::Expr::Literal { value } => {
                                        assert_eq!(value, &ir::Literal::Number(1.0));
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
            (ast::BinaryOp::Add, ir::BinaryOp::Add),
            (ast::BinaryOp::Sub, ir::BinaryOp::Sub),
            (ast::BinaryOp::TrueSub, ir::BinaryOp::TrueSub),
            (ast::BinaryOp::Mul, ir::BinaryOp::Mul),
            (ast::BinaryOp::Div, ir::BinaryOp::Div),
            (ast::BinaryOp::TrueDiv, ir::BinaryOp::TrueDiv),
            (ast::BinaryOp::Mod, ir::BinaryOp::Mod),
            (ast::BinaryOp::Pow, ir::BinaryOp::Pow),
            (ast::BinaryOp::And, ir::BinaryOp::And),
            (ast::BinaryOp::Or, ir::BinaryOp::Or),
            (ast::BinaryOp::MinMax, ir::BinaryOp::MinMax),
        ];

        for (ast_op, expected_ir_op) in operations {
            // create the binary operation node
            let ast_op_node = test_ast::binary_op_node(ast_op);

            // resolve the binary operation
            let result = resolve_binary_op(&ast_op_node);

            // check the result
            assert_eq!(result.value(), &expected_ir_op);
        }
    }

    #[test]
    fn test_resolve_comparison_op_all_operations() {
        // create the operations
        let operations = vec![
            (ast::ComparisonOp::LessThan, ir::ComparisonOp::LessThan),
            (ast::ComparisonOp::LessThanEq, ir::ComparisonOp::LessThanEq),
            (
                ast::ComparisonOp::GreaterThan,
                ir::ComparisonOp::GreaterThan,
            ),
            (
                ast::ComparisonOp::GreaterThanEq,
                ir::ComparisonOp::GreaterThanEq,
            ),
            (ast::ComparisonOp::Eq, ir::ComparisonOp::Eq),
            (ast::ComparisonOp::NotEq, ir::ComparisonOp::NotEq),
        ];

        for (ast_op, expected_ir_op) in operations {
            // create the comparison operation node
            let ast_op_node = test_ast::comparison_op_node(ast_op);

            // resolve the comparison operation
            let result = resolve_comparison_op(&ast_op_node);

            // check the result
            assert_eq!(result.value(), &expected_ir_op);
        }
    }

    #[test]
    fn test_resolve_unary_op_all_operations() {
        // create the operations
        let operations = vec![
            (ast::UnaryOp::Neg, ir::UnaryOp::Neg),
            (ast::UnaryOp::Not, ir::UnaryOp::Not),
        ];

        for (ast_op, expected_ir_op) in operations {
            // create the unary operation node
            let ast_op_node = test_ast::unary_op_node(ast_op);

            // resolve the unary operation
            let result = resolve_unary_op(&ast_op_node);

            // check the result
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
            let ast_func_name_node = test_ast::identifier_node(func_name);

            // resolve the function name
            let result = resolve_function_name(&ast_func_name_node, &builtin_ref);

            // check the result
            let expected_func_builtin = ir::FunctionName::builtin(ir::Identifier::new(func_name));
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
            let ast_func_name_node = test_ast::identifier_node(func_name);

            // resolve the function name
            let result = resolve_function_name(&ast_func_name_node, &builtin_ref);

            // check the result
            match result.value() {
                ir::FunctionName::Imported(name) => {
                    assert_eq!(name.as_str(), func_name);
                }
                ir::FunctionName::Builtin(_) => {
                    panic!("Expected imported function for '{func_name}', got {result:?}")
                }
            }
        }
    }

    #[test]
    fn test_resolve_literal_all_types() {
        // Test number
        let ast_number = test_ast::literal_number_node(42.5);
        let ir_number = resolve_literal(&ast_number);
        assert_eq!(ir_number, ir::Literal::Number(42.5));

        // Test string
        let ast_string = test_ast::literal_string_node("test string");
        let ir_string = resolve_literal(&ast_string);
        assert_eq!(ir_string, ir::Literal::String("test string".to_string()));

        // Test boolean
        let ast_bool = test_ast::literal_boolean_node(false);
        let ir_bool = resolve_literal(&ast_bool);
        assert_eq!(ir_bool, ir::Literal::Boolean(false));
    }

    #[test]
    fn test_resolve_expression_with_errors() {
        // create the expression
        let ast_left_var = test_ast::identifier_variable_node("undefined1");
        let ast_left_expr = test_ast::variable_expr_node(ast_left_var);
        let ast_right_var = test_ast::identifier_variable_node("undefined2");
        let ast_right_expr = test_ast::variable_expr_node(ast_right_var);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let expr = test_ast::binary_op_expr_node(ast_add_op, ast_left_expr, ast_right_expr);

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

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

                assert!(error_identifiers.contains(&ir::Identifier::new("undefined1")));
                assert!(error_identifiers.contains(&ir::Identifier::new("undefined2")));
            }
            _ => panic!("Expected error, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_parenthesized_expression() {
        // Test a simple parenthesized expression: (1 + 2)
        let ast_left = test_ast::literal_number_expr_node(1.0);
        let ast_right = test_ast::literal_number_expr_node(2.0);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let inner_expr =
            test_ast::binary_op_expr_node(ast_add_op.clone(), ast_left.clone(), ast_right.clone());
        let expr = test_ast::parenthesized_expr_node(inner_expr.clone());

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::BinaryOp { op, left, right } => {
                    assert_eq!(op.value(), &ir::BinaryOp::Add);

                    match left.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(1.0));
                        }
                        _ => panic!("Expected literal on left side, got {:?}", left.value()),
                    }

                    match right.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(2.0));
                        }
                        _ => panic!("Expected literal on right side, got {:?}", right.value()),
                    }
                }
                _ => panic!("Expected binary operation, got {:?}", result.value()),
            },
            _ => panic!("Expected successful result, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_nested_parenthesized_expression() {
        // Test nested parentheses: ((1 + 2) * 3)
        let ast_inner_left = test_ast::literal_number_expr_node(1.0);
        let ast_inner_right = test_ast::literal_number_expr_node(2.0);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let inner_binary = test_ast::binary_op_expr_node(
            ast_add_op.clone(),
            ast_inner_left.clone(),
            ast_inner_right.clone(),
        );
        let inner_parenthesized = test_ast::parenthesized_expr_node(inner_binary.clone());
        let ast_outer_right = test_ast::literal_number_expr_node(3.0);
        let ast_mul_op = test_ast::binary_op_node(ast::BinaryOp::Mul);
        let expr = test_ast::binary_op_expr_node(
            ast_mul_op.clone(),
            inner_parenthesized.clone(),
            ast_outer_right.clone(),
        );

        // create the context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => {
                match result.value() {
                    ir::Expr::BinaryOp { op, left, right } => {
                        assert_eq!(op.value(), &ir::BinaryOp::Mul);

                        // check left side ((1 + 2))
                        match left.value() {
                            ir::Expr::BinaryOp {
                                op: left_op,
                                left: left_left,
                                right: left_right,
                            } => {
                                assert_eq!(left_op.value(), &ir::BinaryOp::Add);

                                match left_left.value() {
                                    ir::Expr::Literal { value } => {
                                        assert_eq!(value, &ir::Literal::Number(1.0));
                                    }
                                    _ => panic!(
                                        "Expected literal on left side, got {:?}",
                                        left_left.value()
                                    ),
                                }

                                match left_right.value() {
                                    ir::Expr::Literal { value } => {
                                        assert_eq!(value, &ir::Literal::Number(2.0));
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
                        match right.value() {
                            ir::Expr::Literal { value } => {
                                assert_eq!(value, &ir::Literal::Number(3.0));
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
        let ast_inner_literal = test_ast::literal_number_expr_node(42.0);
        let first_parentheses = test_ast::parenthesized_expr_node(ast_inner_literal.clone());
        let expr = test_ast::parenthesized_expr_node(first_parentheses);

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::Literal { value } => {
                    assert_eq!(value, &ir::Literal::Number(42.0));
                }
                _ => panic!("Expected literal expression, got {:?}", result.value()),
            },
            _ => panic!("Expected successful result, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression() {
        // Test a simple comparison expression: x < 5
        let left_var = test_ast::identifier_variable_node("x");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_expr = test_ast::literal_number_expr_node(5.0);
        let op = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let rest_chained = [];

        let expr = test_ast::comparison_op_expr_node(
            op.clone(),
            left_expr.clone(),
            right_expr.clone(),
            rest_chained,
        );

        // create the context
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::ComparisonOp {
                    op,
                    left,
                    right,
                    rest_chained,
                } => {
                    assert_eq!(op.value(), &ir::ComparisonOp::LessThan);

                    match left.value() {
                        ir::Expr::Variable(variable) => {
                            assert_eq!(variable, &ir::Variable::Builtin(ir::Identifier::new("x")));
                        }
                        _ => panic!("Expected variable expression, got {:?}", left.value()),
                    }

                    match right.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(5.0));
                        }
                        _ => panic!("Expected literal expression, got {:?}", right.value()),
                    }

                    assert_eq!(rest_chained.len(), 0);
                }
                _ => panic!("Expected comparison operation, got {:?}", result.value()),
            },
            _ => panic!("Expected comparison operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_chained_comparison_expression() {
        // Test a chained comparison expression: 1 < x < 10
        let left_expr = test_ast::literal_number_expr_node(1.0);
        let middle_var = test_ast::identifier_variable_node("x");
        let middle_expr = test_ast::variable_expr_node(middle_var);
        let right_expr = test_ast::literal_number_expr_node(10.0);

        let op1 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr = test_ast::comparison_op_expr_node(
            op1.clone(),
            left_expr.clone(),
            middle_expr.clone(),
            [(op2.clone(), right_expr.clone())],
        );

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        match result {
            Ok(result) => match result.value() {
                ir::Expr::ComparisonOp {
                    op,
                    left,
                    right,
                    rest_chained,
                } => {
                    assert_eq!(op.value(), &ir::ComparisonOp::LessThan);

                    match left.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(1.0));
                        }
                        _ => panic!("Expected literal expression, got {:?}", left.value()),
                    }

                    match right.value() {
                        ir::Expr::Variable(variable) => {
                            assert_eq!(variable, &ir::Variable::Builtin(ir::Identifier::new("x")));
                        }
                        _ => panic!("Expected variable expression, got {:?}", right.value()),
                    }

                    assert_eq!(rest_chained.len(), 1);
                    let (chained_op, chained_expr) = &rest_chained[0];
                    assert_eq!(chained_op.value(), &ir::ComparisonOp::LessThan);

                    match chained_expr.value() {
                        ir::Expr::Literal { value } => {
                            assert_eq!(value, &ir::Literal::Number(10.0));
                        }
                        _ => panic!(
                            "Expected literal expression in chained comparison, got {:?}",
                            chained_expr.value()
                        ),
                    }
                }
                _ => panic!("Expected comparison operation, got {:?}", result.value()),
            },
            _ => panic!("Expected comparison operation, got {result:?}"),
        }
    }

    #[test]
    fn test_resolve_comparison_expression_error_from_left_operand() {
        // Test error propagation from left operand of comparison expression
        let left_var = test_ast::identifier_variable_node("undefined_left");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_expr = test_ast::literal_number_expr_node(5.0);
        let op = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let rest_chained = [];

        let expr = test_ast::comparison_op_expr_node(
            op,
            left_expr.clone(),
            right_expr.clone(),
            rest_chained,
        );

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &ir::Identifier::new("undefined_left"));
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
        let left_expr = test_ast::literal_number_expr_node(1.0);
        let right_var = test_ast::identifier_variable_node("undefined_right");
        let right_expr = test_ast::variable_expr_node(right_var);
        let op = test_ast::comparison_op_node(ast::ComparisonOp::GreaterThan);

        let expr = test_ast::comparison_op_expr_node(op, left_expr.clone(), right_expr.clone(), []);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &ir::Identifier::new("undefined_right"));
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
        let left_expr = test_ast::literal_number_expr_node(1.0);
        let middle_expr = test_ast::literal_number_expr_node(2.0);
        let chained_var = test_ast::identifier_variable_node("undefined_chained");
        let chained_expr = test_ast::variable_expr_node(chained_var);

        let op1 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr = test_ast::comparison_op_expr_node(
            op1.clone(),
            left_expr.clone(),
            middle_expr.clone(),
            [(op2.clone(), chained_expr.clone())],
        );

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        match result {
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                match &errors[0] {
                    VariableResolutionError::UndefinedParameter {
                        model_path: None,
                        parameter,
                        reference_span: _,
                    } => {
                        assert_eq!(parameter, &ir::Identifier::new("undefined_chained"));
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
        let left_var = test_ast::identifier_variable_node("undefined_left");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_var = test_ast::identifier_variable_node("undefined_right");
        let right_expr = test_ast::variable_expr_node(right_var);
        let chained_var = test_ast::identifier_variable_node("undefined_chained");
        let chained_expr = test_ast::variable_expr_node(chained_var);

        let op1 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr = test_ast::comparison_op_expr_node(
            op1.clone(),
            left_expr.clone(),
            right_expr.clone(),
            [(op2.clone(), chained_expr.clone())],
        );

        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

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

                assert!(error_identifiers.contains(&ir::Identifier::new("undefined_left")));
                assert!(error_identifiers.contains(&ir::Identifier::new("undefined_right")));
                assert!(error_identifiers.contains(&ir::Identifier::new("undefined_chained")));
            }
            _ => panic!("Expected multiple errors, got {result:?}"),
        }
    }
}
