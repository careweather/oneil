//! Expression parsing for the Oneil language.

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, map, opt},
    multi::{many0, separated_list0},
};

use oneil_ast::{
    BinaryOp, BinaryOpNode, ComparisonOp, Expr, ExprNode, IdentifierNode, Literal, Node, UnaryOp,
    Variable,
};
use oneil_shared::span::Span;

use crate::{
    error::{ErrorHandlingParser, ParserError},
    token::{
        keyword::{and, false_, not, or, true_},
        literal::{number, string},
        naming::identifier,
        symbol::{
            bang_equals, bar, caret, comma, dot, equals_equals, greater_than, greater_than_equals,
            less_than, less_than_equals, minus, minus_minus, paren_left, paren_right, percent,
            plus, slash, slash_slash, star,
        },
    },
    util::{InputSpan, Parser, Result},
};

/// Creates a left-associative binary operator parser.
///
/// This function constructs a parser that handles left-associative binary operations
/// like addition, multiplication, etc. It parses a sequence of operands separated
/// by operators and builds the AST with proper left associativity.
///
/// For example, `a + b + c` is parsed as `(a + b) + c` rather than `a + (b + c)`.
///
/// # Arguments
///
/// * `operand` - Parser for the operands (e.g., expressions of higher precedence)
/// * `operator` - Parser for the binary operators
///
/// # Returns
///
/// A parser that handles left-associative binary operations with proper error handling.
fn left_associative_binary_op<'a>(
    mut operand: impl Parser<'a, ExprNode, ParserError> + Copy,
    mut operator: impl Parser<'a, BinaryOpNode, ParserError>,
) -> impl Parser<'a, ExprNode, ParserError> {
    move |input| {
        let (rest, first_operand) = operand.parse(input)?;
        let (rest, rest_operands) = many0(|input| {
            let (rest, operator) = operator.parse(input)?;
            let (rest, operand) = operand
                .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                    &operator,
                ))
                .parse(rest)?;
            Ok((rest, (operator, operand)))
        })
        .parse(rest)?;

        let expr = rest_operands
            .into_iter()
            .fold(first_operand, |acc, (op, expr)| {
                let left = acc;
                let right = expr;

                let span = Span::from_start_and_end(&left.span(), &right.span());
                let whitespace_span = right.whitespace_span();

                Node::new(Expr::binary_op(op, left, right), span, whitespace_span)
            });

        Ok((rest, expr))
    }
}

/// Parses an expression
///
/// This function **may not consume the complete input**.
pub fn parse(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    expr(input)
}

/// Parses an expression
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    all_consuming(expr).parse(input)
}

/// Parses an expression with proper operator precedence.
///
/// This function is the entry point for expression parsing and delegates
/// to the highest precedence level (OR expressions). The precedence hierarchy
/// from lowest to highest is:
///
/// 1. OR (`or`)
/// 2. AND (`and`)
/// 3. NOT (`not`)
/// 4. Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`)
/// 5. Min/Max (`|`)
/// 6. Addition/Subtraction (`+`, `-`, `--`)
/// 7. Multiplication/Division (`*`, `/`, `//`, `%`)
/// 8. Exponentiation (`^`)
/// 9. Negation (`-`)
/// 10. Primary expressions (literals, variables, function calls, parentheses)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns an expression node with proper operator precedence.
fn expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    or_expr
        .convert_error_to(ParserError::expect_expr)
        .parse(input)
}

/// Parses an OR expression (lowest precedence)
fn or_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let or = or
        .map(|token| token.into_node_with_value(BinaryOp::Or))
        .convert_errors();
    left_associative_binary_op(and_expr, or).parse(input)
}

/// Parses an AND expression
fn and_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let and = and
        .map(|token| token.into_node_with_value(BinaryOp::And))
        .convert_errors();
    left_associative_binary_op(not_expr, and).parse(input)
}

/// Parses a NOT expression
fn not_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    alt((
        |input| {
            let (rest, not_op) = not
                .map(|token| token.into_node_with_value(UnaryOp::Not))
                .convert_errors()
                .parse(input)?;

            let (rest, expr) = not_expr
                .or_fail_with(ParserError::unary_op_missing_operand(&not_op))
                .parse(rest)?;

            let span = Span::from_start_and_end(&not_op.span(), &expr.span());
            let whitespace_span = expr.whitespace_span();

            Ok((
                rest,
                Node::new(Expr::unary_op(not_op, expr), span, whitespace_span),
            ))
        },
        comparison_expr,
    ))
    .parse(input)
}

