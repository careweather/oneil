//! Provides parsers for Oneil language keywords.
//!
//! This module contains parsers for all reserved keywords in the Oneil language.
//! Each keyword parser ensures that the keyword is followed by a non-alphanumeric
//! character or '_' to prevent partial matches (e.g., "and" vs "android").
//!
//! # Examples
//!
//! ```
//! use oneil::parser::token::keyword::{and, if_};
//! use oneil::parser::{Config, Span};
//!
//! let input = Span::new_extra("and true", Config::default());
//! let (rest, matched) = and(input).unwrap();
//! assert_eq!(matched.fragment(), &"and");
//!
//! let input = Span::new_extra("if x > 0", Config::default());
//! let (rest, matched) = if_(input).unwrap();
//! assert_eq!(matched.fragment(), &"if");
//! ```

use nom::{
    Parser as _,
    bytes::complete::tag,
    character::complete::satisfy,
    combinator::{eof, peek},
};

use crate::parser::token::error;

use super::{Parser, Result, Span, util::token};

/// Keywords that are valid in the Oneil language.
pub const KEYWORDS: &[&str] = &[
    "and", "as", "false", "from", "if", "import", "not", "or", "true", "section", "test", "use",
];

fn keyword(kw_str: &str, error_kind: error::ExpectKeyword) -> impl Parser<Span, error::TokenError> {
    let next_char_is_not_ident_char =
        peek(satisfy(|c: char| !c.is_alphanumeric() && c != '_')).map(|_| ());
    let reached_end_of_file = eof.map(|_| ());
    error::convert_err(
        token((
            tag(kw_str),
            next_char_is_not_ident_char.or(reached_end_of_file),
        )),
        error::TokenErrorKind::Keyword(error_kind),
    )
}

/// Parses the 'and' keyword token.
pub fn and(input: Span) -> Result<Span, error::TokenError> {
    keyword("and", error::ExpectKeyword::And).parse(input)
}

/// Parses the 'as' keyword token.
pub fn as_(input: Span) -> Result<Span, error::TokenError> {
    keyword("as", error::ExpectKeyword::As).parse(input)
}

/// Parses the 'false' keyword token.
pub fn false_(input: Span) -> Result<Span, error::TokenError> {
    keyword("false", error::ExpectKeyword::False).parse(input)
}

/// Parses the 'from' keyword token.
pub fn from(input: Span) -> Result<Span, error::TokenError> {
    keyword("from", error::ExpectKeyword::From).parse(input)
}

/// Parses the 'if' keyword token.
pub fn if_(input: Span) -> Result<Span, error::TokenError> {
    keyword("if", error::ExpectKeyword::If).parse(input)
}

/// Parses the 'import' keyword token.
pub fn import(input: Span) -> Result<Span, error::TokenError> {
    keyword("import", error::ExpectKeyword::Import).parse(input)
}

/// Parses the 'not' keyword token.
pub fn not(input: Span) -> Result<Span, error::TokenError> {
    keyword("not", error::ExpectKeyword::Not).parse(input)
}

/// Parses the 'or' keyword token.
pub fn or(input: Span) -> Result<Span, error::TokenError> {
    keyword("or", error::ExpectKeyword::Or).parse(input)
}

/// Parses the 'true' keyword token.
pub fn true_(input: Span) -> Result<Span, error::TokenError> {
    keyword("true", error::ExpectKeyword::True).parse(input)
}

/// Parses the 'section' keyword token.
pub fn section(input: Span) -> Result<Span, error::TokenError> {
    keyword("section", error::ExpectKeyword::Section).parse(input)
}

/// Parses the 'test' keyword token.
pub fn test(input: Span) -> Result<Span, error::TokenError> {
    keyword("test", error::ExpectKeyword::Test).parse(input)
}

