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

/// Parses inline whitespace (spaces and tabs) and returns unit `()`.
///
/// This function consumes any amount of whitespace (including none) and always succeeds.
/// It's useful for handling optional whitespace between tokens.
pub fn inline_whitespace(input: Span) -> Result<(), TokenError> {
    value((), space0).map_err(error::from_nom).parse(input)
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
    let mut token_parser = terminated(recognize(f), inline_whitespace);
    move |input| {
        token_parser.parse(input).map_err(|e| match e {
            // Note that we only use the error kind for the error case
            // because any failures *should* be handled where the failures occur
            nom::Err::Error(e) => nom::Err::Error(error::convert_to_kind(error_kind)(e)),
            nom::Err::Failure(e) => nom::Err::Failure(e),
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Config;

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
        use nom::bytes::complete::tag;
        let mut parser = token(
            tag("foo").map_err(error::from_nom),
            TokenErrorKind::ExpectIdentifier,
        );
        let input = Span::new_extra("foo   bar", Config::default());
        let (rest, matched) = parser
            .parse(input)
            .expect("should parse token with trailing whitespace");
        assert_eq!(matched.fragment(), &"foo");
        assert_eq!(rest.fragment(), &"bar");
    }

    #[test]
    fn test_token_no_match() {
        use nom::bytes::complete::tag;
        let mut parser = token(
            tag("baz").map_err(error::from_nom),
            TokenErrorKind::ExpectIdentifier,
        );
        let input = Span::new_extra("foo   bar", Config::default());
        let res = parser.parse(input);
        assert!(res.is_err());
    }
}
