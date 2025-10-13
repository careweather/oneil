use nom::{Parser as NomParser, character::complete::space0, combinator::recognize};
use oneil_ast::SpanLike;

use crate::token::{
    InputSpan, Parser, Result,
    error::{ErrorHandlingParser, TokenError},
};

/// A token representing a lexical element in Oneil source code.
///
/// A token consists of two parts:
/// - **Lexeme**: The actual text content of the token (e.g., "if", "123", "+")
/// - **Whitespace**: Any trailing whitespace that follows the lexeme
///
/// This structure allows the parser to maintain precise location information
/// while handling whitespace appropriately during tokenization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    lexeme: InputSpan<'a>,
    whitespace: InputSpan<'a>,
}

impl<'a> Token<'a> {
    /// Creates a new token with the specified lexeme and whitespace spans.
    #[must_use]
    pub const fn new(lexeme: InputSpan<'a>, whitespace: InputSpan<'a>) -> Self {
        Self { lexeme, whitespace }
    }

    /// Returns the text content of the token.
    pub fn lexeme(&self) -> &str {
        self.lexeme.fragment()
    }

    /// Returns the trailing whitespace that follows the token.
    #[cfg(test)]
    pub fn whitespace(&self) -> &str {
        self.whitespace.fragment()
    }
}

impl SpanLike for Token<'_> {
    /// Returns the starting offset of the token's lexeme.
    fn get_start(&self) -> usize {
        self.lexeme.location_offset()
    }

    /// Returns the ending offset of the token's lexeme.
    fn get_length(&self) -> usize {
        self.lexeme.fragment().len()
    }

    /// Returns the ending offset including trailing whitespace.
    fn get_whitespace_length(&self) -> usize {
        self.whitespace.fragment().len()
    }
}

/// Parses inline whitespace (spaces and tabs) and returns the parsed whitespace.
///
/// This function consumes any amount of whitespace (including none) and always succeeds.
/// It's useful for handling optional whitespace between tokens.
pub fn inline_whitespace(input: InputSpan<'_>) -> Result<'_, InputSpan<'_>, TokenError> {
    space0.parse(input)
}

/// Wraps a parser to handle trailing whitespace after the matched content.
///
/// This function takes a parser `f` and returns a new parser that:
/// 1. Recognizes the content matched by `f`
/// 2. Consumes any trailing whitespace after the match
/// 3. Returns the matched content as a `Token` with lexeme and whitespace spans
///
/// This is useful for tokenizing where whitespace between tokens should be handled automatically.
pub fn token<'a, O>(
    f: impl Parser<'a, O, TokenError>,
    convert_error: impl Fn(TokenError) -> TokenError,
) -> impl Parser<'a, Token<'a>, TokenError> {
    (
        recognize(f).convert_error_to(convert_error),
        inline_whitespace,
    )
        .map(|(lexeme, whitespace)| Token::new(lexeme, whitespace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use nom::bytes::complete::tag;

    #[test]
    fn inline_whitespace_spaces() {
        let input = InputSpan::new_extra("   abc", Config::default());
        let (rest, _) = inline_whitespace(input).expect("should parse leading spaces");
        assert_eq!(rest.fragment(), &"abc");
    }

    #[test]
    fn inline_whitespace_tabs() {
        let input = InputSpan::new_extra("\t\tfoo", Config::default());
        let (rest, _) = inline_whitespace(input).expect("should parse leading tabs");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn inline_whitespace_none() {
        let input = InputSpan::new_extra("bar", Config::default());
        let (rest, _) = inline_whitespace(input).expect("should parse no whitespace");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn token_with_whitespace() {
        let mut parser = token(tag("foo"), TokenError::expected_identifier);
        let input = InputSpan::new_extra("foo   bar", Config::default());
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse token with trailing whitespace");
        assert_eq!(matched.lexeme(), "foo");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    #[expect(
        clippy::assertions_on_result_states,
        reason = "we don't care about the result, just that it's an error"
    )]
    fn token_no_match() {
        let mut parser = token(tag("baz"), TokenError::expected_identifier);
        let input = InputSpan::new_extra("foo   bar", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err());
    }
}
