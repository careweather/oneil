//! Partial result error handling for the Oneil parser

use nom::error::ParseError;
use oneil_ast::Model;

use crate::error::ParserError;

/// An error type representing a parsing operation that failed, but may still
/// have a partial result
///
/// This type is useful for error recovery scenarios where a parser can
/// successfully parse some content before encountering an error. The partial
/// result can be used for debugging, error reporting, or in some cases,
/// partial evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
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
///
/// Types implementing this trait can represent a "no result" state,
/// which is useful when a parsing operation fails before producing
/// any meaningful output.
///
/// # Examples
///
/// ```rust
/// use oneil_parser::error::partial::CanBeEmpty;
/// use oneil_parser::error::ParserError;
///
/// let empty_vec: Vec<i32> = Vec::empty();
/// assert!(empty_vec.is_empty());
///
/// let empty_option: Option<String> = Option::empty();
/// assert!(empty_option.is_none());
/// ```
pub trait CanBeEmpty {
    /// Returns an empty instance of the type
    ///
    /// This method should return a value that represents the
    /// "empty" or "no result" state for the implementing type.
    fn empty() -> Self;
}

/// Implementation for `Vec<T>` - returns an empty vector
impl<T> CanBeEmpty for Vec<T> {
    fn empty() -> Self {
        Self::new()
    }
}

/// Implementation for `Option<T>` - returns `None`
impl<T> CanBeEmpty for Option<T> {
    fn empty() -> Self {
        None
    }
}

/// Implementation for `Box<T>` - returns an empty box
impl<T: CanBeEmpty> CanBeEmpty for Box<T> {
    fn empty() -> Self {
        Self::new(T::empty())
    }
}

/// Implementation for `Model` - returns an empty model
impl CanBeEmpty for Model {
    fn empty() -> Self {
        Self::new(None, vec![], vec![])
    }
}

impl<T, E> ErrorsWithPartialResult<T, E> {
    /// Creates a new `ErrorsWithPartialResult` with the given partial result
    /// and errors
    #[must_use]
    pub const fn new(partial_result: T, errors: Vec<E>) -> Self {
        Self {
            partial_result,
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
