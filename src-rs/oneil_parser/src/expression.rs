//! Expression parsing for the Oneil language.
//!
//! These expressions can be used in a variety of contexts, including:
//! - Parameter values
//! - Parameter limits
//! - Piecewise conditions
//! - Tests
//! - Model inputs

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, map, map_res, opt},
    multi::{many0, separated_list0},
};

use oneil_ast::{
    Span as AstSpan,
    expression::{BinaryOp, BinaryOpNode, Expr, ExprNode, Literal, UnaryOp, Variable},
    naming::Identifier,
    node::Node,
};

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
    util::{Parser, Result, Span},
};

fn left_associative_binary_op<'a>(
    mut operand: impl Parser<'a, ExprNode, ParserError> + Copy,
    mut operator: impl Parser<'a, BinaryOpNode, ParserError>,
) -> impl Parser<'a, ExprNode, ParserError> {
    move |input| {
        let (rest, first_operand) = operand.parse(input)?;
        let (rest, rest_operands) = many0(|input| {
            let (rest, operator) = operator.parse(input)?;
            let (rest, operand) = operand
                .or_fail_with(ParserError::binary_op_missing_second_operand(&operator))
                .parse(rest)?;
            Ok((rest, (operator, operand)))
        })
        .parse(rest)?;

        let expr = rest_operands
            .into_iter()
            .fold(first_operand, |acc, (op, expr)| {
                let left = acc;
                let right = expr;
                let span = AstSpan::calc_span(&left, &right);
                Node::new(span, Expr::binary_op(op, left, right))
            });

        Ok((rest, expr))
    }
}

/// Parses an expression
///
/// This function **may not consume the complete input**.
pub fn parse(input: Span) -> Result<ExprNode, ParserError> {
    expr(input)
}

/// Parses an expression
///
/// This function **fails if the complete input is not consumed**.
pub fn parse_complete(input: Span) -> Result<ExprNode, ParserError> {
    all_consuming(expr).parse(input)
}

/// Parses an expression
fn expr(input: Span) -> Result<ExprNode, ParserError> {
    or_expr
        .convert_error_to(ParserError::expect_expr)
        .parse(input)
}

/// Parses an OR expression (lowest precedence)
fn or_expr(input: Span) -> Result<ExprNode, ParserError> {
    let or = or
        .map(|token| Node::new(token, BinaryOp::Or))
        .convert_errors();
    left_associative_binary_op(and_expr, or).parse(input)
}

/// Parses an AND expression
fn and_expr(input: Span) -> Result<ExprNode, ParserError> {
    let and = and
        .map(|token| Node::new(token, BinaryOp::And))
        .convert_errors();
    left_associative_binary_op(not_expr, and).parse(input)
}

/// Parses a NOT expression
fn not_expr(input: Span) -> Result<ExprNode, ParserError> {
    alt((
        |input| {
            let (rest, not_op) = not
                .map(|token| Node::new(token, UnaryOp::Not))
                .convert_errors()
                .parse(input)?;

            let (rest, expr) = not_expr
                .or_fail_with(ParserError::unary_op_missing_operand(&not_op))
                .parse(rest)?;

            let span = AstSpan::calc_span(&not_op, &expr);

            Ok((rest, Node::new(span, Expr::unary_op(not_op, expr))))
        },
        comparison_expr,
    ))
    .parse(input)
}