/// Parses a comparison expression
fn comparison_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let mut op = alt((
        less_than_equals.map(|token| token.into_node_with_value(ComparisonOp::LessThanEq)),
        greater_than_equals.map(|token| token.into_node_with_value(ComparisonOp::GreaterThanEq)),
        less_than.map(|token| token.into_node_with_value(ComparisonOp::LessThan)),
        greater_than.map(|token| token.into_node_with_value(ComparisonOp::GreaterThan)),
        equals_equals.map(|token| token.into_node_with_value(ComparisonOp::Eq)),
        bang_equals.map(|token| token.into_node_with_value(ComparisonOp::NotEq)),
    ))
    .convert_errors();

    let (rest, first_operand) = minmax_expr.parse(input)?;
    let (rest, rest_operands) = many0(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = minmax_expr
            .or_fail_with(ParserError::expr_comparison_op_missing_second_operand(
                &operator,
            ))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let mut rest_operands = rest_operands.into_iter();

    let maybe_second_operand = rest_operands.next();

    let expr = match maybe_second_operand {
        Some((second_op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;

            let span = Span::from_start_and_end(&left.span(), &right.span());
            let whitespace_span = right.whitespace_span();

            Node::new(
                Expr::comparison_op(second_op, left, right, rest_operands.collect()),
                span,
                whitespace_span,
            )
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a min/max expression
fn minmax_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let (rest, first_operand) = additive_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = bar
            .map(|token| token.into_node_with_value(BinaryOp::MinMax))
            .convert_errors()
            .parse(input)?;

        let (rest, operand) = additive_expr
            .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                &operator,
            ))
            .parse(rest)?;

        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = Span::from_start_and_end(&left.span(), &right.span());
            let whitespace_span = right.whitespace_span();

            Node::new(Expr::binary_op(op, left, right), span, whitespace_span)
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses an additive expression
fn additive_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let op = alt((
        plus.map(|token| token.into_node_with_value(BinaryOp::Add)),
        minus.map(|token| token.into_node_with_value(BinaryOp::Sub)),
        minus_minus.map(|token| token.into_node_with_value(BinaryOp::EscapedSub)),
    ))
    .convert_errors();

    left_associative_binary_op(multiplicative_expr, op).parse(input)
}

/// Parses a multiplicative expression
fn multiplicative_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let op = alt((
        star.map(|token| token.into_node_with_value(BinaryOp::Mul)),
        slash.map(|token| token.into_node_with_value(BinaryOp::Div)),
        slash_slash.map(|token| token.into_node_with_value(BinaryOp::EscapedDiv)),
        percent.map(|token| token.into_node_with_value(BinaryOp::Mod)),
    ))
    .convert_errors();

    left_associative_binary_op(exponential_expr, op).parse(input)
}

/// Parses an exponential expression (right associative)
fn exponential_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let mut op = caret
        .map(|token| token.into_node_with_value(BinaryOp::Pow))
        .convert_errors();

    let (rest, first_operand) = neg_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = exponential_expr
            .or_fail_with(ParserError::expr_binary_op_missing_second_operand(
                &operator,
            ))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;

            let span = Span::from_start_and_end(&left.span(), &right.span());
            let whitespace_span = right.whitespace_span();

            Node::new(Expr::binary_op(op, left, right), span, whitespace_span)
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a negation expression
fn neg_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    alt((
        |input| {
            let (rest, minus_op) = minus
                .map(|token| token.into_node_with_value(UnaryOp::Neg))
                .convert_errors()
                .parse(input)?;

            let (rest, expr) = neg_expr
                .or_fail_with(ParserError::unary_op_missing_operand(&minus_op))
                .parse(rest)?;

            let span = Span::from_start_and_end(&minus_op.span(), &expr.span());
            let whitespace_span = expr.whitespace_span();

            Ok((
                rest,
                Node::new(Expr::unary_op(minus_op, expr), span, whitespace_span),
            ))
        },
        primary_expr,
    ))
    .parse(input)
}

