use nom::{IResult, Parser as NomParser, error::Error};
use nom_locate::LocatedSpan;

/// A span of text in the input string
///
/// Currently just an alias for `LocatedSpan<&str>`, but it may be updated in
/// the future
pub type Span<'a> = LocatedSpan<&'a str>;

/// A result of a parser
///
/// Currently just an alias for `IResult<Span<'a>, O, Error<Span<'a>>>`, but it
/// may be updated in the future
pub type Result<'a, O> = IResult<Span<'a>, O, Error<Span<'a>>>;

pub trait Parser<'a, O>: NomParser<Span<'a>, Output = O, Error = Error<Span<'a>>> {}

impl<'a, O, P> Parser<'a, O> for P where P: NomParser<Span<'a>, Output = O, Error = Error<Span<'a>>> {}