/// Parses a comparison expression
fn comparison_expr(input: Span) -> Result<ExprNode, ParserError> {
    let mut op = alt((
        less_than_equals.map(|token| Node::new(token, BinaryOp::LessThanEq)),
        greater_than_equals.map(|token| Node::new(token, BinaryOp::GreaterThanEq)),
        less_than.map(|token| Node::new(token, BinaryOp::LessThan)),
        greater_than.map(|token| Node::new(token, BinaryOp::GreaterThan)),
        equals_equals.map(|token| Node::new(token, BinaryOp::Eq)),
        bang_equals.map(|token| Node::new(token, BinaryOp::NotEq)),
    ))
    .convert_errors();

    let (rest, first_operand) = minmax_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = minmax_expr
            .or_fail_with(ParserError::binary_op_missing_second_operand(&operator))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = AstSpan::calc_span(&left, &right);
            Node::new(span, Expr::binary_op(op, left, right))
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a min/max expression
///
/// Ex: `min_weight | max_weight`
fn minmax_expr(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, first_operand) = additive_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = bar
            .map(|token| Node::new(token, BinaryOp::MinMax))
            .convert_errors()
            .parse(input)?;

        let (rest, operand) = additive_expr
            .or_fail_with(ParserError::binary_op_missing_second_operand(&operator))
            .parse(rest)?;

        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = AstSpan::calc_span(&left, &right);
            Node::new(span, Expr::binary_op(op, left, right))
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses an additive expression
fn additive_expr(input: Span) -> Result<ExprNode, ParserError> {
    let op = alt((
        plus.map(|token| Node::new(token, BinaryOp::Add)),
        minus.map(|token| Node::new(token, BinaryOp::Sub)),
        minus_minus.map(|token| Node::new(token, BinaryOp::TrueSub)),
    ))
    .convert_errors();

    left_associative_binary_op(multiplicative_expr, op).parse(input)
}

/// Parses a multiplicative expression
fn multiplicative_expr(input: Span) -> Result<ExprNode, ParserError> {
    let op = alt((
        star.map(|token| Node::new(token, BinaryOp::Mul)),
        slash.map(|token| Node::new(token, BinaryOp::Div)),
        slash_slash.map(|token| Node::new(token, BinaryOp::TrueDiv)),
        percent.map(|token| Node::new(token, BinaryOp::Mod)),
    ))
    .convert_errors();

    left_associative_binary_op(exponential_expr, op).parse(input)
}

/// Parses an exponential expression (right associative)
fn exponential_expr(input: Span) -> Result<ExprNode, ParserError> {
    let mut op = caret
        .map(|token| Node::new(token, BinaryOp::Pow))
        .convert_errors();

    let (rest, first_operand) = neg_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = exponential_expr
            .or_fail_with(ParserError::binary_op_missing_second_operand(&operator))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => {
            let left = first_operand;
            let right = second_operand;
            let span = AstSpan::calc_span(&left, &right);
            Node::new(span, Expr::binary_op(op, left, right))
        }
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a negation expression
fn neg_expr(input: Span) -> Result<ExprNode, ParserError> {
    alt((
        |input| {
            let (rest, minus_op) = minus
                .map(|token| Node::new(token, UnaryOp::Neg))
                .convert_errors()
                .parse(input)?;

            let (rest, expr) = neg_expr
                .or_fail_with(ParserError::unary_op_missing_operand(&minus_op))
                .parse(rest)?;

            let span = AstSpan::calc_span(&minus_op, &expr);

            Ok((rest, Node::new(span, Expr::unary_op(minus_op, expr))))
        },
        primary_expr,
    ))
    .parse(input)
}

/// Parses a primary expression (literals, identifiers, function calls, parenthesized expressions)
fn primary_expr(input: Span) -> Result<ExprNode, ParserError> {
    alt((
        map_res(number.convert_errors(), |n| {
            let parse_result = n.lexeme().parse::<f64>();
            match parse_result {
                Ok(n_value) => {
                    let node = Node::new(n, Literal::number(n_value));
                    let node = Node::new(n, Expr::literal(node));
                    Ok(node)
                }
                Err(_) => Err(ParserError::invalid_number(&n)),
            }
        }),
        map(string.convert_errors(), |s| {
            // trim quotes from the string
            let s_contents = s.lexeme()[1..s.lexeme().len() - 1].to_string();
            let node = Node::new(s, Literal::string(s_contents));
            let node = Node::new(s, Expr::literal(node));
            node
        }),
        map(true_.convert_errors(), |t| {
            let node = Node::new(t, Literal::boolean(true));
            let node = Node::new(t, Expr::literal(node));
            node
        }),
        map(false_.convert_errors(), |t| {
            let node = Node::new(t, Literal::boolean(false));
            let node = Node::new(t, Expr::literal(node));
            node
        }),
        function_call,
        variable,
        parenthesized_expr,
    ))
    .parse(input)
}

/// Parses a function call
fn function_call(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, name) = identifier.convert_errors().parse(input)?;
    let name_span = AstSpan::from(&name);
    let name = Node::new(name_span, Identifier::new(name.lexeme().to_string()));

    let (rest, paren_left_span) = paren_left.convert_errors().parse(rest)?;
    let (rest, args) = separated_list0(comma.convert_errors(), expr).parse(rest)?;
    let (rest, paren_right_span) = paren_right
        .or_fail_with(ParserError::unclosed_paren(&paren_left_span))
        .parse(rest)?;

    let span = AstSpan::calc_span(&name, &paren_right_span);

    Ok((rest, Node::new(span, Expr::function_call(name, args))))
}

/// Parses a variable name
fn variable(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, first_id) = identifier.convert_errors().parse(input)?;
    let first_id_span = AstSpan::from(&first_id);
    let first_id = Node::new(
        first_id_span,
        Identifier::new(first_id.lexeme().to_string()),
    );

    let (rest, rest_ids) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, id) = identifier
            .or_fail_with(ParserError::variable_missing_parent(&dot_token))
            .parse(rest)?;
        let id_span = AstSpan::from(&id);
        let id = Node::new(id_span, Identifier::new(id.lexeme().to_string()));

        Ok((rest, id))
    })
    .parse(rest)?;

    let first_id = Node::new(first_id_span, Variable::identifier(first_id));

    let variable = rest_ids.into_iter().fold(first_id, |acc, id| {
        let start = &acc;
        let end = &id;
        let span = AstSpan::calc_span(start, end);

        Node::new(span, Variable::accessor(id, acc))
    });

    let span = AstSpan::from(&variable);
    let expr = Node::new(span, Expr::variable(variable));

    Ok((rest, expr))
}