/// Parses a primary expression (literals, identifiers, function calls, parenthesized expressions)
fn primary_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    alt((
        map(number.convert_errors(), |n| {
            let parse_result = n.lexeme_str.parse::<f64>();
            let parse_result = parse_result.expect("all valid numbers should parse correctly");

            let literal_node = n.into_node_with_value(Literal::number(parse_result));
            literal_node.wrap(Expr::literal)
        }),
        map(string.convert_errors(), |s| {
            // trim quotes from the string
            let s_contents = s.lexeme_str[1..s.lexeme_str.len() - 1].to_string();
            let literal_node = s.into_node_with_value(Literal::string(s_contents));
            literal_node.wrap(Expr::literal)
        }),
        map(true_.convert_errors(), |t| {
            let literal_node = t.into_node_with_value(Literal::boolean(true));
            literal_node.wrap(Expr::literal)
        }),
        map(false_.convert_errors(), |t| {
            let literal_node = t.into_node_with_value(Literal::boolean(false));
            literal_node.wrap(Expr::literal)
        }),
        function_call,
        variable,
        parenthesized_expr,
    ))
    .parse(input)
}

/// Parses a function call
fn function_call(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let (rest, name) = identifier.convert_errors().parse(input)?;
    let name_node = IdentifierNode::from(name);

    let (rest, paren_left_token) = paren_left.convert_errors().parse(rest)?;
    let (rest, args) = separated_list0(comma.convert_errors(), expr).parse(rest)?;
    let (rest, paren_right_token) = paren_right
        .or_fail_with(ParserError::unclosed_paren(paren_left_token.lexeme_span))
        .parse(rest)?;

    let span = Span::from_start_and_end(&name_node.span(), &paren_right_token.lexeme_span);
    let whitespace_span = paren_right_token.whitespace_span;

    Ok((
        rest,
        Node::new(Expr::function_call(name_node, args), span, whitespace_span),
    ))
}

/// Parses a variable name
fn variable(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let (rest, parameter_id) = identifier.convert_errors().parse(input)?;
    let parameter_id_node = IdentifierNode::from(parameter_id);

    let (rest, reference_model_id_node) = opt(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, reference_model_id) = identifier
            .or_fail_with(ParserError::expr_variable_missing_reference_model(
                dot_token.lexeme_span,
            ))
            .parse(rest)?;

        let reference_model_id_node = IdentifierNode::from(reference_model_id);

        Ok((rest, reference_model_id_node))
    })
    .parse(rest)?;

    let variable_node = match reference_model_id_node {
        Some(reference_model_id_node) => {
            let variable_span = Span::from_start_and_end(
                &parameter_id_node.span(),
                &reference_model_id_node.span(),
            );
            let variable_whitespace_span = reference_model_id_node.whitespace_span();

            let variable = Variable::model_parameter(reference_model_id_node, parameter_id_node);

            Node::new(variable, variable_span, variable_whitespace_span)
        }
        None => parameter_id_node.wrap(Variable::identifier),
    };

    let expr = variable_node.wrap(Expr::variable);

    Ok((rest, expr))
}

