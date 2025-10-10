use nom::{IResult, Parser as NomParser, error::Error};
use nom_locate::LocatedSpan;
use oneil_shared::span::{SourceLocation, Span};

use super::config::Config;

/// A span of text in the input string.
///
/// The span contains both the text content and the configuration for the parser.
/// This type is used throughout the parser to track source locations and parser configuration.
pub type InputSpan<'a> = LocatedSpan<&'a str, Config>;

pub fn source_location_from(input_span: InputSpan<'_>) -> SourceLocation {
    SourceLocation {
        offset: input_span.location_offset(),
        line: usize::try_from(input_span.location_line())
            .expect("usize should be greater than or equal to u32"),
        column: input_span.get_column(),
    }
}

pub fn span_from(start_input_span: InputSpan<'_>, end_input_span: InputSpan<'_>) -> Span {
    Span::new(
        source_location_from(start_input_span),
        source_location_from(end_input_span),
    )
}

/// A result type for parser operations.
///
/// This type alias provides a consistent result type for all parser functions,
/// wrapping nom's `IResult` with our custom Span type.
pub type Result<'a, O, E = Error<InputSpan<'a>>> = IResult<InputSpan<'a>, O, E>;

/// A trait for parser implementations that work with our custom Span type.
///
/// This trait is automatically implemented for any type that implements nom's Parser trait
/// with our custom Span type. It serves as a convenience wrapper to ensure consistent
/// parser implementations throughout the codebase.
pub trait Parser<'a, O, E = Error<InputSpan<'a>>>:
    NomParser<InputSpan<'a>, Output = O, Error = E>
{
}

impl<'a, O, E, P> Parser<'a, O, E> for P where P: NomParser<InputSpan<'a>, Output = O, Error = E> {}

#[cfg(test)]
pub mod test {
    macro_rules! assert_node_contains {
        ($node:expr, $value:expr, start_offset: $start_offset:expr, end_offset: $end_offset:expr) => {
            use oneil_ast::Node;

            let node: &Node<_> = $node;

            assert_eq!(**node, $value);
            assert_eq!(node.span().start().offset, $start_offset);
            assert_eq!(node.span().end().offset, $end_offset);
        };
    }

    pub(crate) use assert_node_contains;
}
