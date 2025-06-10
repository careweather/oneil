//! Provides parsers for literal values in the Oneil language.
//!
//! This module contains parsers for numeric and string literals. The numeric parser
//! supports integers and floating point numbers with optional signs and exponents.
//! The string parser handles double-quoted string literals.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::token::literal::{number, string};
//! use oneil::parser::{Config, Span};
//!
//! // Parse a floating point number
//! let input = Span::new_extra("-42.5e-2", Config::default());
//! let (rest, matched) = number(input).unwrap();
//! assert_eq!(matched.fragment(), &"-42.5e-2");
//!
//! // Parse a string literal
//! let input = Span::new_extra("\"hello world\" rest", Config::default());
//! let (rest, matched) = string(input).unwrap();
//! assert_eq!(matched.fragment(), &"\"hello world\"");
//! ```

use nom::{
    Parser as _,
    bytes::complete::{tag, take_while},
    character::complete::{digit1, one_of},
    combinator::{cut, flat_map, opt},
};

use super::{
    Result, Span,
    error::{NumberError, StringError, TokenError, TokenErrorKind},
    util::token,
};
use crate::parser::error::ErrorHandlingParser as _;

/// Parses a number literal, supporting optional sign, decimal, and exponent.
pub fn number(input: Span) -> Result<Span, TokenError> {
    fn number_error<'a>(
        e: NumberError<'a>,
    ) -> impl Fn(nom::error::Error<Span<'a>>) -> TokenError<'a> {
        move |err| TokenError::new(TokenErrorKind::Number(e), err.input)
    }

    // Needed for type inference
    let tag = tag::<_, _, nom::error::Error<Span>>;
    let digit1 = digit1::<_, nom::error::Error<Span>>;
    let one_of = one_of::<_, _, nom::error::Error<Span>>;

    let opt_sign = opt(one_of("+-").convert_errors());

    let digit = digit1.convert_errors();

    let opt_decimal = opt(flat_map(tag(".").convert_errors(), |decimal_point_span| {
        cut(digit1).map_failure(number_error(NumberError::InvalidDecimalPart {
            decimal_point_span,
        }))
    }));

    let opt_exponent = opt(flat_map(tag("e").or(tag("E")).convert_errors(), |e_span| {
        (
            opt(tag("+").or(tag("-")).convert_errors()),
            cut(digit1).map_failure(number_error(NumberError::InvalidExponentPart { e_span })),
        )
    }));

    token(
        (opt_sign, digit, opt_decimal, opt_exponent),
        TokenErrorKind::Number(NumberError::ExpectNumber),
    )
    .parse(input)
}

/// Parses a string literal delimited by double quotes.
pub fn string(input: Span) -> Result<Span, TokenError> {
    let unterminated_string_error = |open_quote_span| {
        TokenErrorKind::String(StringError::UnterminatedString { open_quote_span })
    };

    // Needed for type inference
    let tag = tag::<_, _, nom::error::Error<Span>>;
    let take_while = take_while::<_, _, nom::error::Error<Span>>;

    token(
        flat_map(tag("\"").convert_errors(), |open_quote_span: Span| {
            (
                take_while(|c: char| c != '"' && c != '\n').convert_errors(),
                cut(tag("\"")).map_failure(move |e: nom::error::Error<Span>| {
                    TokenError::new(unterminated_string_error(open_quote_span), e.input)
                }),
            )
        }),
        TokenErrorKind::String(StringError::ExpectString),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Config, Span};

    #[test]
    fn test_number_integer() {
        let input = Span::new_extra("42 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse integer");
        assert_eq!(matched.fragment(), &"42");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_negative_integer() {
        let input = Span::new_extra("-17 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse negative integer");
        assert_eq!(matched.fragment(), &"-17");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_decimal() {
        let input = Span::new_extra("3.1415 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse decimal");
        assert_eq!(matched.fragment(), &"3.1415");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_exponent() {
        let input = Span::new_extra("2.5e10 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse exponent");
        assert_eq!(matched.fragment(), &"2.5e10");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_negative_exponent() {
        let input = Span::new_extra("-1.2E-3 rest", Config::default());
        let (rest, matched) = number(input).expect("should parse negative exponent");
        assert_eq!(matched.fragment(), &"-1.2E-3");
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
        assert_eq!(matched.fragment(), &"\"hello\"");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_string_with_spaces() {
        let input = Span::new_extra("\"foo bar\" baz", Config::default());
        let (rest, matched) = string(input).expect("should parse string with spaces");
        assert_eq!(matched.fragment(), &"\"foo bar\"");
        assert_eq!(rest.fragment(), &"baz");
    }

    #[test]
    fn test_string_escape_sequences_not_supported() {
        let input = Span::new_extra("\"foo \\\" bar", Config::default());
        let (rest, matched) =
            string(input).expect("should parse string (escape sequences not supported)");
        assert_eq!(matched.fragment(), &"\"foo \\\"");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_string_unterminated() {
        let input = Span::new_extra("\"unterminated", Config::default());
        let res = string(input);
        assert!(res.is_err(), "should not parse unterminated string");
    }
}
