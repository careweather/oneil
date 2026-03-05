//! Partial result error handling for the Oneil parser

#[cfg(test)]
use std::ops;

use nom::error::ParseError;
use oneil_ast::{Model, ModelNode};
use oneil_shared::{
    partial::PartialError,
    span::{SourceLocation, Span},
};

use crate::{error::ParserError, util::InputSpan};

/// A wrapper around the shared partial error type that can be used as a parser error
#[derive(Debug, Clone)]
pub(crate) struct ParserPartialModelError(PartialError<ModelNode, Vec<ParserError>>);

impl ParserPartialModelError {
    /// Creates a new `ParserPartialModelError` with the given partial result
    /// and errors
    #[must_use]
    pub const fn new(partial_result: ModelNode, error_collection: Vec<ParserError>) -> Self {
        Self(PartialError::new(partial_result, error_collection))
    }
}

impl ParseError<InputSpan<'_>> for ParserPartialModelError {
    fn from_error_kind(input: InputSpan<'_>, kind: nom::error::ErrorKind) -> Self {
        let offset = input.location_offset();
        let line = usize::try_from(input.location_line())
            .expect("usize should be greater than or equal to u32");
        let column = input.get_column();

        let source_location = SourceLocation {
            offset,
            line,
            column,
        };

        let span = Span::empty(source_location);

        let model_node = ModelNode::new(Model::empty(), span, span);

        Self(PartialError::new(
            model_node,
            vec![ParserError::from_error_kind(input, kind)],
        ))
    }

    fn append(_input: InputSpan<'_>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl From<ParserPartialModelError> for PartialError<ModelNode, Vec<ParserError>> {
    fn from(partial_error: ParserPartialModelError) -> Self {
        partial_error.0
    }
}

#[cfg(test)]
impl ops::Deref for ParserPartialModelError {
    type Target = PartialError<ModelNode, Vec<ParserError>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
