//! Partial result error handling for the Oneil parser
//!
//! This module provides error handling types that allow parsing operations to
//! return partial results even when errors occur. This is particularly useful
//! for providing better error recovery and more informative error messages
//! in the Oneil language parser.
//!
//! # Key Types
//!
//! - [`ErrorsWithPartialResult<T, E>`]: A wrapper that combines a partial result
//!   with a list of parsing errors
//! - [`CanBeEmpty`]: A trait for types that can represent an empty state
//!
//! # Usage
//!
//! This module is primarily used with the `nom` parsing library to provide
//! error recovery capabilities. When a parser encounters an error, it can
//! still return any successfully parsed content along with the error
//! information.
//!
//! # Integration with nom
//!
//! The `ErrorsWithPartialResult` type implements `nom::error::ParseError`,
//! allowing it to be used directly with nom parsers for error recovery
//! scenarios.

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
    ///
    /// This contains whatever content was successfully parsed before
    /// the error occurred. For types that implement `CanBeEmpty`, this
    /// will be the empty state when no content was successfully parsed.
    pub partial_result: T,
    /// The list of errors encountered during parsing
    ///
    /// This vector contains all parsing errors that occurred during
    /// the operation. Multiple errors can be collected to provide
    /// comprehensive error reporting.
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
///
/// An empty vector represents no successfully parsed items.
impl<T> CanBeEmpty for Vec<T> {
    fn empty() -> Self {
        Self::new()
    }
}

/// Implementation for `Option<T>` - returns `None`
///
/// `None` represents the absence of a successfully parsed value.
impl<T> CanBeEmpty for Option<T> {
    fn empty() -> Self {
        None
    }
}

/// Implementation for `Box<T>` - returns an empty box
///
/// An empty box represents no successfully parsed value.
impl<T: CanBeEmpty> CanBeEmpty for Box<T> {
    fn empty() -> Self {
        Self::new(T::empty())
    }
}

/// Implementation for `Model` - returns an empty model
///
/// An empty model represents no successfully parsed model.
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
