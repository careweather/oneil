//! Expression parsing for the Oneil language.
//!
//! These expressions can be used in a variety of contexts, including:
//! - Parameter values
//! - Parameter limits
//! - Piecewise conditions
//! - Tests
//! - Model inputs
//!
//! # Examples
//!
//! ```
//! use oneil_parser::expression::parse;
//! use oneil_parser::{Config, Span};
//!
//! // Parse a simple arithmetic expression
//! let input = Span::new_extra("2 + 3 * 4", Config::default());
//! let (_, expr) = parse(input).unwrap();
//!
//! // Parse a complex expression with multiple operators
//! let input = Span::new_extra("-(x + y)^2 < foo(a, b) and not c", Config::default());
//! let (_, expr) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, cut, map, map_res, opt, value},
    multi::{many0, separated_list0},
};

use oneil_ast::expression::{BinaryOp, Expr, Literal, UnaryOp, Variable};

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
    mut operand: impl Parser<'a, Expr, ParserError> + Copy,
    mut operator: impl Parser<'a, BinaryOp, ParserError>,
) -> impl Parser<'a, Expr, ParserError> {
    move |input| {
        let (rest, first_operand) = operand.parse(input)?;
        let (rest, rest_operands) = many0(|input| {
            let (rest, operator) = operator.parse(input)?;
            let (rest, operand) = cut(operand)
                .map_error(ParserError::binary_op_missing_second_operand(operator))
                .parse(rest)?;
            Ok((rest, (operator, operand)))
        })
        .parse(rest)?;

        let expr = rest_operands
            .into_iter()
            .fold(first_operand, |acc, (op, expr)| Expr::BinaryOp {
                op,
                left: Box::new(acc),
                right: Box::new(expr),
            });

        Ok((rest, expr))
    }
}

/// Parses an expression
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil_parser::expression::parse;
/// use oneil_parser::{Config, Span};
///
/// let input = Span::new_extra("2 + 3 * 4", Config::default());
/// let (rest, expr) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil_parser::expression::parse;
/// use oneil_parser::{Config, Span};
///
/// let input = Span::new_extra("2 + 3 * 4 rest", Config::default());
/// let (rest, expr) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"rest");
/// ```
pub fn parse(input: Span) -> Result<Expr, ParserError> {
    expr(input)
}

/// Parses an expression
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil_parser::expression::parse_complete;
/// use oneil_parser::{Config, Span};
///
/// let input = Span::new_extra("2 + 3 * 4", Config::default());
/// let (rest, expr) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil_parser::expression::parse_complete;
/// use oneil_parser::{Config, Span};
///
/// let input = Span::new_extra("2 + 3 * 4 rest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse_complete(input: Span) -> Result<Expr, ParserError> {
    all_consuming(expr).parse(input)
}

/// Parses an expression
fn expr(input: Span) -> Result<Expr, ParserError> {
    or_expr.map_error(ParserError::expect_expr).parse(input)
}

/// Parses an OR expression (lowest precedence)
fn or_expr(input: Span) -> Result<Expr, ParserError> {
    let or = value(BinaryOp::Or, or).convert_errors();
    left_associative_binary_op(and_expr, or).parse(input)
}

/// Parses an AND expression
fn and_expr(input: Span) -> Result<Expr, ParserError> {
    let and = value(BinaryOp::And, and).convert_errors();
    left_associative_binary_op(not_expr, and).parse(input)
}

/// Parses a NOT expression
fn not_expr(input: Span) -> Result<Expr, ParserError> {
    alt((
        |input| {
            let not = value(UnaryOp::Not, not);

            let (rest, op) = not.convert_errors().parse(input)?;

            let (rest, expr) = cut(not_expr)
                .map_error(ParserError::unary_op_missing_operand(op))
                .parse(rest)?;

            Ok((
                rest,
                Expr::UnaryOp {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                },
            ))
        },
        comparison_expr,
    ))
    .parse(input)
}