/// Parses the 'use' keyword token.
pub fn use_(input: Span) -> Result<Span, error::TokenError> {
    keyword("use", error::ExpectKeyword::Use).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{config::Config, token::Span};

    #[test]
    fn test_and() {
        let input = Span::new_extra("and rest", Config::default());
        let (rest, matched) = and(input).expect("should parse 'and' keyword");
        assert_eq!(matched.fragment(), &"and");
        assert_eq!(rest.fragment(), &"rest");
    }

    #[test]
    fn test_as() {
        let input = Span::new_extra("as foo", Config::default());
        let (rest, matched) = as_(input).expect("should parse 'as' keyword");
        assert_eq!(matched.fragment(), &"as");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn test_false() {
        let input = Span::new_extra("false true", Config::default());
        let (rest, matched) = false_(input).expect("should parse 'false' keyword");
        assert_eq!(matched.fragment(), &"false");
        assert_eq!(rest.fragment(), &"true");
    }

    #[test]
    fn test_from() {
        let input = Span::new_extra("from bar", Config::default());
        let (rest, matched) = from(input).expect("should parse 'from' keyword");
        assert_eq!(matched.fragment(), &"from");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_if() {
        let input = Span::new_extra("if baz", Config::default());
        let (rest, matched) = if_(input).expect("should parse 'if' keyword");
        assert_eq!(matched.fragment(), &"if");
        assert_eq!(rest.fragment(), &"baz");
    }

    #[test]
    fn test_import() {
        let input = Span::new_extra("import foo", Config::default());
        let (rest, matched) = import(input).expect("should parse 'import' keyword");
        assert_eq!(matched.fragment(), &"import");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn test_not() {
        let input = Span::new_extra("not bar", Config::default());
        let (rest, matched) = not(input).expect("should parse 'not' keyword");
        assert_eq!(matched.fragment(), &"not");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_or() {
        let input = Span::new_extra("or baz", Config::default());
        let (rest, matched) = or(input).expect("should parse 'or' keyword");
        assert_eq!(matched.fragment(), &"or");
        assert_eq!(rest.fragment(), &"baz");
    }

    #[test]
    fn test_true() {
        let input = Span::new_extra("true false", Config::default());
        let (rest, matched) = true_(input).expect("should parse 'true' keyword");
        assert_eq!(matched.fragment(), &"true");
        assert_eq!(rest.fragment(), &"false");
    }

    #[test]
    fn test_section() {
        let input = Span::new_extra("section test", Config::default());
        let (rest, matched) = section(input).expect("should parse 'section' keyword");
        assert_eq!(matched.fragment(), &"section");
        assert_eq!(rest.fragment(), &"test");
    }

    #[test]
    fn test_test() {
        let input = Span::new_extra("test use", Config::default());
        let (rest, matched) = test(input).expect("should parse 'test' keyword");
        assert_eq!(matched.fragment(), &"test");
        assert_eq!(rest.fragment(), &"use");
    }

    #[test]
    fn test_use() {
        let input = Span::new_extra("use foo", Config::default());
        let (rest, matched) = use_(input).expect("should parse 'use' keyword");
        assert_eq!(matched.fragment(), &"use");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn test_keyword_with_trailing_whitespace() {
        let input = Span::new_extra("and   foo", Config::default());
        let (rest, matched) = and(input).expect("should parse 'and' with trailing whitespace");
        assert_eq!(matched.fragment(), &"and");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn test_keyword_not_at_start() {
        let mut parser = keyword("and", error::ExpectKeyword::And);
        let input = Span::new_extra("foo and bar", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err(), "should not parse 'and' if not at start");
    }

    #[test]
    fn test_keyword_prefix() {
        let mut parser = keyword("and", error::ExpectKeyword::And);
        let input = Span::new_extra("andrew", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err(), "should not parse 'and' as prefix");
    }

    #[test]
    fn test_keyword_matches_at_end_of_file() {
        let mut parser = keyword("and", error::ExpectKeyword::And);
        let input = Span::new_extra("and", Config::default());
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse 'and' at end of file");
        assert_eq!(matched.fragment(), &"and");
        assert_eq!(rest.fragment(), &"");
    }
}
