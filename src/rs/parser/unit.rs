//! Unit expression parsing for the Oneil language.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::unit::parse;
//! use oneil::parser::{Config, Span};
//!
//! // Parse a simple unit
//! let input = Span::new_extra("kg", Config::default());
//! let (_, unit) = parse(input).unwrap();
//!
//! // Parse a compound unit
//! let input = Span::new_extra("m/s^2", Config::default());
//! let (_, unit) = parse(input).unwrap();
//! ```

use nom::{
    Parser as _,
    branch::alt,
    combinator::{all_consuming, cut, map, opt},
    multi::many0,
};

use crate::ast::unit::{UnitExpr, UnitOp};

use super::{
    error::{ErrorHandlingParser as _, ParserError},
    token::{
        literal::number,
        naming::identifier,
        symbol::{caret, paren_left, paren_right, slash, star},
    },
    util::{Result, Span},
};

/// Parses a unit expression
///
/// This function **may not consume the complete input**.
///
/// # Examples
///
/// ```
/// use oneil::parser::unit::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("m/s^2", Config::default());
/// let (rest, unit) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::unit::parse;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("m/s^2 rest", Config::default());
/// let (rest, unit) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"rest");
/// ```
pub fn parse(input: Span) -> Result<UnitExpr, ParserError> {
    unit_expr(input)
}

/// Parses a unit expression
///
/// This function **fails if the complete input is not consumed**.
///
/// # Examples
///
/// ```
/// use oneil::parser::unit::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("m/s^2", Config::default());
/// let (rest, unit) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil::parser::unit::parse_complete;
/// use oneil::parser::{Config, Span};
///
/// let input = Span::new_extra("m/s^2 rest", Config::default());
/// let result = parse_complete(input);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse_complete(input: Span) -> Result<UnitExpr, ParserError> {
    all_consuming(unit_expr).parse(input)
}

/// Parses a unit expression
fn unit_expr(input: Span) -> Result<UnitExpr, ParserError> {
    let op = alt((
        map(star, |_| UnitOp::Multiply),
        map(slash, |_| UnitOp::Divide),
    ));

    (unit_term, many0((op.errors_into(), cut(unit_term))))
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
fn unit_term(input: Span) -> Result<UnitExpr, ParserError> {
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
    })
    .errors_into();

    let parse_parenthesized = map(
        (
            paren_left.errors_into(),
            cut((unit_expr, paren_right.errors_into())),
        ),
        |(_, (expr, _))| expr,
    );

    parse_unit.or(parse_parenthesized).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Config;

    #[test]
    fn test_simple_unit() {
        let input = Span::new_extra("kg", Config::default());
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
        let input = Span::new_extra("m^2", Config::default());
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
        let input = Span::new_extra("kg*m", Config::default());
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
        let input = Span::new_extra("m/s", Config::default());
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
        let input = Span::new_extra("m^2*kg/s^2", Config::default());
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
        let input = Span::new_extra("(kg*m)/s^2", Config::default());
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
    fn test_parse_complete_success() {
        let input = Span::new_extra("kg", Config::default());
        let (rest, unit) = parse_complete(input).unwrap();
        assert_eq!(
            unit,
            UnitExpr::Unit {
                identifier: "kg".to_string(),
                exponent: None
            }
        );
        assert_eq!(rest.fragment(), &"");
    }

    #[test]
    fn test_parse_complete_with_remaining_input() {
        let input = Span::new_extra("kg rest", Config::default());
        let result = parse_complete(input);
        assert!(result.is_err());
    }
}