/// Parses a comparison expression
fn comparison_expr(input: Span) -> Result<Expr, ParserError> {
    let mut op = alt((
        value(BinaryOp::LessThanEq, less_than_equals),
        value(BinaryOp::GreaterThanEq, greater_than_equals),
        value(BinaryOp::LessThan, less_than),
        value(BinaryOp::GreaterThan, greater_than),
        value(BinaryOp::Eq, equals_equals),
        value(BinaryOp::NotEq, bang_equals),
    ))
    .convert_errors();

    let (rest, first_operand) = minmax_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = cut(minmax_expr)
            .map_error(ParserError::binary_op_missing_second_operand(operator))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => Expr::BinaryOp {
            op,
            left: Box::new(first_operand),
            right: Box::new(second_operand),
        },
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a min/max expression
///
/// Ex: `min_weight | max_weight`
fn minmax_expr(input: Span) -> Result<Expr, ParserError> {
    let (rest, first_operand) = additive_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = value(BinaryOp::MinMax, bar).convert_errors().parse(input)?;
        let (rest, operand) = cut(additive_expr)
            .map_error(ParserError::binary_op_missing_second_operand(operator))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => Expr::BinaryOp {
            op,
            left: Box::new(first_operand),
            right: Box::new(second_operand),
        },
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses an additive expression
fn additive_expr(input: Span) -> Result<Expr, ParserError> {
    let op = alt((
        value(BinaryOp::Add, plus),
        value(BinaryOp::Sub, minus),
        value(BinaryOp::TrueSub, minus_minus),
    ))
    .convert_errors();

    left_associative_binary_op(multiplicative_expr, op).parse(input)
}

/// Parses a multiplicative expression
fn multiplicative_expr(input: Span) -> Result<Expr, ParserError> {
    let op = alt((
        value(BinaryOp::Mul, star),
        value(BinaryOp::Div, slash),
        value(BinaryOp::TrueDiv, slash_slash),
        value(BinaryOp::Mod, percent),
    ))
    .convert_errors();

    left_associative_binary_op(exponential_expr, op).parse(input)
}

/// Parses an exponential expression (right associative)
fn exponential_expr(input: Span) -> Result<Expr, ParserError> {
    let mut op = value(BinaryOp::Pow, caret).convert_errors();

    let (rest, first_operand) = neg_expr.parse(input)?;
    let (rest, second_operand) = opt(|input| {
        let (rest, operator) = op.parse(input)?;
        let (rest, operand) = cut(exponential_expr)
            .map_error(ParserError::binary_op_missing_second_operand(operator))
            .parse(rest)?;
        Ok((rest, (operator, operand)))
    })
    .parse(rest)?;

    let expr = match second_operand {
        Some((op, second_operand)) => Expr::BinaryOp {
            op,
            left: Box::new(first_operand),
            right: Box::new(second_operand),
        },
        None => first_operand,
    };

    Ok((rest, expr))
}

/// Parses a negation expression
fn neg_expr(input: Span) -> Result<Expr, ParserError> {
    alt((
        |input| {
            let minus = value(UnaryOp::Neg, minus);
            let (rest, op) = minus.convert_errors().parse(input)?;
            let (rest, expr) = cut(neg_expr)
                .map_error(ParserError::unary_op_missing_operand(op))
                .parse(rest)?;
            Ok((
                rest,
                Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                },
            ))
        },
        primary_expr,
    ))
    .parse(input)
}

/// Parses a primary expression (literals, identifiers, function calls, parenthesized expressions)
fn primary_expr(input: Span) -> Result<Expr, ParserError> {
    alt((
        map_res(number.convert_errors(), |n| {
            let parse_result = n.lexeme().parse::<f64>();
            match parse_result {
                Ok(n) => Ok(Expr::Literal(Literal::Number(n))),
                Err(_) => Err(ParserError::invalid_number(n)),
            }
        }),
        map(string.convert_errors(), |s| {
            // trim quotes from the string
            let s_contents = s.lexeme()[1..s.lexeme().len() - 1].to_string();
            Expr::Literal(Literal::String(s_contents))
        }),
        map(true_.convert_errors(), |_| {
            Expr::Literal(Literal::Boolean(true))
        }),
        map(false_.convert_errors(), |_| {
            Expr::Literal(Literal::Boolean(false))
        }),
        function_call,
        variable,
        parenthesized_expr,
    ))
    .parse(input)
}

/// Parses a function call
fn function_call(input: Span) -> Result<Expr, ParserError> {
    let (rest, name) = identifier.convert_errors().parse(input)?;
    let (rest, paren_left_span) = paren_left.convert_errors().parse(rest)?;
    let (rest, args) = separated_list0(comma.convert_errors(), expr).parse(rest)?;
    let (rest, _) = cut(paren_right)
        .map_failure(ParserError::unclosed_paren(paren_left_span))
        .parse(rest)?;

    Ok((
        rest,
        Expr::FunctionCall {
            name: name.lexeme().to_string(),
            args,
        },
    ))
}

/// Parses a variable name
fn variable(input: Span) -> Result<Expr, ParserError> {
    let (rest, first_id) = identifier.convert_errors().parse(input)?;
    let (rest, rest_ids) = many0(|input| {
        let (rest, dot_token) = dot.convert_errors().parse(input)?;
        let (rest, id) = cut(identifier.map_error(ParserError::variable_missing_parent(dot_token)))
            .parse(rest)?;
        Ok((rest, id))
    })
    .parse(rest)?;

    let expr = rest_ids.into_iter().fold(
        Variable::Identifier(first_id.lexeme().to_string()),
        |acc, id| Variable::Accessor {
            parent: id.lexeme().to_string(),
            component: Box::new(acc),
        },
    );

    Ok((rest, Expr::Variable(expr)))
}

/// Parses a parenthesized expression
fn parenthesized_expr(input: Span) -> Result<Expr, ParserError> {
    let (rest, paren_left_span) = paren_left.convert_errors().parse(input)?;

    let (rest, expr) = cut(expr)
        .map_failure(ParserError::paren_missing_expression(paren_left_span))
        .parse(rest)?;

    let (rest, _) = cut(paren_right)
        .map_failure(ParserError::unclosed_paren(paren_left_span))
        .parse(rest)?;

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
        assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
    }

    #[test]
    fn test_primary_expr_string() {
        let input = Span::new_extra("\"hello\"", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::String("hello".to_string())));
    }

    #[test]
    fn test_primary_expr_boolean_true() {
        let input = Span::new_extra("true", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_primary_expr_boolean_false() {
        let input = Span::new_extra("false", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Boolean(false)));
    }

    #[test]
    fn test_primary_expr_simple_identifier() {
        let input = Span::new_extra("foo", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::Variable(Variable::Identifier("foo".to_string()))
        );
    }

    #[test]
    fn test_primary_expr_multiword_identifier() {
        let input = Span::new_extra("foo.bar.baz", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::Variable(Variable::Accessor {
                parent: "baz".to_string(),
                component: Box::new(Variable::Accessor {
                    parent: "bar".to_string(),
                    component: Box::new(Variable::Identifier("foo".to_string())),
                }),
            })
        );
    }

    #[test]
    fn test_function_call() {
        let input = Span::new_extra("foo(1, 2)", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::FunctionCall { name, args } => {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_neg_expr() {
        let input = Span::new_extra("-42", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::UnaryOp {
                op: UnaryOp::Neg,
                expr: Box::new(Expr::Literal(Literal::Number(42.0))),
            }
        );
    }

    #[test]
    fn test_exponential_expr() {
        let input = Span::new_extra("2^3", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Pow, ..
            } => (),
            _ => panic!("Expected power operation"),
        }
    }

    #[test]
    fn test_multiplicative_expr() {
        let input = Span::new_extra("2*3", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Mul, ..
            } => (),
            _ => panic!("Expected multiplication"),
        }
    }

    #[test]
    fn test_additive_expr() {
        let input = Span::new_extra("2+3", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Add, ..
            } => (),
            _ => panic!("Expected addition"),
        }
    }

    #[test]
    fn test_minmax_expr() {
        let input = Span::new_extra("min_weight | max_weight", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::MinMax,
                ..
            } => (),
            _ => panic!("Expected min/max operation"),
        }
    }

    #[test]
    fn test_comparison_expr() {
        let input = Span::new_extra("2<3", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::LessThan,
                ..
            } => (),
            _ => panic!("Expected less than comparison"),
        }
    }

    #[test]
    fn test_not_expr() {
        let input = Span::new_extra("not true", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::UnaryOp {
                op: UnaryOp::Not, ..
            } => (),
            _ => panic!("Expected not operation"),
        }
    }

    #[test]
    fn test_and_expr() {
        let input = Span::new_extra("true and false", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::And, ..
            } => (),
            _ => panic!("Expected and operation"),
        }
    }

    #[test]
    fn test_or_expr() {
        let input = Span::new_extra("true or false", Config::default());
        let (_, expr) = parse(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Or, ..
            } => (),
            _ => panic!("Expected or operation"),
        }
    }

    #[test]
    fn test_complex_expr() {
        let input = Span::new_extra("-(2 + 3*4^2) < foo(5, 6) and not bar", Config::default());
        let (_, expr) = parse(input).unwrap();
        // The exact structure is complex but we just verify it parses
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::And,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_complete_success() {
        let input = Span::new_extra("42", Config::default());
        let (rest, expr) = parse_complete(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("42 rest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
