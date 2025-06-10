//! Expression parsing for the Oneil language.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::expression::parse;
//! use oneil::parser::{Config, Span};
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
    multi::{many0, separated_list0, separated_list1},
};

use crate::ast::{
    Expr,
    expression::{BinaryOp, Literal, UnaryOp},
};

use super::{
    error::{ErrorHandlingParser as _, ParserError, ParserErrorKind},
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
    operand: impl Parser<'a, Expr, ParserError<'a>> + Copy,
    operator: impl Parser<'a, BinaryOp, ParserError<'a>>,
) -> impl Parser<'a, Expr, ParserError<'a>> {
    (operand, many0((operator, cut(operand)))).map(|(first, rest)| {
        rest.into_iter()
            .fold(first, |acc, (op, expr)| Expr::BinaryOp {
                op,
                left: Box::new(acc),
                right: Box::new(expr),
            })
    })
}

/// Parses an expression
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil::parser::expression::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("2 + 3 * 4", Config::default());
/// let (rest, expr) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::expression::parse;
/// use oneil::parser::{Config, Span};
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
/// use oneil::parser::expression::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("2 + 3 * 4", Config::default());
/// let (rest, expr) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::expression::parse_complete;
/// use oneil::parser::{Config, Span};
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
    or_expr
        .map_error(|e| ParserError::new(ParserErrorKind::ExpectExpr, e.span))
        .parse(input)
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
    (not.convert_errors(), cut(not_expr))
        .map(|(_, expr)| Expr::UnaryOp {
            op: UnaryOp::Not,
            expr: Box::new(expr),
        })
        .or(comparison_expr)
        .parse(input)
}

/// Parses a comparison expression
fn comparison_expr(input: Span) -> Result<Expr, ParserError> {
    let op = alt((
        value(BinaryOp::LessThanEq, less_than_equals),
        value(BinaryOp::GreaterThanEq, greater_than_equals),
        value(BinaryOp::LessThan, less_than),
        value(BinaryOp::GreaterThan, greater_than),
        value(BinaryOp::Eq, equals_equals),
        value(BinaryOp::NotEq, bang_equals),
    ))
    .convert_errors();

    (minmax_expr, opt((op, cut(minmax_expr))))
        .map(|(left, rest)| match rest {
            Some((op, right)) => Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            None => left,
        })
        .parse(input)
}

/// Parses a min/max expression
///
/// Ex: `min_weight | max_weight`
fn minmax_expr(input: Span) -> Result<Expr, ParserError> {
    ((
        additive_expr,
        opt((
            value(BinaryOp::MinMax, bar).convert_errors(),
            cut(additive_expr),
        )),
    ))
        .map(|(left, rest)| match rest {
            Some((op, right)) => Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            None => left,
        })
        .parse(input)
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
    let op = value(BinaryOp::Pow, caret).convert_errors();
    (neg_expr, opt((op, cut(exponential_expr))))
        .map(|(left, rest)| match rest {
            Some((op, right)) => Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            None => left,
        })
        .parse(input)
}

/// Parses a negation expression
fn neg_expr(input: Span) -> Result<Expr, ParserError> {
    (minus.convert_errors(), cut(neg_expr))
        .map(|(_, expr)| Expr::UnaryOp {
            op: UnaryOp::Neg,
            expr: Box::new(expr),
        })
        .or(primary_expr)
        .parse(input)
}

/// Parses a primary expression (literals, identifiers, function calls, parenthesized expressions)
fn primary_expr(input: Span) -> Result<Expr, ParserError> {
    alt((
        map_res(number.convert_errors(), |n| {
            let parse_result = n.fragment().parse::<f64>();
            match parse_result {
                Ok(n) => Ok(Expr::Literal(Literal::Number(n))),
                Err(_) => Err(ParserErrorKind::InvalidNumber(n.fragment())),
            }
        }),
        map(string.convert_errors(), |s| {
            // trim quotes from the string
            let s_contents = s.fragment()[1..s.len() - 1].to_string();
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
        map(
            (
                paren_left.convert_errors(),
                cut((expr, paren_right.convert_errors())),
            ),
            |(_, (e, _))| e,
        ),
    ))
    .parse(input)
}

/// Parses a function call
fn function_call(input: Span) -> Result<Expr, ParserError> {
    (
        identifier.convert_errors(),
        paren_left.convert_errors(),
        cut((
            separated_list0(comma.convert_errors(), expr),
            paren_right.convert_errors(),
        )),
    )
        .map(|(name, _, (args, _))| Expr::FunctionCall {
            name: name.to_string(),
            args: args,
        })
        .parse(input)
}

/// Parses a variable name
fn variable(input: Span) -> Result<Expr, ParserError> {
    separated_list1(dot, identifier)
        .convert_errors()
        .map(|ids| Expr::Variable(ids.into_iter().map(|id| id.to_string()).collect()))
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Config;

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
        assert_eq!(expr, Expr::Variable(vec!["foo".to_string()]));
    }

    #[test]
    fn test_primary_expr_multiword_identifier() {
        let input = Span::new_extra("foo.bar.baz", Config::default());
        let (_, expr) = parse(input).unwrap();
        assert_eq!(
            expr,
            Expr::Variable(vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string()
            ])
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
