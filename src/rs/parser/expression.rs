//! Expression parsing for the Oneil language.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::expression::parse;
//! use oneil::parser::util::Span;
//!
//! // Parse a simple arithmetic expression
//! let input = Span::new("2 + 3 * 4");
//! let (_, expr) = parse(input).unwrap();
//!
//! // Parse a complex expression with multiple operators
//! let input = Span::new("-(x + y)^2 < foo(a, b) and not c");
//! let (_, expr) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    branch::alt,
    combinator::{cut, map, map_res, opt, value},
    multi::{fold_many0, many0, separated_list0},
    sequence::tuple,
};

use crate::ast::{
    Expr, Literal,
    expression::{BinaryOp, UnaryOp},
};

use super::{
    token::{
        keyword::{and, false_, not, or, true_},
        literal::{number, string},
        naming::identifier,
        symbol::{
            bang_equals, caret, comma, equals_equals, greater_than, greater_than_equals, less_than,
            less_than_equals, minus, minus_minus, paren_left, paren_right, percent, plus, slash,
            slash_slash, star,
        },
    },
    util::{Parser, Result, Span},
};

fn left_associative_binary_op<'a>(
    operand: impl Parser<'a, Expr> + Copy,
    operator: impl Parser<'a, BinaryOp>,
) -> impl Parser<'a, Expr> {
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
pub fn parse(input: Span) -> Result<Expr> {
    expr(input)
}

/// Parses an expression
fn expr(input: Span) -> Result<Expr> {
    or_expr(input)
}

/// Parses an OR expression (lowest precedence)
fn or_expr(input: Span) -> Result<Expr> {
    let or = value(BinaryOp::Or, or);
    left_associative_binary_op(and_expr, or).parse(input)
}

/// Parses an AND expression
fn and_expr(input: Span) -> Result<Expr> {
    let and = value(BinaryOp::And, and);
    left_associative_binary_op(not_expr, and).parse(input)
}

/// Parses a NOT expression
fn not_expr(input: Span) -> Result<Expr> {
    (not, cut(not_expr))
        .map(|(_, expr)| Expr::UnaryOp {
            op: UnaryOp::Not,
            expr: Box::new(expr),
        })
        .or(comparison_expr)
        .parse(input)
}

