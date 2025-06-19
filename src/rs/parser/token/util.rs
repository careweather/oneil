use nom::{Parser as _, character::complete::space0, combinator::recognize};

use super::{
    Parser, Result, Span,
    error::{self, TokenError, TokenErrorKind},
};
use crate::parser::error::ErrorHandlingParser as _;

pub struct Token<'a> {
    lexeme: Span<'a>,
    whitespace: Span<'a>,
}

impl<'a> Token<'a> {
    pub fn new(lexeme: Span<'a>, whitespace: Span<'a>) -> Self {
        Self { lexeme, whitespace }
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme.fragment()
    }

    pub fn lexeme_offset(&self) -> usize {
        self.lexeme.location_offset()
    }

    pub fn lexeme_line(&self) -> u32 {
        self.lexeme.location_line()
    }

    pub fn lexeme_column(&self) -> usize {
        self.lexeme.get_column()
    }

    pub fn get_lexeme_end_offset(&self) -> usize {
        let start = self.lexeme.location_offset();
        let length = self.lexeme.fragment().len();
        start + length
    }

    pub fn whitespace(&self) -> &str {
        self.whitespace.fragment()
    }

    pub fn whitespace_offset(&self) -> usize {
        self.whitespace.location_offset()
    }

    pub fn get_whitespace_end_offset(&self) -> usize {
        let start = self.whitespace.location_offset();
        let length = self.whitespace.fragment().len();
        start + length
    }
}

/// Parses inline whitespace (spaces and tabs) and returns unit `()`.
///
/// This function consumes any amount of whitespace (including none) and always succeeds.
/// It's useful for handling optional whitespace between tokens.
pub fn inline_whitespace(input: Span) -> Result<Span, TokenError> {
    space0.parse(input)
}

/// Wraps a parser to handle trailing whitespace after the matched content.
///
/// This function takes a parser `f` and returns a new parser that:
/// 1. Recognizes the content matched by `f`
/// 2. Consumes any trailing whitespace after the match
/// 3. Returns the matched content as a `Span`
///
/// This is useful for tokenizing where whitespace between tokens should be handled automatically.
pub fn token<'a, O>(
    f: impl Parser<'a, O, TokenError<'a>>,
    error_kind: TokenErrorKind<'a>,
) -> impl Parser<'a, Token<'a>, TokenError<'a>> {
    (recognize(f), inline_whitespace)
        .map(|(lexeme, whitespace)| Token::new(lexeme, whitespace))
        .map_error(move |e| error::TokenError::new(error_kind, e.span))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Config;
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
        let mut parser = token(tag("foo"), TokenErrorKind::ExpectIdentifier);
        let input = Span::new_extra("foo   bar", Config::default());
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse token with trailing whitespace");
        assert_eq!(matched.lexeme(), "foo");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_no_match() {
        let mut parser = token(tag("baz"), TokenErrorKind::ExpectIdentifier);
        let input = Span::new_extra("foo   bar", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err());
    }
}
