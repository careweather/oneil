//! Partial result type.
//!
//! [`MaybePartialResult`] represents an outcome that may be fully successful or
//! "partial": a value of type `T` was produced, but errors of type `E` also
//! occurred. On full success it holds `T`; on partial success it holds a
//! [`PartialError`] containing both the partial value and the error collection.

/// Result that may be fully successful or partial (value plus errors).
///
/// On success, holds a value of type `T`. On partial failure, holds a
/// [`PartialError<T, E>`] containing the partial value and an error collection
/// of type `E`.
#[derive(Debug)]
pub struct MaybePartialResult<T, E>(Result<T, PartialError<T, E>>);

impl<T, E> MaybePartialResult<T, E> {
    /// Builds a fully successful result.
    #[must_use]
    pub const fn ok(value: T) -> Self {
        Self(Ok(value))
    }

    /// Builds a partial result: a value was produced but errors occurred.
    #[must_use]
    pub const fn err(partial_result: T, error_collection: E) -> Self {
        let partial_error = PartialError::new(partial_result, error_collection);

        Self(Err(partial_error))
    }

    /// Returns the value: the success value if fully successful, or the partial
    /// value if this is a partial error.
    #[must_use]
    pub const fn maybe_partial_value(&self) -> &T {
        match &self.0 {
            Ok(value) => value,
            Err(partial_error) => &partial_error.partial_result,
        }
    }

    /// Converts this result into the underlying `Result`.
    #[expect(
        clippy::missing_errors_doc,
        reason = "this is a trivial transformation into the underlying result type"
    )]
    pub fn into_result(self) -> Result<T, PartialError<T, E>> {
        self.0
    }
}

impl<T, E> From<Result<T, PartialError<T, E>>> for MaybePartialResult<T, E> {
    fn from(result: Result<T, PartialError<T, E>>) -> Self {
        Self(result)
    }
}

impl<T, E> From<MaybePartialResult<T, E>> for Result<T, PartialError<T, E>> {
    fn from(result: MaybePartialResult<T, E>) -> Self {
        result.into_result()
    }
}

/// Error for a partial result: a value was produced but errors occurred.
///
/// Holds the partial value of type `T` and an error collection of type `E`.
#[derive(Debug, Clone)]
pub struct PartialError<T, E> {
    /// The value that was produced despite the errors.
    pub partial_result: T,
    /// The errors that occurred while producing the partial result.
    pub error_collection: E,
}

impl<T, E> PartialError<T, E> {
    /// Creates a partial error from a partial value and an error collection.
    #[must_use]
    pub const fn new(partial_result: T, error_collection: E) -> Self {
        Self {
            partial_result,
            error_collection,
        }
    }
}
