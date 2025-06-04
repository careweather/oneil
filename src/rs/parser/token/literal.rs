use nom::{
    Parser as _,
    bytes::complete::take_while,
    character::complete::{char, digit1},
    combinator::{cut, opt},
};

use super::{Result, Span, util::token};

/// Parses a number literal, supporting optional sign, decimal, and exponent.
pub fn number(input: Span) -> Result<Span> {
    let sign1 = opt(char('+').or(char('-')));
    let sign2 = opt(char('+').or(char('-')));
    let e = char('e').or(char('E'));
    token((
        opt(sign1),
        digit1,
        opt((char('.'), cut(digit1))),
        opt((e, cut((sign2, digit1)))),
    ))
    .parse(input)
}

/// Parses a string literal delimited by double quotes.
pub fn string(input: Span) -> Result<Span> {
    token((
        char('"'),
        cut((take_while(|c: char| c != '"' && c != '\n'), char('"'))),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::token::Span;

    #[test]
    fn test_number_integer() {
        let input = Span::new("42 rest");
        let (rest, matched) = number(input).expect("should parse integer");
        assert_eq!(matched.fragment(), &"42");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_negative_integer() {
        let input = Span::new("-17 rest");
        let (rest, matched) = number(input).expect("should parse negative integer");
        assert_eq!(matched.fragment(), &"-17");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_decimal() {
        let input = Span::new("3.1415 rest");
        let (rest, matched) = number(input).expect("should parse decimal");
        assert_eq!(matched.fragment(), &"3.1415");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_exponent() {
        let input = Span::new("2.5e10 rest");
        let (rest, matched) = number(input).expect("should parse exponent");
        assert_eq!(matched.fragment(), &"2.5e10");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_negative_exponent() {
        let input = Span::new("-1.2E-3 rest");
        let (rest, matched) = number(input).expect("should parse negative exponent");
        assert_eq!(matched.fragment(), &"-1.2E-3");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_number_invalid() {
        let input = Span::new("foo");
        let res = number(input);
        assert!(res.is_err());
    }

    #[test]
    fn test_string_simple() {
        let input = Span::new("\"hello\" rest");
        let (rest, matched) = string(input).expect("should parse string");
        assert_eq!(matched.fragment(), &"\"hello\"");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_string_with_spaces() {
        let input = Span::new("\"foo bar\" baz");
        let (rest, matched) = string(input).expect("should parse string with spaces");
        assert_eq!(matched.fragment(), &"\"foo bar\"");
        assert_eq!(rest.fragment(), &"baz");
    }

    #[test]
    fn test_string_escape_sequences_not_supported() {
        let input = Span::new("\"foo \\\" bar");
        let (rest, matched) =
            string(input).expect("should parse string (escape sequences not supported)");
        assert_eq!(matched.fragment(), &"\"foo \\\"");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_string_unterminated() {
        let input = Span::new("\"unterminated");
        let res = string(input);
        assert!(res.is_err(), "should not parse unterminated string");
    }
}
