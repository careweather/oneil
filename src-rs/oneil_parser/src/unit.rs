//! Unit expression parsing for the Oneil language.
//!
//! # Examples
//!
//! ```
//! use oneil_parser::unit::parse;
//! use oneil_parser::{Config, Span};
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

use oneil_ast::unit::{UnitExpr, UnitOp};

use crate::{
    error::{ErrorHandlingParser, ParserError},
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
/// use oneil_parser::unit::parse;
/// use oneil_parser::{Config, Span};
///
/// let input = Span::new_extra("m/s^2", Config::default());
/// let (rest, unit) = parse(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil_parser::unit::parse;
/// use oneil_parser::{Config, Span};
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
/// use oneil_parser::unit::parse_complete;
/// use oneil_parser::{Config, Span};
///
/// let input = Span::new_extra("m/s^2", Config::default());
/// let (rest, unit) = parse_complete(input).unwrap();
/// assert_eq!(rest.fragment(), &"");
/// ```
///
/// ```
/// use oneil_parser::unit::parse_complete;
/// use oneil_parser::{Config, Span};
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
    let (rest, first_term) = unit_term.map_error(ParserError::expect_unit).parse(input)?;

    let (rest, rest_terms) = many0(|input| {
        let op = alt((
            map(star, |_| UnitOp::Multiply),
            map(slash, |_| UnitOp::Divide),
        ));

        let (rest, op) = op.convert_errors().parse(input)?;
        let (rest, term) =
            cut(unit_term.map_error(ParserError::unit_missing_second_term(op))).parse(rest)?;
        Ok((rest, (op, term)))
    })
    .parse(rest)?;

    let expr = rest_terms
        .into_iter()
        .fold(first_term, |acc, (op, expr)| UnitExpr::BinaryOp {
            op,
            left: Box::new(acc),
            right: Box::new(expr),
        });

    Ok((rest, expr))
}

/// Parses a unit term
fn unit_term(input: Span) -> Result<UnitExpr, ParserError> {
    let parse_unit = |input| {
        let (rest, id) = identifier.convert_errors().parse(input)?;
        let (rest, exp) = opt(|input| {
            let (rest, caret_token) = caret.convert_errors().parse(input)?;
            let (rest, exp) =
                cut(number.map_error(ParserError::unit_missing_exponent(caret_token)))
                    .parse(rest)?;
            Ok((rest, exp))
        })
        .parse(rest)?;

        let exp = exp.map(|n| {
            let parse_result = n.lexeme().parse::<f64>();
            parse_result.map_err(|_| n)
        });

        let exp = match exp {
            Some(Ok(exp)) => Some(exp),
            Some(Err(n)) => {
                return Err(nom::Err::Failure(ParserError::invalid_number(n)()));
            }
            None => None,
        };

        let expr = UnitExpr::Unit {
            identifier: id.lexeme().to_string(),
            exponent: exp,
        };

        Ok((rest, expr))
    };

    let parse_parenthesized = |input| {
        let (rest, paren_left_token) = paren_left.convert_errors().parse(input)?;

        let (rest, expr) =
            cut(unit_expr.map_error(ParserError::unit_paren_missing_expr(paren_left_token)))
                .parse(rest)?;

        let (rest, _) = cut(paren_right.map_error(ParserError::unclosed_paren(paren_left_token)))
            .parse(rest)?;

        Ok((rest, expr))
    };

    alt((parse_unit, parse_parenthesized)).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

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