/// Parses a parenthesized expression
fn parenthesized_expr(input: Span) -> Result<ExprNode, ParserError> {
    let (rest, paren_left_span) = paren_left.convert_errors().parse(input)?;

    let (rest, expr) = expr
        .or_fail_with(ParserError::paren_missing_expression(&paren_left_span))
        .parse(rest)?;

    let (rest, paren_right_span) = paren_right
        .or_fail_with(ParserError::unclosed_paren(&paren_left_span))
        .parse(rest)?;

    // note: we need to wrap the expr in a parenthesized node in order to keep the spans accurate
    //       otherwise, calculating spans using the parenthesized node as a start or end span
    //       will result in the calculated span ignoring the parens
    let span = AstSpan::calc_span(&paren_left_span, &paren_right_span);
    let expr = Node::new(span, Expr::parenthesized(expr));

    Ok((rest, expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn test_primary_expr_number() {
        let input = Span::new_extra("42", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 2, 2),
            Expr::literal(Node::new(AstSpan::new(0, 2, 2), Literal::number(42.0))),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_string() {
        let input = Span::new_extra("\"hello\"", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 7, 7),
            Expr::literal(Node::new(
                AstSpan::new(0, 7, 7),
                Literal::string("hello".to_string()),
            )),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_boolean_true() {
        let input = Span::new_extra("true", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 4, 4),
            Expr::literal(Node::new(AstSpan::new(0, 4, 4), Literal::boolean(true))),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_boolean_false() {
        let input = Span::new_extra("false", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_expr = Node::new(
            AstSpan::new(0, 5, 5),
            Expr::literal(Node::new(AstSpan::new(0, 5, 5), Literal::boolean(false))),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_simple_identifier() {
        let input = Span::new_extra("foo", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 3, 3), Identifier::new("foo".to_string()));

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::variable(Node::new(
                AstSpan::new(0, 3, 3),
                Variable::identifier(expected_id),
            )),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_primary_expr_multiword_identifier() {
        let input = Span::new_extra("foo.bar.baz", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_id = Node::new(AstSpan::new(0, 3, 3), Identifier::new("foo".to_string()));
        let expected_id2 = Node::new(AstSpan::new(4, 7, 7), Identifier::new("bar".to_string()));
        let expected_id3 = Node::new(AstSpan::new(8, 11, 11), Identifier::new("baz".to_string()));

        let variable = Node::new(AstSpan::new(0, 3, 3), Variable::identifier(expected_id));
        let variable = Node::new(
            AstSpan::new(0, 7, 7),
            Variable::accessor(expected_id2, variable),
        );
        let variable = Node::new(
            AstSpan::new(0, 11, 11),
            Variable::accessor(expected_id3, variable),
        );

        let expected_expr = Node::new(AstSpan::new(0, 11, 11), Expr::variable(variable));
        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_function_call() {
        let input = Span::new_extra("foo(1, 2)", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_foo = Node::new(AstSpan::new(0, 3, 3), Identifier::new("foo".to_string()));

        let expected_1 = Node::new(AstSpan::new(4, 5, 5), Literal::number(1.0));
        let expected_1 = Node::new(AstSpan::new(4, 5, 5), Expr::literal(expected_1));

        let expected_2 = Node::new(AstSpan::new(7, 8, 8), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(7, 8, 8), Expr::literal(expected_2));

        let expected_expr = Node::new(
            AstSpan::new(0, 9, 9),
            Expr::function_call(expected_foo, vec![expected_1, expected_2]),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_neg_expr() {
        let input = Span::new_extra("-42", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_42 = Node::new(AstSpan::new(1, 3, 3), Literal::number(42.0));
        let expected_42 = Node::new(AstSpan::new(1, 3, 3), Expr::literal(expected_42));

        let op = Node::new(AstSpan::new(0, 1, 1), UnaryOp::Neg);

        let expected_expr = Node::new(AstSpan::new(0, 3, 3), Expr::unary_op(op, expected_42));

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_exponential_expr() {
        let input = Span::new_extra("2^3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::Pow);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_multiplicative_expr() {
        let input = Span::new_extra("2*3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::Mul);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_additive_expr() {
        let input = Span::new_extra("2+3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::Add);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_minmax_expr() {
        let input = Span::new_extra("min_weight | max_weight", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_min = Node::new(
            AstSpan::new(0, 10, 11),
            Identifier::new("min_weight".to_string()),
        );
        let expected_min = Node::new(AstSpan::new(0, 10, 11), Variable::identifier(expected_min));
        let expected_min = Node::new(AstSpan::new(0, 10, 11), Expr::variable(expected_min));

        let expected_max = Node::new(
            AstSpan::new(13, 23, 23),
            Identifier::new("max_weight".to_string()),
        );
        let expected_max = Node::new(AstSpan::new(13, 23, 23), Variable::identifier(expected_max));
        let expected_max = Node::new(AstSpan::new(13, 23, 23), Expr::variable(expected_max));

        let op = Node::new(AstSpan::new(11, 12, 13), BinaryOp::MinMax);

        let expected_expr = Node::new(
            AstSpan::new(0, 23, 23),
            Expr::binary_op(op, expected_min, expected_max),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_comparison_expr() {
        let input = Span::new_extra("2<3", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Literal::number(2.0));
        let expected_2 = Node::new(AstSpan::new(0, 1, 1), Expr::literal(expected_2));

        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Literal::number(3.0));
        let expected_3 = Node::new(AstSpan::new(2, 3, 3), Expr::literal(expected_3));

        let op = Node::new(AstSpan::new(1, 2, 2), BinaryOp::LessThan);

        let expected_expr = Node::new(
            AstSpan::new(0, 3, 3),
            Expr::binary_op(op, expected_2, expected_3),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_not_expr() {
        let input = Span::new_extra("not true", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_true = Node::new(AstSpan::new(4, 8, 8), Literal::boolean(true));
        let expected_true = Node::new(AstSpan::new(4, 8, 8), Expr::literal(expected_true));

        let op = Node::new(AstSpan::new(0, 3, 4), UnaryOp::Not);

        let expected_expr = Node::new(AstSpan::new(0, 8, 8), Expr::unary_op(op, expected_true));

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_and_expr() {
        let input = Span::new_extra("true and false", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_true = Node::new(AstSpan::new(0, 4, 5), Literal::boolean(true));
        let expected_true = Node::new(AstSpan::new(0, 4, 5), Expr::literal(expected_true));

        let expected_false = Node::new(AstSpan::new(9, 14, 14), Literal::boolean(false));
        let expected_false = Node::new(AstSpan::new(9, 14, 14), Expr::literal(expected_false));

        let op = Node::new(AstSpan::new(5, 8, 9), BinaryOp::And);

        let expected_expr = Node::new(
            AstSpan::new(0, 14, 14),
            Expr::binary_op(op, expected_true, expected_false),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_or_expr() {
        let input = Span::new_extra("true or false", Config::default());
        let (_, expr) = parse(input).unwrap();

        let expected_true = Node::new(AstSpan::new(0, 4, 5), Literal::boolean(true));
        let expected_true = Node::new(AstSpan::new(0, 4, 5), Expr::literal(expected_true));

        let expected_false = Node::new(AstSpan::new(8, 13, 13), Literal::boolean(false));
        let expected_false = Node::new(AstSpan::new(8, 13, 13), Expr::literal(expected_false));

        let op = Node::new(AstSpan::new(5, 7, 8), BinaryOp::Or);

        let expected_expr = Node::new(
            AstSpan::new(0, 13, 13),
            Expr::binary_op(op, expected_true, expected_false),
        );

        assert_eq!(expr, expected_expr);
    }

    #[test]
    fn test_complex_expr() {
        let input = Span::new_extra("-(2 + 3*4^2) < foo(5, 6) and not bar", Config::default());
        let (_, expr) = parse(input).unwrap();
        // The exact structure is complex but we just verify it parses
        assert!(matches!(expr.node_value(), Expr::BinaryOp { .. }));
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("42", Config::default());
        let (rest, expr) = parse_complete(input).unwrap();

        let expected_42 = Node::new(AstSpan::new(0, 2, 2), Literal::number(42.0));
        let expected_42 = Node::new(AstSpan::new(0, 2, 2), Expr::literal(expected_42));

        assert_eq!(expr, expected_42);
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("42 rest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
