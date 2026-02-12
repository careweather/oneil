//! Partial result error handling for the Oneil parser

#[cfg(test)]
use std::ops;

use nom::error::ParseError;
use oneil_shared::partial::PartialError;

use crate::error::ParserError;

/// A wrapper around the shared partial error type that can be used as a parser error
#[derive(Debug, Clone)]
pub(crate) struct ParserPartialError<T, E>(PartialError<T, Vec<E>>);

impl<T, E> ParserPartialError<T, E> {
    /// Creates a new `ErrorsWithPartialResult` with the given partial result
    /// and errors
    #[must_use]
    pub const fn new(partial_result: T, error_collection: Vec<E>) -> Self {
        Self(PartialError::new(partial_result, error_collection))
    }
}

impl<I, T, E> ParseError<I> for ParserPartialError<T, E>
where
    T: Default,
    E: ParseError<I>,
{
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self(PartialError::new(
            T::default(),
            vec![E::from_error_kind(input, kind)],
        ))
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<T, E> From<E> for ParserPartialError<T, ParserError>
where
    T: Default,
    E: Into<ParserError>,
{
    fn from(e: E) -> Self {
        Self(PartialError::new(T::default(), vec![e.into()]))
    }
}

impl<T, E> From<ParserPartialError<T, E>> for PartialError<T, Vec<E>> {
    fn from(partial_error: ParserPartialError<T, E>) -> Self {
        partial_error.0
    }
}

#[cfg(test)]
impl<T, E> ops::Deref for ParserPartialError<T, E> {
    type Target = PartialError<T, Vec<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
