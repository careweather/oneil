//! Provides parsers for literal values in the Oneil language.
//!
//! This module contains parsers for numeric and string literals. The numeric parser
//! supports integers and floating point numbers with optional signs and exponents.
//! The string parser handles double-quoted string literals.

use nom::{
    Parser as _,
    bytes::complete::{tag, take_while},
    character::complete::{digit1, one_of},
    combinator::opt,
};

use crate::token::{
    Result, Span,
    error::{ErrorHandlingParser, TokenError},
    util::{Token, token},
};

/// Parses a number literal, supporting optional sign, decimal, and exponent.
///
/// All valid numbers should be parsed correctly because they conform to the
/// grammar noted here:
/// https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64
///
/// Therefore, when this parser is used, we can use the following pattern to convert it to an f64:
/// ```ignore
/// let parse_result = n.lexeme().parse::<f64>();
/// let parse_result = parse_result.expect("all valid numbers should parse correctly");
/// ```
pub fn number(input: Span) -> Result<Token, TokenError> {
    let opt_sign = opt(one_of("+-"));

    let digit = digit1;

    let opt_decimal = opt(|input| -> Result<_, TokenError> {
        let (rest, decimal_point_span) = tag(".").parse(input)?;
        let (rest, _) = digit1
            .or_fail_with(TokenError::invalid_decimal_part(decimal_point_span))
            .parse(rest)?;
        Ok((rest, ()))
    });

    let opt_exponent = opt(|input| {
        let (rest, e_span) = tag("e").or(tag("E")).parse(input)?;
        let (rest, _) = opt(one_of("+-")).parse(rest)?;
        let (rest, _) = digit1
            .or_fail_with(TokenError::invalid_exponent_part(e_span))
            .parse(rest)?;
        Ok((rest, ()))
    });

    token(
        (opt_sign, digit, opt_decimal, opt_exponent),
        TokenError::expected_number,
    )
    .parse(input)
}

/// Parses a string literal delimited by double quotes.
pub fn string(input: Span) -> Result<Token, TokenError> {
    token(
        |input| {
            let (rest, open_quote_span) = tag("\"").parse(input)?;
            let (rest, _) = take_while(|c: char| c != '"' && c != '\n').parse(rest)?;
            let (rest, _) = tag("\"")
                .or_fail_with(TokenError::unclosed_string(open_quote_span))
                .parse(rest)?;
            Ok((rest, ()))
        },
        TokenError::expected_string,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Span};

    #[test]
    fn test_number_integer() {
        let input = Span::new_extra("42 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse integer");
        assert_eq!(matched.lexeme(), "42");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_negative_integer() {
        let input = Span::new_extra("-17 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse negative integer");
        assert_eq!(matched.lexeme(), "-17");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_decimal() {
        let input = Span::new_extra("3.1415 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse decimal");
        assert_eq!(matched.lexeme(), "3.1415");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_exponent() {
        let input = Span::new_extra("2.5e10 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse exponent");
        assert_eq!(matched.lexeme(), "2.5e10");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_negative_exponent() {
        let input = Span::new_extra("-1.2E-3 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse negative exponent");
        assert_eq!(matched.lexeme(), "-1.2E-3");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_invalid() {
        let input = Span::new_extra("foo", Config::default());
        let res = number(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_string_simple() {
        let input = Span::new_extra("\"hello\" rest", Config::default());
        let (rest, matched) = string(input).expect("should parse string");
        assert_eq!(matched.lexeme(), "\"hello\"");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_string_with_spaces() {
        let input = Span::new_extra("\"foo bar\" baz", Config::default());
        let (rest, matched) = string(input).expect("should parse string with spaces");
        assert_eq!(matched.lexeme(), "\"foo bar\"");
        assert_eq!(rest.fragment(), &"baz");
    }

    #[test]
    fn test_string_escape_sequences_not_supported() {
        let input = Span::new_extra("\"foo \\\" bar", Config::default());
        let (rest, matched) =
            string(input).expect("should parse string (escape sequences not supported)");
        assert_eq!(matched.lexeme(), "\"foo \\\"");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_string_unterminated() {
        let input = Span::new_extra("\"unterminated", Config::default());
        let res = string(input);
        assert!(res.is_err(), "should not parse unterminated string");
    }
}