/// Parses a parenthesized expression
fn parenthesized_expr(input: InputSpan<'_>) -> Result<'_, ExprNode, ParserError> {
    let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

    let (rest, expr) = expr
        .or_fail_with(ParserError::expr_paren_missing_expression(
            paren_left_token.lexeme_span,
        ))
        .parse(rest)?;

    let (rest, paren_right_token) = paren_right
        .or_fail_with(ParserError::unclosed_paren(paren_left_token.lexeme_span))
        .parse(rest)?;

    // note: we need to wrap the expr in a parenthesized node in order to keep the spans accurate
    //       otherwise, calculating spans using the parenthesized node as a start or end span
    //       will result in the calculated span ignoring the parens
    let span = Span::from_start_and_end(
        &paren_left_token.lexeme_span,
        &paren_right_token.lexeme_span,
    );
    let whitespace_span = paren_right_token.whitespace_span;

    let expr = Node::new(Expr::parenthesized(expr), span, whitespace_span);

    Ok((rest, expr))
}

#[cfg(test)]
#[expect(
    clippy::float_cmp,
    reason = "it will be obvious when floating point equality fails and we need to use a tolerance"
)]
#[expect(
    clippy::bool_assert_comparison,
    reason = "testing the contents of AST nodes"
)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn primary_expr_number() {
        let input = InputSpan::new_extra("42", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 42.0);
    }

    #[test]
    fn primary_expr_string() {
        let input = InputSpan::new_extra("'hello'", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::String(value) = value.take_value() else {
            panic!("expected string");
        };

        assert_eq!(value, "hello".to_string());
    }

    #[test]
    fn primary_expr_boolean_true() {
        let input = InputSpan::new_extra("true", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, true);
    }

    #[test]
    fn primary_expr_boolean_false() {
        let input = InputSpan::new_extra("false", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, false);
    }

    #[test]
    fn primary_expr_simple_identifier() {
        let input = InputSpan::new_extra("foo", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::Variable(value) = expr.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "foo");
    }

    #[test]
    fn primary_expr_multiword_identifier() {
        let input = InputSpan::new_extra("foo.bar", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::Variable(value) = expr.take_value() else {
            panic!("expected variable");
        };

        let Variable::ModelParameter {
            reference_model,
            parameter,
        } = value.take_value()
        else {
            panic!("expected model parameter");
        };

        assert_eq!(reference_model.as_str(), "bar");
        assert_eq!(parameter.as_str(), "foo");
    }

    #[test]
    fn function_call() {
        let input = InputSpan::new_extra("foo(1, 2)", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::FunctionCall { name, mut args } = expr.take_value() else {
            panic!("expected function call");
        };

        assert_eq!(name.as_str(), "foo");

        assert_eq!(args.len(), 2);

        let arg1 = args.remove(0);

        let Expr::Literal(value) = arg1.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 1.0);

        let arg2 = args.remove(0);

        let Expr::Literal(value) = arg2.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 2.0);
    }

    #[test]
    fn neg_expr() {
        let input = InputSpan::new_extra("-42", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::UnaryOp { op, expr } = expr.take_value() else {
            panic!("expected unary op");
        };

        assert_eq!(op.take_value(), UnaryOp::Neg);

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 42.0);
    }

    #[test]
    fn exponential_expr() {
        let input = InputSpan::new_extra("2^3", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::BinaryOp { op, left, right } = expr.take_value() else {
            panic!("expected binary op");
        };

        assert_eq!(op.take_value(), BinaryOp::Pow);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 2.0);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 3.0);
    }

    #[test]
    fn multiplicative_expr() {
        let input = InputSpan::new_extra("2*3", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::BinaryOp { op, left, right } = expr.take_value() else {
            panic!("expected binary op");
        };

        assert_eq!(op.take_value(), BinaryOp::Mul);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 2.0);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 3.0);
    }

    #[test]
    fn additive_expr() {
        let input = InputSpan::new_extra("2+3", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::BinaryOp { op, left, right } = expr.take_value() else {
            panic!("expected binary op");
        };

        assert_eq!(op.take_value(), BinaryOp::Add);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 2.0);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 3.0);
    }

    #[test]
    fn minmax_expr() {
        let input = InputSpan::new_extra("min_weight | max_weight", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::BinaryOp { op, left, right } = expr.take_value() else {
            panic!("expected binary op");
        };

        assert_eq!(op.take_value(), BinaryOp::MinMax);

        let Expr::Variable(value) = left.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "min_weight");

        let Expr::Variable(value) = right.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "max_weight");
    }

    #[test]
    fn comparison_expr() {
        let input = InputSpan::new_extra("2<3", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } = expr.take_value()
        else {
            panic!("expected comparison op");
        };

        assert_eq!(op.take_value(), ComparisonOp::LessThan);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 2.0);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 3.0);

        assert_eq!(rest_chained.len(), 0);
    }

    #[test]
    fn not_expr() {
        let input = InputSpan::new_extra("not true", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::UnaryOp { op, expr } = expr.take_value() else {
            panic!("expected unary op");
        };

        assert_eq!(op.take_value(), UnaryOp::Not);

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, true);
    }

    #[test]
    fn and_expr() {
        let input = InputSpan::new_extra("true and false", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::BinaryOp { op, left, right } = expr.take_value() else {
            panic!("expected binary op");
        };

        assert_eq!(op.take_value(), BinaryOp::And);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, true);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, false);
    }

    #[test]
    fn or_expr() {
        let input = InputSpan::new_extra("true or false", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::BinaryOp { op, left, right } = expr.take_value() else {
            panic!("expected binary op");
        };

        assert_eq!(op.take_value(), BinaryOp::Or);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, true);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Boolean(value) = value.take_value() else {
            panic!("expected boolean");
        };

        assert_eq!(value, false);
    }

    #[test]
    fn chained_comparison_expr() {
        let input = InputSpan::new_extra("1 < 2 < 3", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::ComparisonOp {
            op,
            left,
            right,
            mut rest_chained,
        } = expr.take_value()
        else {
            panic!("expected comparison op");
        };

        assert_eq!(op.take_value(), ComparisonOp::LessThan);

        let Expr::Literal(value) = left.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 1.0);

        let Expr::Literal(value) = right.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 2.0);

        assert_eq!(rest_chained.len(), 1);

        let (op, expr) = rest_chained.remove(0);

        assert_eq!(*op, ComparisonOp::LessThan);

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 3.0);
    }

    #[test]
    fn chained_comparison_expr_different_ops() {
        let input = InputSpan::new_extra("x <= y < z", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::ComparisonOp {
            op,
            left,
            right,
            mut rest_chained,
        } = expr.take_value()
        else {
            panic!("expected comparison op");
        };

        assert_eq!(op.take_value(), ComparisonOp::LessThanEq);

        let Expr::Variable(value) = left.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "x");

        let Expr::Variable(value) = right.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "y");

        assert_eq!(rest_chained.len(), 1);

        let (op, expr) = rest_chained.remove(0);

        assert_eq!(op.take_value(), ComparisonOp::LessThan);

        let Expr::Variable(value) = expr.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "z");
    }

    #[test]
    fn chained_comparison_expr_with_expressions() {
        let input = InputSpan::new_extra("x + 1 < y * 2 <= z - 3", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        // This is a complex expression, so we just verify it parses correctly
        // and has the right structure for a chained comparison
        assert!(matches!(expr.take_value(), Expr::ComparisonOp { .. }));
    }

    #[test]
    fn single_comparison_expr() {
        let input = InputSpan::new_extra("x != y", Config::default());
        let (_, expr) = parse(input).expect("parsing should succeed");

        let Expr::ComparisonOp {
            op,
            left,
            right,
            rest_chained,
        } = expr.take_value()
        else {
            panic!("expected comparison op");
        };

        assert_eq!(op.take_value(), ComparisonOp::NotEq);

        let Expr::Variable(value) = left.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "x");

        let Expr::Variable(value) = right.take_value() else {
            panic!("expected variable");
        };

        let Variable::Identifier(value) = value.take_value() else {
            panic!("expected identifier");
        };

        assert_eq!(value.as_str(), "y");

        assert_eq!(rest_chained.len(), 0);
    }

    #[test]
    fn parse_complete_success() {
        let input = InputSpan::new_extra("42", Config::default());
        let (_, expr) = parse_complete(input).expect("parsing should succeed");

        let Expr::Literal(value) = expr.take_value() else {
            panic!("expected literal");
        };

        let Literal::Number(value) = value.take_value() else {
            panic!("expected number");
        };

        assert_eq!(value, 42.0);
    }

    #[test]
    #[expect(
        clippy::assertions_on_result_states,
        reason = "we don't care about the error, just that it's an error"
    )]
    fn parse_complete_with_remaining_input() {
        let input = InputSpan::new_extra("42 rest", Config::default());
        let result = parse_complete(input);

        assert!(result.is_err());
    }

    mod general_error {
        use crate::error::reason::{ExpectKind, ParserErrorReason};

        use super::*;

        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Expected error for empty input");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Expr)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Expected error for whitespace only");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Expr)
            ));
        }

        #[test]
        fn symbols_only() {
            let input = InputSpan::new_extra("+++", Config::default());
            let result = parse(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Expected error for symbols only");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Expr)
            ));
        }

        #[test]
        fn parse_complete_with_remaining_input() {
            let input = InputSpan::new_extra("42 + 1 rest", Config::default());
            let result = parse_complete(input);

            let Err(nom::Err::Error(error)) = result else {
                panic!("Expected error for remaining input");
            };

            assert_eq!(error.error_offset, 7);
            assert_eq!(error.reason, ParserErrorReason::UnexpectedToken);
        }
    }

    mod unary_op_error {
        use crate::error::reason::{ExprKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn negation_missing_operand() {
            let input = InputSpan::new_extra("-", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 1);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, UnaryOp::Neg);
            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }

        #[test]
        fn not_missing_operand() {
            let input = InputSpan::new_extra("not", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::UnaryOpMissingOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, UnaryOp::Not);
            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 3);
        }
    }

    mod binary_op_error {
        use crate::error::reason::{ExprKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn addition_missing_second_operand() {
            let input = InputSpan::new_extra("1 +", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::Add);
            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn multiplication_missing_second_operand() {
            let input = InputSpan::new_extra("2 *", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::Mul);
            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn exponentiation_missing_second_operand() {
            let input = InputSpan::new_extra("2 ^", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::Pow);
            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn comparison_missing_second_operand() {
            let input = InputSpan::new_extra("x <", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 3);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::ComparisonOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, ComparisonOp::LessThan);
            assert_eq!(cause.start().offset, 2);
            assert_eq!(cause.end().offset, 3);
        }

        #[test]
        fn logical_and_missing_second_operand() {
            let input = InputSpan::new_extra("true and", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::And);
            assert_eq!(cause.start().offset, 5);
            assert_eq!(cause.end().offset, 8);
        }

        #[test]
        fn logical_or_missing_second_operand() {
            let input = InputSpan::new_extra("false or", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::Or);
            assert_eq!(cause.start().offset, 6);
            assert_eq!(cause.end().offset, 8);
        }

        #[test]
        fn minmax_missing_second_operand() {
            let input = InputSpan::new_extra("min_weight |", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 12);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::MinMax);
            assert_eq!(cause.start().offset, 11);
            assert_eq!(cause.end().offset, 12);
        }
    }

    mod parenthesized_expr_error {
        use crate::error::reason::{ExprKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn missing_expression_in_parentheses() {
            let input = InputSpan::new_extra("()", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 1);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::ParenMissingExpr),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }

        #[test]
        fn unclosed_parentheses() {
            let input = InputSpan::new_extra("(1 + 2", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 6);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedParen,
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }

        #[test]
        fn nested_unclosed_parentheses() {
            let input = InputSpan::new_extra("((1 + 2)", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedParen,
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 0);
            assert_eq!(cause.end().offset, 1);
        }
    }

    mod function_call_error {
        use crate::error::reason::{IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's ok"
        )]
        fn missing_opening_paren() {
            let input = InputSpan::new_extra("foo", Config::default());
            let result = parse(input);
            // This should succeed as it's a valid variable
            assert!(result.is_ok());
        }

        #[test]
        fn missing_closing_paren() {
            let input = InputSpan::new_extra("foo(1, 2", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 8);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedParen,
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 4);
        }

        #[test]
        #[expect(
            clippy::assertions_on_result_states,
            reason = "we don't care about the result, just that it's ok"
        )]
        fn empty_function_call() {
            let input = InputSpan::new_extra("foo()", Config::default());
            let result = parse(input);
            // This should succeed as it's a valid function call with no arguments
            assert!(result.is_ok());
        }

        #[test]
        fn missing_argument_after_comma() {
            let input = InputSpan::new_extra("foo(1,)", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 5);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::UnclosedParen,
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 4);
        }
    }

    mod variable_error {
        use crate::error::reason::{ExprKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn missing_identifier_after_dot() {
            let input = InputSpan::new_extra("foo.", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 4);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::VariableMissingReferenceModel),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(cause.start().offset, 3);
            assert_eq!(cause.end().offset, 4);
        }
    }

    mod literal_error {
        use crate::error::reason::{ExpectKind, ParserErrorReason};

        use super::*;

        #[test]
        fn unterminated_string() {
            let input = InputSpan::new_extra("'hello", Config::default());
            let result = parse(input);

            // This should be a token error for unterminated string
            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 6);
            assert!(matches!(error.reason, ParserErrorReason::TokenError(_)));
        }

        #[test]
        fn invalid_number() {
            let input = InputSpan::new_extra("@", Config::default());
            let result = parse(input);

            // This should be an Expect(Expr) error since @ is not a valid expression start
            let Err(nom::Err::Error(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 0);
            assert!(matches!(
                error.reason,
                ParserErrorReason::Expect(ExpectKind::Expr)
            ));
        }
    }

    mod precedence_error {
        use crate::error::reason::{ExprKind, IncompleteKind, ParserErrorReason};

        use super::*;

        #[test]
        fn chained_binary_ops_missing_operand() {
            let input = InputSpan::new_extra("1 + 2 *", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 7);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::Mul);
            assert_eq!(cause.start().offset, 6);
            assert_eq!(cause.end().offset, 7);
        }

        #[test]
        fn complex_expression_missing_operand() {
            let input = InputSpan::new_extra("(1 + 2) * 3 +", Config::default());
            let result = parse(input);

            let Err(nom::Err::Failure(error)) = result else {
                panic!("Unexpected result {result:?}");
            };

            assert_eq!(error.error_offset, 13);

            let ParserErrorReason::Incomplete {
                kind: IncompleteKind::Expr(ExprKind::BinaryOpMissingSecondOperand { operator }),
                cause,
            } = error.reason
            else {
                panic!("Unexpected reason {:?}", error.reason);
            };

            assert_eq!(operator, BinaryOp::Add);
            assert_eq!(cause.start().offset, 12);
            assert_eq!(cause.end().offset, 13);
        }
    }
}
