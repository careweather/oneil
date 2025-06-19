use nom::error::ParseError;

use super::ParserError;

/// An error type representing a parsing operation that failed, but may still
/// have a partial result
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorsWithPartialResult<T, E> {
    /// The partial result of the parsing operation
    pub partial_result: T,
    /// The list of errors encountered during parsing
    pub errors: Vec<E>,
}

/// A trait for types that can be empty
///
/// This trait allows `ErrorsWithPartialResult` to be produced from
/// just an error, without a partial result, since the partial result
/// is considered to be the empty state.
pub trait CanBeEmpty {
    /// Returns an empty instance of the type
    fn empty() -> Self;
}

impl<T> CanBeEmpty for Vec<T> {
    fn empty() -> Self {
        Self::new()
    }
}

impl<T> CanBeEmpty for Option<T> {
    fn empty() -> Self {
        None
    }
}

impl<T, E> ErrorsWithPartialResult<T, E> {
    /// Creates a new `ErrorsWithPartialResult` with the given partial result
    /// and errors
    pub fn new(partial_result: T, errors: Vec<E>) -> Self {
        Self {
            partial_result: partial_result,
            errors,
        }
    }
}

impl<I, T, E> ParseError<I> for ErrorsWithPartialResult<T, E>
where
    T: CanBeEmpty,
    E: ParseError<I>,
{
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self {
            partial_result: T::empty(),
            errors: vec![E::from_error_kind(input, kind)],
        }
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<T, E> From<E> for ErrorsWithPartialResult<T, ParserError>
where
    T: CanBeEmpty,
    E: Into<ParserError>,
{
    fn from(e: E) -> Self {
        Self {
            partial_result: T::empty(),
            errors: vec![e.into()],
        }
    }
}