/// Parses a comparison expression
fn comparison_expr(input: Span) -> Result<Expr> {
    let op = alt((
        value(BinaryOp::LessThanEq, less_than_equals),
        value(BinaryOp::GreaterThanEq, greater_than_equals),
        value(BinaryOp::LessThan, less_than),
        value(BinaryOp::GreaterThan, greater_than),
        value(BinaryOp::Eq, equals_equals),
        value(BinaryOp::NotEq, bang_equals),
    ));

    (additive_expr, opt((op, cut(additive_expr))))
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
fn additive_expr(input: Span) -> Result<Expr> {
    let op = alt((
        value(BinaryOp::Add, plus),
        value(BinaryOp::Sub, minus),
        value(BinaryOp::TrueSub, minus_minus),
    ));
    left_associative_binary_op(multiplicative_expr, op).parse(input)
}

/// Parses a multiplicative expression
fn multiplicative_expr(input: Span) -> Result<Expr> {
    let op = alt((
        value(BinaryOp::Mul, star),
        value(BinaryOp::Div, slash),
        value(BinaryOp::TrueDiv, slash_slash),
        value(BinaryOp::Mod, percent),
    ));
    left_associative_binary_op(exponential_expr, op).parse(input)
}

/// Parses an exponential expression (right associative)
fn exponential_expr(input: Span) -> Result<Expr> {
    let op = value(BinaryOp::Pow, caret);
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
fn neg_expr(input: Span) -> Result<Expr> {
    (minus, cut(neg_expr))
        .map(|(_, expr)| Expr::UnaryOp {
            op: UnaryOp::Neg,
            expr: Box::new(expr),
        })
        .or(primary_expr)
        .parse(input)
}

/// Parses a primary expression (literals, identifiers, function calls, parenthesized expressions)
fn primary_expr(input: Span) -> Result<Expr> {
    alt((
        map(number, |n| {
            // TODO(error): Better error handling for float parsing
            let parse_result = n
                .fragment()
                .parse::<f64>()
                .expect("TODO: better error handling for float parsing");
            Expr::Literal(Literal::Number(parse_result))
        }),
        map(string, |s| {
            // trim quotes from the string
            let s_contents = s.fragment()[1..s.len() - 1].to_string();
            Expr::Literal(Literal::String(s_contents))
        }),
        map(true_, |_| Expr::Literal(Literal::Boolean(true))),
        map(false_, |_| Expr::Literal(Literal::Boolean(false))),
        function_call,
        map(identifier, |id| Expr::Variable(id.to_string())),
        map((paren_left, cut((expr, paren_right))), |(_, (e, _))| e),
    ))
    .parse(input)
}

/// Parses a function call
fn function_call(input: Span) -> Result<Expr> {
    (
        identifier,
        paren_left,
        cut((separated_list0(comma, expr), paren_right)),
    )
        .map(|(name, _, (args, _))| Expr::FunctionCall {
            name: name.to_string(),
            args: args,
        })
        .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_expr_number() {
        let input = Span::new("42");
        let (_, expr) = primary_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Number(42.0)));
    }

    #[test]
    fn test_primary_expr_string() {
        let input = Span::new("\"hello\"");
        let (_, expr) = primary_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::String("hello".to_string())));
    }

    #[test]
    fn test_primary_expr_boolean_true() {
        let input = Span::new("true");
        let (_, expr) = primary_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_primary_expr_boolean_false() {
        let input = Span::new("false");
        let (_, expr) = primary_expr(input).unwrap();
        assert_eq!(expr, Expr::Literal(Literal::Boolean(false)));
    }

    #[test]
    fn test_primary_expr_identifier() {
        let input = Span::new("foo");
        let (_, expr) = primary_expr(input).unwrap();
        assert_eq!(expr, Expr::Variable("foo".to_string()));
    }

    #[test]
    fn test_function_call() {
        let input = Span::new("foo(1, 2)");
        let (_, expr) = function_call(input).unwrap();
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
        let input = Span::new("-42");
        let (_, expr) = neg_expr(input).unwrap();
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
        let input = Span::new("2^3");
        let (_, expr) = exponential_expr(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Pow, ..
            } => (),
            _ => panic!("Expected power operation"),
        }
    }

    #[test]
    fn test_multiplicative_expr() {
        let input = Span::new("2*3");
        let (_, expr) = multiplicative_expr(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Mul, ..
            } => (),
            _ => panic!("Expected multiplication"),
        }
    }

    #[test]
    fn test_additive_expr() {
        let input = Span::new("2+3");
        let (_, expr) = additive_expr(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Add, ..
            } => (),
            _ => panic!("Expected addition"),
        }
    }

    #[test]
    fn test_comparison_expr() {
        let input = Span::new("2<3");
        let (_, expr) = comparison_expr(input).unwrap();
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
        let input = Span::new("not true");
        let (_, expr) = not_expr(input).unwrap();
        match expr {
            Expr::UnaryOp {
                op: UnaryOp::Not, ..
            } => (),
            _ => panic!("Expected not operation"),
        }
    }

    #[test]
    fn test_and_expr() {
        let input = Span::new("true and false");
        let (_, expr) = and_expr(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::And, ..
            } => (),
            _ => panic!("Expected and operation"),
        }
    }

    #[test]
    fn test_or_expr() {
        let input = Span::new("true or false");
        let (_, expr) = or_expr(input).unwrap();
        match expr {
            Expr::BinaryOp {
                op: BinaryOp::Or, ..
            } => (),
            _ => panic!("Expected or operation"),
        }
    }

    #[test]
    fn test_complex_expr() {
        let input = Span::new("-(2 + 3*4^2) < foo(5, 6) and not bar");
        let (_, expr) = expr(input).unwrap();
        // The exact structure is complex but we just verify it parses
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                op: BinaryOp::And,
                ..
            }
        ));
    }
}
