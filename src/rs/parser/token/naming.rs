//! Provides parsers for identifiers and labels in the Oneil language.
//!
//! This module contains parsers for identifiers (variable names, function names, etc.)
//! and labels (section names, test names, etc.). Identifiers follow standard programming
//! language rules, while labels are more permissive to allow for descriptive names.
//!
//! # Examples
//!
//! ```
//! use oneil::parser::token::naming::{identifier, label};
//! use oneil::parser::util::Span;
//!
//! // Parse an identifier
//! let input = Span::new("my_var123 rest");
//! let (rest, matched) = identifier(input).unwrap();
//! assert_eq!(matched.fragment(), &"my_var123");
//!
//! // Parse a label
//! let input = Span::new("My Test Case: rest");
//! let (rest, matched) = label(input).unwrap();
//! assert_eq!(matched.fragment(), &"My Test Case");
//! ```

use nom::{
    Parser as _, bytes::complete::take_while, character::complete::satisfy, combinator::verify,
};

use super::{Result, Span, keyword, util::token};

/// Parses an identifier (alphabetic or underscore, then alphanumeric or underscore).
///
/// # Examples
///
/// ```
/// use oneil::parser::token::naming::identifier;
/// use oneil::parser::util::Span;
///
/// // Basic identifier
/// let input = Span::new("variable rest");
/// let (rest, matched) = identifier(input).unwrap();
/// assert_eq!(matched.fragment(), &"variable");
/// assert_eq!(rest.fragment(), &"rest");
///
/// // Identifier with underscore and numbers
/// let input = Span::new("my_var_123 rest");
/// let (rest, matched) = identifier(input).unwrap();
/// assert_eq!(matched.fragment(), &"my_var_123");
/// assert_eq!(rest.fragment(), &"rest");
///
/// // Identifier starting with underscore
/// let input = Span::new("_private rest");
/// let (rest, matched) = identifier(input).unwrap();
/// assert_eq!(matched.fragment(), &"_private");
/// assert_eq!(rest.fragment(), &"rest");
///
/// // Invalid identifier (starting with number) should fail
/// let input = Span::new("123invalid");
/// assert!(identifier(input).is_err());
/// ```
pub fn identifier(input: Span) -> Result<Span> {
    verify(
        token((
            satisfy(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )),
        |identifier| !keyword::KEYWORDS.contains(&identifier.fragment()),
    )
    .parse(input)
}

/// Parses a label (alphabetic or underscore, then alphanumeric, underscore, dash, space, or tab).
/// # Examples
///
/// Note that labels are often followed by a colon as a delimiter, but other
/// tokens (such as a linebreak) can also be used.
///
/// ```
/// use oneil::parser::token::naming::label;
/// use oneil::parser::util::Span;
///
/// // Basic label
/// let input = Span::new("Test Case: rest");
/// let (rest, matched) = label(input).unwrap();
/// assert_eq!(matched.fragment(), &"Test Case");
/// assert_eq!(rest.fragment(), &": rest");
///
/// // Label with mixed characters
/// let input = Span::new("My_Test-Case: rest");
/// let (rest, matched) = label(input).unwrap();
/// assert_eq!(matched.fragment(), &"My_Test-Case");
/// assert_eq!(rest.fragment(), &": rest");
///
/// // Label with tabs
/// let input = Span::new("Test\tCase: rest");
/// let (rest, matched) = label(input).unwrap();
/// assert_eq!(matched.fragment(), &"Test\tCase");
/// assert_eq!(rest.fragment(), &": rest");
///
/// // Invalid label (starting with number) should fail
/// let input = Span::new("123Test");
/// assert!(label(input).is_err());
/// ```
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
