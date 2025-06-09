use nom::{Err, error::ParseError};

use super::{Parser, Span};

pub fn convert_err<'a>(
    mut parser: impl Parser<'a, Span<'a>>,
    kind: TokenErrorKind,
) -> impl Parser<'a, Span<'a>, TokenError<'a>> {
    move |input| {
        parser.parse(input).map_err(|e| match e {
            Err::Error(e) => Err::Error(TokenError::new(kind, e.input)),
            Err::Failure(e) => Err::Failure(TokenError::new(kind, e.input)),
            Err::Incomplete(e) => Err::Incomplete(e),
        })
    }
}

pub fn map_err<'a>(
    mut parser: impl Parser<'a, Span<'a>>,
    f: impl Fn(&nom::error::Error<Span<'a>>) -> TokenErrorKind,
) -> impl Parser<'a, Span<'a>, TokenError<'a>> {
    move |input| {
        parser.parse(input).map_err(|e| match e {
            Err::Error(e) => Err::Error(TokenError::new(f(&e), e.input)),
            Err::Failure(e) => Err::Failure(TokenError::new(f(&e), e.input)),
            Err::Incomplete(e) => Err::Incomplete(e),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokenError<'a> {
    kind: TokenErrorKind,
    span: Span<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenErrorKind {
    Keyword(ExpectKeyword),
    NomError(nom::error::ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpectKeyword {
    And,
    As,
    False,
    From,
    If,
    Import,
    Not,
    Or,
    Test,
    True,
    Section,
    Use,
}

impl<'a> TokenError<'a> {
    fn new(kind: TokenErrorKind, span: Span<'a>) -> Self {
        Self { kind, span }
    }
}

impl<'a> ParseError<Span<'a>> for TokenError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: TokenErrorKind::NomError(kind),
            span: input,
        }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> From<nom::error::Error<Span<'a>>> for TokenError<'a> {
    fn from(e: nom::error::Error<Span<'a>>) -> Self {
        Self::from_error_kind(e.input, e.code)
    }
}
