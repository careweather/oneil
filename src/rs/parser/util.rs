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

pub trait Parser<'a, O, E = Error<Span<'a>>>: NomParser<Span<'a>, Output = O, Error = E> {}

impl<'a, O, E, P> Parser<'a, O, E> for P where P: NomParser<Span<'a>, Output = O, Error = E> {}
