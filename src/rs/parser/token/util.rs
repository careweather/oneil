use nom::{
    Parser as _,
    character::complete::space0,
    combinator::{recognize, value},
    sequence::terminated,
};

use super::{
    Parser, Result, Span,
    error::{self, TokenError, TokenErrorKind},
};
use crate::parser::error::ErrorHandlingParser as _;

/// Parses inline whitespace (spaces and tabs) and returns unit `()`.
///
/// This function consumes any amount of whitespace (including none) and always succeeds.
/// It's useful for handling optional whitespace between tokens.
pub fn inline_whitespace(input: Span) -> Result<(), TokenError> {
    // Needed for type inference
    let space0 = space0::<_, nom::error::Error<Span>>;

    value((), space0).errors_into().parse(input)
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
    error_kind: TokenErrorKind,
) -> impl Parser<'a, Span<'a>, TokenError<'a>> {
    terminated(recognize(f), inline_whitespace)
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
        // Needed for type inference
        let tag = tag::<_, _, nom::error::Error<Span>>;

        let mut parser = token(tag("foo").errors_into(), TokenErrorKind::ExpectIdentifier);
        let input = Span::new_extra("foo   bar", Config::default());
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse token with trailing whitespace");
        assert_eq!(matched.fragment(), &"foo");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_no_match() {
        // Needed for type inference
        let tag = tag::<_, _, nom::error::Error<Span>>;

        let mut parser = token(tag("baz").errors_into(), TokenErrorKind::ExpectIdentifier);
        let input = Span::new_extra("foo   bar", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err());
    }
}
