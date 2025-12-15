//! Expression resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    BuiltinRef,
    error::{self, VariableResolutionError},
    resolver::resolve_variable::resolve_variable,
    util::context::{ParameterContext, ReferenceContext},
};

/// Resolves an AST expression into a model expression.
pub fn resolve_expr(
    value: &ast::ExprNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
    let span = value.span();

    match &**value {
        ast::Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } => resolve_comparison_expression(
            span,
            op,
            left,
            right,
            rest_chained,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::BinaryOp { op, left, right } => resolve_binary_expression(
            span,
            op,
            left,
            right,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::UnaryOp { op, expr } => resolve_unary_expression(
            span,
            op,
            expr,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::FunctionCall { name, args } => resolve_function_call_expression(
            span,
            name,
            args,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
        ast::Expr::Variable(variable) => {
            resolve_variable_expression(variable, builtin_ref, reference_context, parameter_context)
        }
        ast::Expr::Literal(literal) => Ok(resolve_literal_expression(span, literal)),
        ast::Expr::Parenthesized { expr } => resolve_parenthesized_expression(
            expr,
            builtin_ref,
            reference_context,
            parameter_context,
        ),
    }
}

/// Resolves a comparison expression with optional chained comparisons.
fn resolve_comparison_expression(
    span: Span,
    op: &ast::ComparisonOpNode,
    left: &ast::ExprNode,
    right: &ast::ExprNode,
    rest_chained: &[(ast::ComparisonOpNode, ast::ExprNode)],
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
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

    let expr = ir::Expr::comparison_op(span, op_with_span, left, right, rest_chained);
    Ok(expr)
}

/// Resolves a binary operation expression.
fn resolve_binary_expression(
    span: Span,
    op: &ast::BinaryOpNode,
    left: &ast::ExprNode,
    right: &ast::ExprNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
    let left = resolve_expr(left, builtin_ref, reference_context, parameter_context);
    let right = resolve_expr(right, builtin_ref, reference_context, parameter_context);
    let op_with_span = resolve_binary_op(op);

    let (left, right) = error::combine_errors(left, right)?;

    let expr = ir::Expr::binary_op(span, op_with_span, left, right);
    Ok(expr)
}

/// Resolves a unary operation expression.
fn resolve_unary_expression(
    span: Span,
    op: &ast::UnaryOpNode,
    expr: &ast::ExprNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
    let expr = resolve_expr(expr, builtin_ref, reference_context, parameter_context);
    let op_with_span = resolve_unary_op(op);

    match expr {
        Ok(expr) => Ok(ir::Expr::unary_op(span, op_with_span, expr)),
        Err(errors) => Err(errors),
    }
}

/// Resolves a function call expression.
fn resolve_function_call_expression(
    span: Span,
    name: &ast::IdentifierNode,
    args: &[ast::ExprNode],
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
    let name_with_span = resolve_function_name(name, builtin_ref);
    let args = args
        .iter()
        .map(|arg| resolve_expr(arg, builtin_ref, reference_context, parameter_context));

    let args = error::combine_error_list(args)?;

    let expr = ir::Expr::function_call(span, name.span(), name_with_span, args);
    Ok(expr)
}

/// Resolves a variable expression.
fn resolve_variable_expression(
    variable: &ast::VariableNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
    resolve_variable(variable, builtin_ref, reference_context, parameter_context)
        .map_err(|error| vec![error])
}

/// Resolves a literal expression.
fn resolve_literal_expression(span: Span, literal: &ast::LiteralNode) -> ir::Expr {
    let literal = resolve_literal(literal);
    ir::Expr::literal(span, literal)
}

/// Resolves a parenthesized expression.
fn resolve_parenthesized_expression(
    expr: &ast::ExprNode,
    builtin_ref: &impl BuiltinRef,
    reference_context: &ReferenceContext<'_, '_>,
    parameter_context: &ParameterContext<'_>,
) -> Result<ir::Expr, Vec<VariableResolutionError>> {
    resolve_expr(expr, builtin_ref, reference_context, parameter_context)
}

/// Converts an AST comparison operation to a model comparison operation.
fn resolve_comparison_op(op: &ast::ComparisonOpNode) -> ir::ComparisonOp {
    match &**op {
        ast::ComparisonOp::LessThan => ir::ComparisonOp::LessThan,
        ast::ComparisonOp::LessThanEq => ir::ComparisonOp::LessThanEq,
        ast::ComparisonOp::GreaterThan => ir::ComparisonOp::GreaterThan,
        ast::ComparisonOp::GreaterThanEq => ir::ComparisonOp::GreaterThanEq,
        ast::ComparisonOp::Eq => ir::ComparisonOp::Eq,
        ast::ComparisonOp::NotEq => ir::ComparisonOp::NotEq,
    }
}

/// Converts an AST binary operation to a model binary operation.
fn resolve_binary_op(op: &ast::BinaryOpNode) -> ir::BinaryOp {
    match &**op {
        ast::BinaryOp::Add => ir::BinaryOp::Add,
        ast::BinaryOp::Sub => ir::BinaryOp::Sub,
        ast::BinaryOp::EscapedSub => ir::BinaryOp::EscapedSub,
        ast::BinaryOp::Mul => ir::BinaryOp::Mul,
        ast::BinaryOp::Div => ir::BinaryOp::Div,
        ast::BinaryOp::EscapedDiv => ir::BinaryOp::EscapedDiv,
        ast::BinaryOp::Mod => ir::BinaryOp::Mod,
        ast::BinaryOp::Pow => ir::BinaryOp::Pow,
        ast::BinaryOp::And => ir::BinaryOp::And,
        ast::BinaryOp::Or => ir::BinaryOp::Or,
        ast::BinaryOp::MinMax => ir::BinaryOp::MinMax,
    }
}

/// Converts an AST unary operation to a model unary operation.
fn resolve_unary_op(op: &ast::UnaryOpNode) -> ir::UnaryOp {
    match &**op {
        ast::UnaryOp::Neg => ir::UnaryOp::Neg,
        ast::UnaryOp::Not => ir::UnaryOp::Not,
    }
}

/// Resolves a function name to a model function name.
fn resolve_function_name(
    name: &ast::IdentifierNode,
    builtin_ref: &impl BuiltinRef,
) -> ir::FunctionName {
    let name = ir::Identifier::new(name.as_str().to_string());

    if builtin_ref.has_builtin_function(&name) {
        ir::FunctionName::builtin(name)
    } else {
        ir::FunctionName::imported(name)
    }
}

/// Converts an AST literal to a model literal.
fn resolve_literal(literal: &ast::LiteralNode) -> ir::Literal {
    match &**literal {
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
    fn resolve_literal_number() {
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
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::Literal { span: _, value } = result else {
            panic!("Expected literal expression, got {result:?}");
        };

        assert_eq!(value, ir::Literal::Number(42.0));
    }

    #[test]
    fn resolve_literal_string() {
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
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::Literal { span: _, value } = result else {
            panic!("Expected literal expression, got {result:?}");
        };

        assert_eq!(value, ir::Literal::String("hello".to_string()));
    }

    #[test]
    fn resolve_literal_boolean() {
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
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::Literal { span: _, value } = result else {
            panic!("Expected literal expression, got {result:?}");
        };

        assert_eq!(value, ir::Literal::Boolean(true));
    }

    #[test]
    fn resolve_binary_op_() {
        // create the expression
        let ast_left = test_ast::literal_number_expr_node(1.0);
        let ast_right = test_ast::literal_number_expr_node(2.0);
        let ast_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let expr = test_ast::binary_op_expr_node(ast_op, ast_left, ast_right);

        // create the context and builtin ref
        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let builtin_ref = TestBuiltinRef::new();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::BinaryOp {
            span: _,
            op,
            left,
            right,
        } = result
        else {
            panic!("Expected binary operation, got {result:?}");
        };

        assert_eq!(op, ir::BinaryOp::Add);

        let ir::Expr::Literal { span: _, value } = *left else {
            panic!("Expected literal expression on left, got {left:?}");
        };
        assert_eq!(value, ir::Literal::Number(1.0));

        let ir::Expr::Literal { span: _, value } = *right else {
            panic!("Expected literal expression on right, got {right:?}");
        };
        assert_eq!(value, ir::Literal::Number(2.0));
    }

    #[test]
    fn resolve_unary_op_() {
        // create the expression
        let ast_inner_expr = test_ast::literal_number_expr_node(5.0);
        let ast_op = test_ast::unary_op_node(ast::UnaryOp::Neg);
        let expr = test_ast::unary_op_expr_node(ast_op, ast_inner_expr);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::UnaryOp { span: _, op, expr } = result else {
            panic!("Expected unary operation, got {result:?}");
        };

        assert_eq!(op, ir::UnaryOp::Neg);

        let ir::Expr::Literal { span: _, value } = *expr else {
            panic!("Expected literal expression, got {expr:?}");
        };
        assert_eq!(value, ir::Literal::Number(5.0));
    }

    #[test]
    fn resolve_function_call_builtin() {
        // create the expression
        let ast_arg = test_ast::literal_number_expr_node(1.0);
        let ast_name = test_ast::identifier_node("foo");
        let expr = test_ast::function_call_expr_node(ast_name, vec![ast_arg]);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new().with_builtin_functions(["foo"]);

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::FunctionCall {
            span: _,
            name_span: _,
            name,
            mut args,
        } = result
        else {
            panic!("Expected function call, got {result:?}");
        };

        assert_eq!(
            name,
            ir::FunctionName::builtin(ir::Identifier::new("foo".to_string()))
        );

        assert_eq!(args.len(), 1);

        let ir::Expr::Literal { span: _, value } = args.remove(0) else {
            panic!("Expected literal argument, got {:?}", args[0]);
        };

        assert_eq!(value, ir::Literal::Number(1.0));
    }

    #[test]
    fn resolve_function_call_imported() {
        // create the expression
        let ast_arg = test_ast::literal_number_expr_node(42.0);
        let ast_name = test_ast::identifier_node("custom_function");
        let expr = test_ast::function_call_expr_node(ast_name, vec![ast_arg]);

        // create the builtin ref and contexts
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::FunctionCall {
            span: _,
            name_span: _,
            name,
            mut args,
        } = result
        else {
            panic!("Expected function call, got {result:?}");
        };

        assert_eq!(
            name,
            ir::FunctionName::imported(ir::Identifier::new("custom_function".to_string()))
        );

        assert_eq!(args.len(), 1);

        let ir::Expr::Literal { span: _, value } = args.remove(0) else {
            panic!("Expected literal argument, got {:?}", args[0]);
        };

        assert_eq!(value, ir::Literal::Number(42.0));
    }

    #[test]
    fn resolve_variable_builtin() {
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
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::Variable { span: _, variable } = result else {
            panic!("Expected variable expression, got {result:?}");
        };

        let ir::Variable::Builtin { ident, .. } = variable else {
            panic!("Expected builtin variable, got {variable:?}");
        };

        assert_eq!(ident.as_str(), "x");
    }

    #[test]
    fn resolve_variable_parameter() {
        // create the expression
        let variable_ast = test_ast::identifier_variable_node("param");
        let expr = test_ast::variable_expr_node(variable_ast);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter = test_ir::ParameterBuilder::new()
            .with_name_str("param")
            .with_simple_number_value(42.0)
            .build();

        let parameter_context_builder =
            ParameterContextBuilder::new().with_parameter_context([parameter]);
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::Variable { span: _, variable } = result else {
            panic!("Expected variable expression, got {result:?}");
        };

        let ir::Variable::Parameter { parameter_name, .. } = variable else {
            panic!("Expected parameter variable, got {variable:?}");
        };

        assert_eq!(parameter_name.as_str(), "param");
    }

    #[test]
    fn resolve_variable_undefined() {
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
        let Err(errors) = result else {
            panic!("Expected error, got {result:?}");
        };

        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        let VariableResolutionError::UndefinedParameter {
            model_path: None,
            parameter_name,
            reference_span: _,
        } = error
        else {
            panic!("Expected undefined parameter error, got {error:?}");
        };

        assert_eq!(parameter_name.as_str(), "undefined");
    }

    #[test]
    fn resolve_complex_expression() {
        // create the expression: (1 + 2) * foo(1)
        let ast_left_1 = test_ast::literal_number_expr_node(1.0);
        let ast_right_1 = test_ast::literal_number_expr_node(2.0);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let inner_binary = test_ast::binary_op_expr_node(ast_add_op, ast_left_1, ast_right_1);

        let ast_func_arg = test_ast::literal_number_expr_node(1.0);
        let ast_func_name = test_ast::identifier_node("foo");
        let func_call = test_ast::function_call_expr_node(ast_func_name, vec![ast_func_arg]);

        let ast_mul_op = test_ast::binary_op_node(ast::BinaryOp::Mul);
        let expr = test_ast::binary_op_expr_node(ast_mul_op, inner_binary, func_call);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::BinaryOp {
            span: _,
            op,
            left,
            right,
        } = result
        else {
            panic!("Expected binary operation, got {result:?}");
        };

        assert_eq!(op, ir::BinaryOp::Mul);

        // check left side (1 + 2)
        let ir::Expr::BinaryOp {
            span: _,
            op: left_op,
            left: left_left,
            right: left_right,
        } = *left
        else {
            panic!("Expected binary operation on left side, got {left:?}");
        };

        assert_eq!(left_op, ir::BinaryOp::Add);

        let ir::Expr::Literal { span: _, value } = *left_left else {
            panic!("Expected literal on left side, got {left_left:?}");
        };
        assert_eq!(value, ir::Literal::Number(1.0));

        let ir::Expr::Literal { span: _, value } = *left_right else {
            panic!("Expected literal on right side, got {left_right:?}");
        };
        assert_eq!(value, ir::Literal::Number(2.0));

        // check right side (foo(3.14))
        let ir::Expr::FunctionCall {
            span: _,
            name_span: _,
            name,
            mut args,
        } = *right
        else {
            panic!("Expected function call on right side, got {right:?}");
        };

        assert_eq!(
            name,
            ir::FunctionName::imported(ir::Identifier::new("foo".to_string()))
        );

        assert_eq!(args.len(), 1);

        let ir::Expr::Literal { span: _, value } = args.remove(0) else {
            panic!("Expected literal argument, got {:?}", args[0]);
        };

        assert_eq!(value, ir::Literal::Number(1.0));
    }

    #[test]
    fn resolve_binary_op_all_operations() {
        // create the operations
        let operations = vec![
            (ast::BinaryOp::Add, ir::BinaryOp::Add),
            (ast::BinaryOp::Sub, ir::BinaryOp::Sub),
            (ast::BinaryOp::EscapedSub, ir::BinaryOp::EscapedSub),
            (ast::BinaryOp::Mul, ir::BinaryOp::Mul),
            (ast::BinaryOp::Div, ir::BinaryOp::Div),
            (ast::BinaryOp::EscapedDiv, ir::BinaryOp::EscapedDiv),
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
            assert_eq!(result, expected_ir_op);
        }
    }

    #[test]
    fn resolve_comparison_op_all_operations() {
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
            assert_eq!(result, expected_ir_op);
        }
    }

    #[test]
    fn resolve_unary_op_all_operations() {
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
            assert_eq!(result, expected_ir_op);
        }
    }

    #[test]
    fn resolve_function_name_builtin() {
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
            let expected_func_builtin =
                ir::FunctionName::builtin(ir::Identifier::new(func_name.to_string()));
            assert_eq!(result, expected_func_builtin);
        }
    }

    #[test]
    fn resolve_function_name_imported() {
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
            let ir::FunctionName::Imported(name) = result else {
                panic!("Expected imported function, got {result:?}");
            };

            assert_eq!(name.as_str(), func_name);
        }
    }

    #[test]
    fn resolve_literal_all_types() {
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
    fn resolve_expression_with_errors() {
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
        let Err(errors) = result else {
            panic!("Expected error, got {result:?}");
        };

        assert_eq!(errors.len(), 2);

        let error_identifiers: Vec<_> = errors
            .iter()
            .filter_map(|e| {
                if let VariableResolutionError::UndefinedParameter {
                    model_path: None,
                    parameter_name,
                    reference_span: _,
                } = e
                {
                    Some(parameter_name.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(error_identifiers.contains(&ir::ParameterName::new("undefined1".to_string())));
        assert!(error_identifiers.contains(&ir::ParameterName::new("undefined2".to_string())));
    }

    #[test]
    fn resolve_parenthesized_expression() {
        // Test a simple parenthesized expression: (1 + 2)
        let ast_left = test_ast::literal_number_expr_node(1.0);
        let ast_right = test_ast::literal_number_expr_node(2.0);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let inner_expr = test_ast::binary_op_expr_node(ast_add_op, ast_left, ast_right);
        let expr = test_ast::parenthesized_expr_node(inner_expr);

        // create the builtin ref and context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::BinaryOp {
            span: _,
            op,
            left,
            right,
        } = result
        else {
            panic!("Expected binary operation, got {result:?}");
        };
        assert_eq!(op, ir::BinaryOp::Add);

        let ir::Expr::Literal { span: _, value } = *left else {
            panic!("Expected literal on left side, got {left:?}");
        };
        assert_eq!(value, ir::Literal::Number(1.0));

        let ir::Expr::Literal { span: _, value } = *right else {
            panic!("Expected literal on right side, got {right:?}");
        };
        assert_eq!(value, ir::Literal::Number(2.0));
    }

    #[test]
    fn resolve_nested_parenthesized_expression() {
        // Test nested parentheses: ((1 + 2) * 3)
        let ast_inner_left = test_ast::literal_number_expr_node(1.0);
        let ast_inner_right = test_ast::literal_number_expr_node(2.0);
        let ast_add_op = test_ast::binary_op_node(ast::BinaryOp::Add);
        let inner_binary =
            test_ast::binary_op_expr_node(ast_add_op, ast_inner_left, ast_inner_right);
        let inner_parenthesized = test_ast::parenthesized_expr_node(inner_binary);
        let ast_outer_right = test_ast::literal_number_expr_node(3.0);
        let ast_mul_op = test_ast::binary_op_node(ast::BinaryOp::Mul);
        let expr = test_ast::binary_op_expr_node(ast_mul_op, inner_parenthesized, ast_outer_right);

        // create the context
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::BinaryOp {
            span: _,
            op,
            left,
            right,
        } = result
        else {
            panic!("Expected binary operation, got {result:?}");
        };
        assert_eq!(op, ir::BinaryOp::Mul);

        let ir::Expr::BinaryOp {
            span: _,
            op: left_op,
            left: left_left,
            right: left_right,
        } = *left
        else {
            panic!("Expected binary operation on left side, got {left:?}");
        };
        assert_eq!(left_op, ir::BinaryOp::Add);

        let ir::Expr::Literal { span: _, value } = *left_left else {
            panic!("Expected literal on left side, got {left_left:?}");
        };
        assert_eq!(value, ir::Literal::Number(1.0));

        let ir::Expr::Literal { span: _, value } = *left_right else {
            panic!("Expected literal on right side, got {left_right:?}");
        };
        assert_eq!(value, ir::Literal::Number(2.0));

        let ir::Expr::Literal { span: _, value } = *right else {
            panic!("Expected literal on right side, got {right:?}");
        };
        assert_eq!(value, ir::Literal::Number(3.0));
    }

    #[test]
    fn resolve_single_literal_multiple_parentheses() {
        // Test a single literal wrapped in multiple parentheses: ((42))
        let ast_inner_literal = test_ast::literal_number_expr_node(42.0);
        let first_parentheses = test_ast::parenthesized_expr_node(ast_inner_literal);
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
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::Literal { span: _, value } = result else {
            panic!("Expected literal expression, got {result:?}");
        };
        assert_eq!(value, ir::Literal::Number(42.0));
    }

    #[test]
    fn resolve_comparison_expression() {
        // Test a simple comparison expression: x < 5
        let left_var = test_ast::identifier_variable_node("x");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_expr = test_ast::literal_number_expr_node(5.0);
        let op = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let rest_chained = [];

        let expr = test_ast::comparison_op_expr_node(op, left_expr, right_expr, rest_chained);

        // create the context
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::ComparisonOp {
            span: _,
            op,
            left,
            right,
            rest_chained,
        } = result
        else {
            panic!("Expected comparison operation, got {result:?}");
        };
        assert_eq!(op, ir::ComparisonOp::LessThan);

        let ir::Expr::Variable { span: _, variable } = *left else {
            panic!("Expected variable expression, got {left:?}");
        };
        let ir::Variable::Builtin { ident, .. } = variable else {
            panic!("Expected builtin variable, got {variable:?}");
        };
        assert_eq!(ident.as_str(), "x");

        let ir::Expr::Literal { span: _, value } = *right else {
            panic!("Expected literal expression, got {right:?}");
        };
        assert_eq!(value, ir::Literal::Number(5.0));

        assert!(rest_chained.is_empty());
    }

    #[test]
    fn resolve_chained_comparison_expression() {
        // Test a chained comparison expression: 1 < x < 10
        let left_expr = test_ast::literal_number_expr_node(1.0);
        let middle_var = test_ast::identifier_variable_node("x");
        let middle_expr = test_ast::variable_expr_node(middle_var);
        let right_expr = test_ast::literal_number_expr_node(10.0);

        let op1 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr =
            test_ast::comparison_op_expr_node(op1, left_expr, middle_expr, [(op2, right_expr)]);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new().with_builtin_variables(["x"]);

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        // resolve the expression
        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        // check the result
        let Ok(result) = result else {
            panic!("Expected successful result, got {result:?}");
        };

        let ir::Expr::ComparisonOp {
            span: _,
            op,
            left,
            right,
            mut rest_chained,
        } = result
        else {
            panic!("Expected comparison operation, got {result:?}");
        };

        assert_eq!(op, ir::ComparisonOp::LessThan);

        let ir::Expr::Literal { span: _, value } = *left else {
            panic!("Expected literal expression, got {left:?}");
        };
        assert_eq!(value, ir::Literal::Number(1.0));

        let ir::Expr::Variable { span: _, variable } = *right else {
            panic!("Expected variable expression, got {right:?}");
        };
        let ir::Variable::Builtin { ident, .. } = variable else {
            panic!("Expected builtin variable, got {variable:?}");
        };
        assert_eq!(ident.as_str(), "x");

        assert_eq!(rest_chained.len(), 1);
        let (chained_op, chained_expr) = rest_chained.remove(0);
        assert_eq!(chained_op, ir::ComparisonOp::LessThan);

        let ir::Expr::Literal { span: _, value } = chained_expr else {
            panic!("Expected literal expression, got {chained_expr:?}");
        };
        assert_eq!(value, ir::Literal::Number(10.0));
    }

    #[test]
    fn resolve_comparison_expression_error_from_left_operand() {
        // Test error propagation from left operand of comparison expression
        let left_var = test_ast::identifier_variable_node("undefined_left");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_expr = test_ast::literal_number_expr_node(5.0);
        let op = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let rest_chained = [];

        let expr = test_ast::comparison_op_expr_node(op, left_expr, right_expr, rest_chained);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        let Err(errors) = result else {
            panic!("Expected error, got {result:?}");
        };

        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        let VariableResolutionError::UndefinedParameter {
            model_path: None,
            parameter_name,
            reference_span: _,
        } = error
        else {
            panic!("Expected undefined parameter error, got {error:?}");
        };

        assert_eq!(parameter_name.as_str(), "undefined_left");
    }

    #[test]
    fn resolve_comparison_expression_error_from_right_operand() {
        // Test error propagation from right operand of comparison expression
        let left_expr = test_ast::literal_number_expr_node(1.0);
        let right_var = test_ast::identifier_variable_node("undefined_right");
        let right_expr = test_ast::variable_expr_node(right_var);
        let op = test_ast::comparison_op_node(ast::ComparisonOp::GreaterThan);

        let expr = test_ast::comparison_op_expr_node(op, left_expr, right_expr, []);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        let Err(errors) = result else {
            panic!("Expected error, got {result:?}");
        };

        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        let VariableResolutionError::UndefinedParameter {
            model_path: None,
            parameter_name,
            reference_span: _,
        } = error
        else {
            panic!("Expected undefined parameter error, got {error:?}");
        };

        assert_eq!(parameter_name.as_str(), "undefined_right");
    }

    #[test]
    fn resolve_comparison_expression_error_from_chained_operand() {
        // Test error propagation from chained comparison operand
        let left_expr = test_ast::literal_number_expr_node(1.0);
        let middle_expr = test_ast::literal_number_expr_node(2.0);
        let chained_var = test_ast::identifier_variable_node("undefined_chained");
        let chained_expr = test_ast::variable_expr_node(chained_var);

        let op1 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr =
            test_ast::comparison_op_expr_node(op1, left_expr, middle_expr, [(op2, chained_expr)]);

        // create the context and builtin ref
        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        let Err(errors) = result else {
            panic!("Expected error, got {result:?}");
        };

        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        let VariableResolutionError::UndefinedParameter {
            model_path: None,
            parameter_name,
            reference_span: _,
        } = error
        else {
            panic!("Expected undefined parameter error, got {error:?}");
        };

        assert_eq!(parameter_name.as_str(), "undefined_chained");
    }

    #[test]
    fn resolve_comparison_expression_multiple_errors() {
        // Test multiple errors from different parts of comparison expression
        let left_var = test_ast::identifier_variable_node("undefined_left");
        let left_expr = test_ast::variable_expr_node(left_var);
        let right_var = test_ast::identifier_variable_node("undefined_right");
        let right_expr = test_ast::variable_expr_node(right_var);
        let chained_var = test_ast::identifier_variable_node("undefined_chained");
        let chained_expr = test_ast::variable_expr_node(chained_var);

        let op1 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);
        let op2 = test_ast::comparison_op_node(ast::ComparisonOp::LessThan);

        let expr =
            test_ast::comparison_op_expr_node(op1, left_expr, right_expr, [(op2, chained_expr)]);

        let builtin_ref = TestBuiltinRef::new();

        let reference_context_builder = ReferenceContextBuilder::new();
        let reference_context = reference_context_builder.build();

        let parameter_context_builder = ParameterContextBuilder::new();
        let parameter_context = parameter_context_builder.build();

        let result = resolve_expr(&expr, &builtin_ref, &reference_context, &parameter_context);

        let Err(errors) = result else {
            panic!("Expected error, got {result:?}");
        };

        assert_eq!(errors.len(), 3);

        let error_identifiers: Vec<_> = errors
            .iter()
            .filter_map(|e| {
                if let VariableResolutionError::UndefinedParameter {
                    model_path: None,
                    parameter_name,
                    reference_span: _,
                } = e
                {
                    Some(parameter_name.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(error_identifiers.contains(&ir::ParameterName::new("undefined_left".to_string())));
        assert!(error_identifiers.contains(&ir::ParameterName::new("undefined_right".to_string())));
        assert!(
            error_identifiers.contains(&ir::ParameterName::new("undefined_chained".to_string()))
        );
    }
}
