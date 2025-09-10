//! Source location and span management for Oneil IR.

use std::ops::Deref;

/// Represents a span of text in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IrSpan {
    start: usize,
    length: usize,
}

impl IrSpan {
    /// Creates a new span with the given start position and length.
    ///
    /// NOTE: the starting position is 0-indexed.
    #[must_use]
    pub const fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// Returns the starting position of this span.
    ///
    /// NOTE: the starting position is 0-indexed.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the length of this span.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Returns the ending position of this span (exclusive).
    ///
    /// NOTE: the ending position is 0-indexed.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.start + self.length
    }
}

/// Associates a span of text with a value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WithSpan<T> {
    value: T,
    span: IrSpan,
}

impl<T> WithSpan<T> {
    /// Creates a new `WithSpan<T>` with the given value and span.
    ///
    #[must_use]
    pub const fn new(value: T, span: IrSpan) -> Self {
        Self { value, span }
    }

    /// Returns a reference to the wrapped value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns a reference to the span.
    #[must_use]
    pub const fn span(&self) -> IrSpan {
        self.span
    }

    /// Consumes the `WithSpan<T>` and returns the wrapped value.
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
