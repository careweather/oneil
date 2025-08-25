use nom::{Parser as NomParser, character::complete::space0, combinator::recognize};
use oneil_ast::span::SpanLike;

use crate::token::{
    Parser, Result, Span,
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
    lexeme: Span<'a>,
    whitespace: Span<'a>,
}

impl<'a> Token<'a> {
    /// Creates a new token with the specified lexeme and whitespace spans.
    ///
    /// # Arguments
    ///
    /// * `lexeme` - The span containing the token's text content
    /// * `whitespace` - The span containing trailing whitespace
    ///
    /// # Returns
    ///
    /// A new `Token` instance.
    pub fn new(lexeme: Span<'a>, whitespace: Span<'a>) -> Self {
        Self { lexeme, whitespace }
    }

    /// Returns the text content of the token.
    ///
    /// This is the actual lexical content that was matched by the parser,
    /// such as keywords, identifiers, literals, or operators.
    ///
    /// # Returns
    ///
    /// A string slice containing the token's text content.
    pub fn lexeme(&self) -> &str {
        self.lexeme.fragment()
    }

    /// Returns the trailing whitespace that follows the token.
    ///
    /// This method is primarily used for testing and debugging purposes.
    /// The whitespace information is preserved for precise error reporting
    /// and location tracking.
    ///
    /// # Returns
    ///
    /// A string slice containing the trailing whitespace.
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
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns `Ok((remaining_input, parsed_whitespace))` on success, where the
/// remaining input excludes the consumed whitespace.
pub fn inline_whitespace(input: Span<'_>) -> Result<'_, Span<'_>, TokenError> {
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
///
/// # Arguments
///
/// * `f` - The parser to wrap
/// * `convert_error` - A function to convert parsing errors to `TokenError`
///
/// # Returns
///
/// A new parser that returns a `Token` containing both the matched content
/// and any trailing whitespace.
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
    fn test_inline_whitespace_spaces() {
        let input = Span::new_extra("   abc", Config::default());
        let (rest, _) = inline_whitespace(input).expect("should parse leading spaces");
        assert_eq!(rest.fragment(), &"abc");
    }

    #[test]
    fn test_inline_whitespace_tabs() {
        let input = Span::new_extra("\t\tfoo", Config::default());
        let (rest, _) = inline_whitespace(input).expect("should parse leading tabs");
        assert_eq!(rest.fragment(), &"foo");
    }

    #[test]
    fn test_inline_whitespace_none() {
        let input = Span::new_extra("bar", Config::default());
        let (rest, _) = inline_whitespace(input).expect("should parse no whitespace");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_with_whitespace() {
        let mut parser = token(tag("foo"), TokenError::expected_identifier);
        let input = Span::new_extra("foo   bar", Config::default());
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse token with trailing whitespace");
        assert_eq!(matched.lexeme(), "foo");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_no_match() {
        let mut parser = token(tag("baz"), TokenError::expected_identifier);
        let input = Span::new_extra("foo   bar", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err());
    }
}
