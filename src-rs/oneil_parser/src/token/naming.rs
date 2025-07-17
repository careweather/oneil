//! Provides parsers for identifiers and labels in the Oneil language.
//!
//! This module contains parsers for identifiers (variable names, function names, etc.)
//! and labels (section names, test names, etc.). Identifiers follow standard programming
//! language rules, while labels are more permissive to allow for descriptive names.

use nom::{
    Parser as _, bytes::complete::take_while, character::complete::satisfy, combinator::verify,
};

use crate::token::{
    Result, Span,
    error::TokenError,
    keyword,
    util::{Token, token},
};

/// Parses an identifier (alphabetic or underscore, then alphanumeric or underscore).
pub fn identifier(input: Span) -> Result<Token, TokenError> {
    verify(
        token(
            |input| {
                let (rest, _) = satisfy(|c: char| c.is_alphabetic() || c == '_').parse(input)?;
                let (rest, _) =
                    take_while(|c: char| c.is_alphanumeric() || c == '_').parse(rest)?;
                Ok((rest, ()))
            },
            TokenError::expected_identifier,
        ),
        |identifier| !keyword::KEYWORDS.contains(&identifier.lexeme()),
    )
    .parse(input)
}

/// Parses a label (like an identifier but can contain spaces, tabs, and dashes).
///
/// Note that labels are often followed by a colon as a delimiter, but other
/// tokens (such as a linebreak) can also be used.
pub fn label(input: Span) -> Result<Token, TokenError> {
    // TODO: verify that the label is not a keyword
    token(
        |input| {
            let (rest, _) = satisfy(|c: char| c.is_alphanumeric() || c == '_').parse(input)?;

            let (rest, _) = take_while(|c: char| {
                c.is_alphanumeric() || c == '_' || c == '-' || c == '\'' || c == ' ' || c == '\t'
            })
            .parse(rest)?;

            Ok((rest, ()))
        },
        TokenError::expected_label,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Span};

    #[test]
    fn test_identifier_basic() {
        let input = Span::new_extra("foo rest", Config::default());
        let (rest, matched) = identifier(input).expect("should parse basic identifier");
        assert_eq!(matched.lexeme(), "foo");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_identifier_underscore() {
        let input = Span::new_extra("_foo123 bar", Config::default());
        let (rest, matched) = identifier(input).expect("should parse identifier with underscore");
        assert_eq!(matched.lexeme(), "_foo123");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_identifier_only_underscore() {
        let input = Span::new_extra("_ rest", Config::default());
        let (rest, matched) = identifier(input).expect("should parse single underscore identifier");
        assert_eq!(matched.lexeme(), "_");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_label_basic() {
        let input = Span::new_extra("foo-bar: rest", Config::default());
        let (rest, matched) = label(input).expect("should parse label with dash");
        assert_eq!(matched.lexeme(), "foo-bar");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_spaces_and_tabs() {
        let input = Span::new_extra("foo bar\tbaz: rest", Config::default());
        let (rest, matched) = label(input).expect("should parse label with spaces and tabs");
        assert_eq!(matched.lexeme(), "foo bar\tbaz");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_numbers() {
        let input = Span::new_extra("123Test: rest", Config::default());
        let (rest, matched) = label(input).expect("should parse label with numbers");
        assert_eq!(matched.lexeme(), "123Test");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_invalid_start() {
        let input = Span::new_extra("-foo", Config::default());
        let res = label(input);
        assert!(res.is_err(), "should not parse label starting with dash");
    }

    #[test]
    fn test_label_only_underscore() {
        let input = Span::new_extra("_: rest", Config::default());
        let (rest, matched) = label(input).expect("should parse label with only underscore");
        assert_eq!(matched.lexeme(), "_");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_multiple_dashes() {
        let input = Span::new_extra("foo-bar-baz: rest", Config::default());
        let (rest, matched) = label(input).expect("should parse label with multiple dashes");
        assert_eq!(matched.lexeme(), "foo-bar-baz");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_trailing_whitespace() {
        let input = Span::new_extra("foo : rest", Config::default());
        let (rest, matched) = label(input).expect("should parse label with trailing whitespace");
        assert_eq!(matched.lexeme(), "foo ");
        assert_eq!(rest.fragment(), &": rest");
    }
}
