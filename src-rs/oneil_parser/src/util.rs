use nom::{IResult, Parser as NomParser, error::Error};
use nom_locate::LocatedSpan;

use super::config::Config;

/// A span of text in the input string.
///
/// The span contains both the text content and the configuration for the parser.
/// This type is used throughout the parser to track source locations and parser configuration.
pub type Span<'a> = LocatedSpan<&'a str, Config>;

/// A result type for parser operations.
///
/// This type alias provides a consistent result type for all parser functions,
/// wrapping nom's IResult with our custom Span type.
pub type Result<'a, O, E = Error<Span<'a>>> = IResult<Span<'a>, O, E>;

/// A trait for parser implementations that work with our custom Span type.
///
/// This trait is automatically implemented for any type that implements nom's Parser trait
/// with our custom Span type. It serves as a convenience wrapper to ensure consistent
/// parser implementations throughout the codebase.
pub trait Parser<'a, O, E = Error<Span<'a>>>: NomParser<Span<'a>, Output = O, Error = E> {}

impl<'a, O, E, P> Parser<'a, O, E> for P where P: NomParser<Span<'a>, Output = O, Error = E> {}
