//! Source location and span management for Oneil IR.
//!
//! This module provides data structures for tracking source locations
//! in Oneil code, including spans (ranges of text) and wrapper types
//! that associate values with their source locations.

use std::ops::Deref;

/// Represents a span of text in a file.
///
/// A `Span` indicates a contiguous range of characters in a source file,
/// defined by a starting position and length. This is used for error reporting,
/// syntax highlighting, and other source location tracking features.
///
/// # Examples
///
/// ```rust
/// use oneil_ir::span::Span;
///
/// // A span starting at position 10 with length 5
/// let span = Span::new(10, 5);
/// assert_eq!(span.start(), 10);
/// assert_eq!(span.length(), 5);
/// assert_eq!(span.end(), 15);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    length: usize,
}

impl Span {
    /// Creates a new span with the given start position and length.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting position (0-indexed)
    /// * `length` - The number of characters in the span
    ///
    /// # Returns
    ///
    /// A new `Span` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::Span;
    ///
    /// let span = Span::new(5, 10);
    /// assert_eq!(span.start(), 5);
    /// assert_eq!(span.length(), 10);
    /// ```
    #[must_use]
    pub const fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// Returns the starting position of this span.
    ///
    /// # Returns
    ///
    /// The 0-indexed starting position.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the length of this span.
    ///
    /// # Returns
    ///
    /// The number of characters in the span.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Returns the ending position of this span (exclusive).
    ///
    /// # Returns
    ///
    /// The 0-indexed ending position (start + length).
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::Span;
    ///
    /// let span = Span::new(10, 5);
    /// assert_eq!(span.end(), 15);
    /// ```
    #[must_use]
    pub const fn end(&self) -> usize {
        self.start + self.length
    }
}

/// Associates a span of text with a value.
///
/// `WithSpan<T>` wraps a value of type `T` with source location information.
/// The span may not represent the exact location of the value itself, but rather
/// indicates where the value was derived from in the source code.
///
/// (TODO: is this a good idea?)
///
/// This is commonly used in parsers and compilers to maintain source location
/// information for error reporting and debugging.
///
/// # Examples
///
/// ```rust
/// use oneil_ir::span::{Span, WithSpan};
///
/// let span = Span::new(10, 5);
/// let value = WithSpan::new("hello", span);
/// assert_eq!(value.value(), &"hello");
/// assert_eq!(value.span().start(), 10);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WithSpan<T> {
    value: T,
    span: Span,
}

impl<T> WithSpan<T> {
    /// Creates a new `WithSpan<T>` with the given value and span.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to wrap
    /// * `span` - The source location span
    ///
    /// # Returns
    ///
    /// A new `WithSpan<T>` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::{Span, WithSpan};
    ///
    /// let span = Span::new(0, 10);
    /// let wrapped = WithSpan::new(42, span);
    /// assert_eq!(*wrapped, 42);
    /// ```
    #[must_use]
    pub const fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    /// Creates a new `WithSpan<T>` with a dummy span for testing purposes.
    ///
    /// This is useful for creating test data where the exact span doesn't matter.
    /// In production code, you should use `new()` with proper spans.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to wrap
    ///
    /// # Returns
    ///
    /// A new `WithSpan<T>` instance with a dummy span (start: 0, length: 0).
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::WithSpan;
    ///
    /// let test_value = WithSpan::test_new("test");
    /// assert_eq!(*test_value, "test");
    /// ```
    #[must_use]
    pub const fn test_new(value: T) -> Self {
        Self::new(value, Span::new(0, 0))
    }

    /// Returns a reference to the wrapped value.
    ///
    /// # Returns
    ///
    /// A reference to the value of type `T`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::{Span, WithSpan};
    ///
    /// let wrapped = WithSpan::new("hello", Span::new(0, 5));
    /// assert_eq!(wrapped.value(), &"hello");
    /// ```
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns a reference to the span.
    ///
    /// # Returns
    ///
    /// A reference to the `Span` associated with this value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::{Span, WithSpan};
    ///
    /// let span = Span::new(10, 5);
    /// let wrapped = WithSpan::new("test", span);
    /// assert_eq!(wrapped.span().start(), 10);
    /// ```
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Consumes the `WithSpan<T>` and returns the wrapped value.
    ///
    /// # Returns
    ///
    /// The value of type `T`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oneil_ir::span::{Span, WithSpan};
    ///
    /// let wrapped = WithSpan::new(42, Span::new(0, 2));
    /// let value = wrapped.take_value();
    /// assert_eq!(value, 42);
    /// ```
    #[must_use]
    pub fn take_value(self) -> T {
        self.value
    }
}

impl<T> Deref for WithSpan<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
