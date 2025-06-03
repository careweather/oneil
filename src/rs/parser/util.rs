use nom::{IResult, error::Error, Parser as NomParser};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Result<'a, O> = IResult<Span<'a>, O, Error<Span<'a>>>;

pub trait Parser<'a, O>: NomParser<Span<'a>, Output = O, Error = Error<Span<'a>>> {}

impl<'a, O, P> Parser<'a, O> for P where P: NomParser<Span<'a>, Output = O, Error = Error<Span<'a>>> {}