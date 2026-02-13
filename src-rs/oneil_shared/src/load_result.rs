//! Load result type.
//!
//! [`LoadResult<T, E>`] represents the outcome of a load operation that may
//! succeed fully, succeed partially (with a value and errors), or fail entirely.

/// Result of a load operation with three possible outcomes.
///
/// - [`LoadResult::Success`]: the load completed successfully with value `T`.
/// - [`LoadResult::Partial`]: a value `T` was produced but errors `E` also occurred.
/// - [`LoadResult::Failure`]: the load failed entirely; nothing was produced, including errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadResult<T, E> {
    /// Load failed completely; no value and no errors (e.g. unable to even start).
    Failure,
    /// Load produced a value but errors also occurred.
    Partial(T, E),
    /// Load completed successfully.
    Success(T),
}

impl<T, E> LoadResult<T, E> {
    /// Builds a failure result with no value and no errors.
    #[must_use]
    pub const fn failure() -> Self {
        Self::Failure
    }

    /// Builds a partial result with a value and errors.
    #[must_use]
    pub const fn partial(value: T, error: E) -> Self {
        Self::Partial(value, error)
    }

    /// Builds a successful result.
    #[must_use]
    pub const fn success(value: T) -> Self {
        Self::Success(value)
    }

    /// Returns `true` if this is [`LoadResult::Success`].
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Returns `true` if this is [`LoadResult::Partial`].
    #[must_use]
    pub const fn is_partial(&self) -> bool {
        matches!(self, Self::Partial(_, _))
    }

    /// Returns `true` if this is [`LoadResult::Failure`].
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failure)
    }

    /// Returns a reference projection of the load result.
    #[must_use]
    pub const fn as_ref(&self) -> LoadResult<&T, &E> {
        match self {
            Self::Failure => LoadResult::Failure,
            Self::Partial(v, e) => LoadResult::Partial(v, e),
            Self::Success(v) => LoadResult::Success(v),
        }
    }

    /// Returns the value if present: `Some` for [`Success`](LoadResult::Success) or
    /// [`Partial`](LoadResult::Partial), `None` for [`Failure`](LoadResult::Failure).
    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        match self {
            Self::Success(v) | Self::Partial(v, _) => Some(v),
            Self::Failure => None,
        }
    }

    /// Returns the error if present: `Some` for [`Partial`](LoadResult::Partial),
    /// `None` for [`Success`](LoadResult::Success) or [`Failure`](LoadResult::Failure).
    #[must_use]
    pub const fn error(&self) -> Option<&E> {
        match self {
            Self::Partial(_, e) => Some(e),
            Self::Failure | Self::Success(_) => None,
        }
    }

    /// Maps a function over the value if present.
    pub fn map<U, F>(self, f: F) -> LoadResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Success(v) => LoadResult::success(f(v)),
            Self::Partial(v, e) => LoadResult::partial(f(v), e),
            Self::Failure => LoadResult::failure(),
        }
    }

    /// Maps a function over the error if present.
    pub fn map_err<F, E2>(self, f: F) -> LoadResult<T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        match self {
            Self::Success(v) => LoadResult::success(v),
            Self::Partial(v, e) => LoadResult::partial(v, f(e)),
            Self::Failure => LoadResult::failure(),
        }
    }
}
