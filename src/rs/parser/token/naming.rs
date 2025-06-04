use nom::{Parser as _, bytes::complete::take_while, character::complete::satisfy};

use super::{Result, Span, util::token};

/// Parses an identifier (alphabetic or underscore, then alphanumeric or underscore).
pub fn identifier(input: Span) -> Result<Span> {
    token((
        satisfy(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

/// Parses a label (alphabetic or underscore, then alphanumeric, underscore, dash, space, or tab).
pub fn label(input: Span) -> Result<Span> {
    token((
        satisfy(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ' || c == '\t'),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::token::Span;

    #[test]
    fn test_identifier_basic() {
        let input = Span::new("foo rest");
        let (rest, matched) = identifier(input).expect("should parse basic identifier");
        assert_eq!(matched.fragment(), &"foo");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_identifier_underscore() {
        let input = Span::new("_foo123 bar");
        let (rest, matched) = identifier(input).expect("should parse identifier with underscore");
        assert_eq!(matched.fragment(), &"_foo123");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_identifier_invalid() {
        let input = Span::new("123abc");
        let res = identifier(input);
        assert!(
            res.is_err(),
            "should not parse identifier starting with digit"
        );
    }

    #[test]
    fn test_identifier_only_underscore() {
        let input = Span::new("_ rest");
        let (rest, matched) = identifier(input).expect("should parse single underscore identifier");
        assert_eq!(matched.fragment(), &"_");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_label_basic() {
        let input = Span::new("foo-bar: rest");
        let (rest, matched) = label(input).expect("should parse label with dash");
        assert_eq!(matched.fragment(), &"foo-bar");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_spaces_and_tabs() {
        let input = Span::new("foo bar\tbaz: rest");
        let (rest, matched) = label(input).expect("should parse label with spaces and tabs");
        assert_eq!(matched.fragment(), &"foo bar\tbaz");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_invalid_start() {
        let input = Span::new("-foo");
        let res = label(input);
        assert!(res.is_err(), "should not parse label starting with dash");
    }

    #[test]
    fn test_label_only_underscore() {
        let input = Span::new("_: rest");
        let (rest, matched) = label(input).expect("should parse label with only underscore");
        assert_eq!(matched.fragment(), &"_");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_multiple_dashes() {
        let input = Span::new("foo-bar-baz: rest");
        let (rest, matched) = label(input).expect("should parse label with multiple dashes");
        assert_eq!(matched.fragment(), &"foo-bar-baz");
        assert_eq!(rest.fragment(), &": rest");
    }

    #[test]
    fn test_label_with_trailing_whitespace() {
        let input = Span::new("foo : rest");
        let (rest, matched) = label(input).expect("should parse label with trailing whitespace");
        assert_eq!(matched.fragment(), &"foo ");
        assert_eq!(rest.fragment(), &": rest");
    }
}
