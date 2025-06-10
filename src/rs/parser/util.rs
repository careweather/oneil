use nom::{IResult, Parser as NomParser, error::Error};
use nom_locate::LocatedSpan;

use super::config::Config;

/// A span of text in the input string
///
/// The span also contains the configuration for the parser.
pub type Span<'a> = LocatedSpan<&'a str, Config>;

/// A result of a parser
///
/// Currently just an alias for `IResult<Span<'a>, O, Error<Span<'a>>>`, but it
/// may be updated in the future
pub type Result<'a, O, E = Error<Span<'a>>> = IResult<Span<'a>, O, E>;

pub trait Parser<'a, O, E = Error<Span<'a>>>: NomParser<Span<'a>, Output = O, Error = E> {
    fn map_err<E2>(mut self, f: impl Fn(E) -> E2) -> impl Parser<'a, O, E2>
    where
        Self: Sized,
        E2: nom::error::ParseError<Span<'a>>,
    {
        move |input| {
            self.parse(input).map_err(|e| match e {
                nom::Err::Error(e) => nom::Err::Error(f(e)),
                nom::Err::Failure(e) => nom::Err::Failure(f(e)),
                nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            })
        }
    }
}

impl<'a, O, E, P> Parser<'a, O, E> for P where P: NomParser<Span<'a>, Output = O, Error = E> {}
