use nom::{
    Parser,
    error::{FromExternalError, ParseError},
};

use super::{
    Span,
    token::error::{TokenError, TokenErrorKind},
};

pub trait ErrorHandlingParser<I, O, E>: Parser<I, Output = O, Error = E>
where
    E: nom::error::ParseError<I>,
{
    fn map_error<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(e.into()),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    fn map_failure<E2>(
        mut self,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(e.into()),
                nom::Err::Failure(e) => nom::Err::Failure(convert_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    fn map_error_and_failure<E2>(
        mut self,
        convert_error: impl Fn(E) -> E2,
        convert_failure: impl Fn(E) -> E2,
    ) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(convert_error(e)),
                nom::Err::Failure(e) => nom::Err::Failure(convert_failure(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }

    fn errors_into<E2>(self) -> impl Parser<I, Output = O, Error = E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<I> + From<E>,
    {
        self.map_error_and_failure(|e| e.into(), |e| e.into())
    }
}

impl<'a, I, O, E, P> ErrorHandlingParser<I, O, E> for P
where
    P: Parser<I, Output = O, Error = E>,
    E: nom::error::ParseError<I>,
{
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParserError<'a> {
    pub kind: ParserErrorKind<'a>,
    pub span: Span<'a>,
}

impl<'a> ParserError<'a> {
    pub fn new(kind: ParserErrorKind<'a>, span: Span<'a>) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParserErrorKind<'a> {
    ExpectDecl,
    ExpectExpr,
    ExpectNote,
    ExpectParameter,
    ExpectTest,
    ExpectUnit,
    InvalidNumber(&'a str),
    TokenError(TokenErrorKind),
    NomError(nom::error::ErrorKind),
}

impl<'a> ParseError<Span<'a>> for ParserError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            kind: ParserErrorKind::NomError(kind),
            span: input,
        }
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> FromExternalError<Span<'a>, ParserErrorKind<'a>> for ParserError<'a> {
    fn from_external_error(
        input: Span<'a>,
        _kind: nom::error::ErrorKind,
        e: ParserErrorKind<'a>,
    ) -> Self {
        Self::new(e, input)
    }
}

impl<'a> From<TokenError<'a>> for ParserError<'a> {
    fn from(e: TokenError<'a>) -> Self {
        Self {
            kind: ParserErrorKind::TokenError(e.kind),
            span: e.span,
        }
    }
}

impl<'a> From<nom::error::Error<Span<'a>>> for ParserError<'a> {
    fn from(e: nom::error::Error<Span<'a>>) -> Self {
        Self {
            kind: ParserErrorKind::NomError(e.code),
            span: e.input,
        }
    }
}
