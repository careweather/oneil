use nom::{Parser as NomParser, character::complete::space0, combinator::recognize};
use oneil_shared::span::{SourceLocation, Span};

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
    pub lexeme_str: &'a str,
    pub lexeme_span: Span,
    pub whitespace_span: Span,
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
    mut f: impl Parser<'a, O, TokenError>,
    convert_error: impl Fn(TokenError) -> TokenError,
) -> impl Parser<'a, Token<'a>, TokenError> {
    move |input| {
        // capture the parser and convert the error function
        let f = |input| (&mut f).parse(input);
        let convert_error = |error| (&convert_error)(error);

        // recognize the lexeme
        let (rest, lexeme) = recognize(f).convert_error_to(convert_error).parse(input)?;

        let lexeme_str = lexeme.fragment();

        let lexeme_start_line = usize::try_from(lexeme.location_line())
            .expect("usize should be greater than or equal to u32");
        let lexeme_start = SourceLocation {
            offset: lexeme.location_offset(),
            line: lexeme_start_line,
            column: lexeme.get_column(),
        };

        let lexeme_end_line = usize::try_from(rest.location_line())
            .expect("usize should be greater than or equal to u32");
        let lexeme_end = SourceLocation {
            offset: rest.location_offset(),
            line: lexeme_end_line,
            column: rest.get_column(),
        };

        let lexeme_span = Span::new(lexeme_start, lexeme_end);

        // consume the whitespace
        let (rest, whitespace) = inline_whitespace.parse(rest)?;

        let whitespace_start_line = usize::try_from(whitespace.location_line())
            .expect("usize should be greater than or equal to u32");
        let whitespace_start = SourceLocation {
            offset: whitespace.location_offset(),
            line: whitespace_start_line,
            column: whitespace.get_column(),
        };

        let whitespace_end_line = usize::try_from(rest.location_line())
            .expect("usize should be greater than or equal to u32");
        let whitespace_end = SourceLocation {
            offset: rest.location_offset(),
            line: whitespace_end_line,
            column: rest.get_column(),
        };

        let whitespace_span = Span::new(whitespace_start, whitespace_end);

        let token = Token {
            lexeme_str,
            lexeme_span,
            whitespace_span,
        };

        Ok((rest, token))
    }
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
        assert_eq!(matched.lexeme_str, "foo");
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
