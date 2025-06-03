use nom::{character::streaming::space0, combinator::value, sequence::terminated, Parser as NomParser};

use super::util::{Span, Result, Parser};

pub fn inline_whitespace<'a>(input: Span<'a>) -> Result<'a, ()> {
    value((), space0).parse(input)
}

pub fn token<'a, F, O>(f: F) -> impl Parser<'a, O>
where
    F: Parser<'a, O>,
{
    terminated(f, inline_whitespace)
}
