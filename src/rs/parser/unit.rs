//! Unit expression parsing for the Oneil language.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::unit::parse;
//! use oneil::parser::Span;
//!
//! // Parse a simple unit
//! let input = Span::new("kg");
//! let (_, unit) = parse(input).unwrap();
//!
//! // Parse a compound unit
//! let input = Span::new("m/s^2");
//! let (_, unit) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    branch::alt,
    combinator::{cut, map, opt},
    multi::many0,
};

use crate::ast::unit::{UnitExpr, UnitOp};

use super::{
    token::{
        literal::number,
        naming::identifier,
        symbol::{caret, paren_left, paren_right, slash, star},
    },
    util::{Result, Span},
};

/// Parses a unit expression
pub fn parse(input: Span) -> Result<UnitExpr> {
    unit_expr(input)
}

/// Parses a unit expression
fn unit_expr(input: Span) -> Result<UnitExpr> {
    let op = alt((
        map(star, |_| UnitOp::Multiply),
        map(slash, |_| UnitOp::Divide),
    ));

    (unit_term, many0((op, cut(unit_term))))
        .map(|(first, rest)| {
            rest.into_iter()
                .fold(first, |acc, (op, expr)| UnitExpr::BinaryOp {
                    op,
                    left: Box::new(acc),
                    right: Box::new(expr),
                })
        })
        .parse(input)
}

/// Parses a unit term
fn unit_term(input: Span) -> Result<UnitExpr> {
    let parse_unit = map((identifier, opt((caret, cut(number)))), |(id, exp)| {
        let exponent = exp.map(|(_, n)| {
            // TODO(error): Better error handling for float parsing
            n.fragment()
                .parse::<f64>()
                .expect("TODO: better error handling for float parsing")
        });
        UnitExpr::Unit {
            identifier: id.to_string(),
            exponent,
        }
    });

    let parse_parenthesized = map(
        (paren_left, cut((unit_expr, paren_right))),
        |(_, (expr, _))| expr,
    );

    parse_unit.or(parse_parenthesized).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_unit() {
        let input = Span::new("kg");
        let (_, unit) = parse(input).unwrap();
        assert_eq!(
            unit,
            UnitExpr::Unit {
                identifier: "kg".to_string(),
                exponent: None
            }
        );
    }

    #[test]
    fn test_unit_with_exponent() {
        let input = Span::new("m^2");
        let (_, unit) = parse(input).unwrap();
        assert_eq!(
            unit,
            UnitExpr::Unit {
                identifier: "m".to_string(),
                exponent: Some(2.0)
            }
        );
    }

    #[test]
    fn test_compound_unit_multiply() {
        let input = Span::new("kg*m");
        let (_, unit) = parse(input).unwrap();
        assert!(matches!(
            unit,
            UnitExpr::BinaryOp {
                op: UnitOp::Multiply,
                ..
            }
        ));
    }

    #[test]
    fn test_compound_unit_divide() {
        let input = Span::new("m/s");
        let (_, unit) = parse(input).unwrap();
        assert!(matches!(
            unit,
            UnitExpr::BinaryOp {
                op: UnitOp::Divide,
                ..
            }
        ));
    }

    #[test]
    fn test_complex_unit() {
        let input = Span::new("m^2*kg/s^2");
        let (_, unit) = parse(input).unwrap();
        assert!(matches!(
            unit,
            UnitExpr::BinaryOp {
                op: UnitOp::Divide,
                ..
            }
        ));
    }

    #[test]
    fn test_parenthesized_unit() {
        let input = Span::new("(kg*m)/s^2");
        let (_, unit) = parse(input).unwrap();
        assert!(matches!(
            unit,
            UnitExpr::BinaryOp {
                op: UnitOp::Divide,
                ..
            }
        ));
    }
}
