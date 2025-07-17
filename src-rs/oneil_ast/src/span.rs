//! Source location spans for error reporting and debugging
//!
//! This module provides the `Span` struct and related functionality for
//! tracking source code locations in the AST.

/// Represents a span of source code with start, end, and whitespace end positions
///
/// Spans are used throughout the AST to provide precise location information
/// for error reporting, debugging, and other source-aware operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
    whitespace_end: usize,
}

impl Span {
    /// Creates a new span with the given positions
    pub fn new(start: usize, end: usize, whitespace_end: usize) -> Self {
        Self {
            start,
            end,
            whitespace_end,
        }
    }

    /// Returns the start position of the span
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end position of the span
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the position where whitespace ends after the span
    pub fn whitespace_end(&self) -> usize {
        self.whitespace_end
    }

    /// Calculates a span from two span-like objects
    ///
    /// The resulting span starts at the start of the first object and ends
    /// at the end of the second object, with whitespace end from the second object.
    pub fn calc_span<T, U>(start_span: &T, end_span: &U) -> Self
    where
        T: SpanLike,
        U: SpanLike,
    {
        Self::new(
            start_span.get_start(),
            end_span.get_end(),
            end_span.get_whitespace_end(),
        )
    }

    /// Calculates a span from three span-like objects
    ///
    /// The resulting span starts at the start of the first object, ends at the
    /// end of the second object, and uses the whitespace end from the third object.
    pub fn calc_span_with_whitespace<T, U, V>(
        start_span: &T,
        end_span: &U,
        whitespace_span: &V,
    ) -> Self
    where
        T: SpanLike,
        U: SpanLike,
        V: SpanLike,
    {
        Self::new(
            start_span.get_start(),
            end_span.get_end(),
            whitespace_span.get_whitespace_end(),
        )
    }
}

/// Trait for objects that can provide span information
///
/// This trait allows different types to provide their source location
/// information in a uniform way.
pub trait SpanLike {
    /// Returns the start position
    fn get_start(&self) -> usize;
    /// Returns the end position
    fn get_end(&self) -> usize;
    /// Returns the position where whitespace ends
    fn get_whitespace_end(&self) -> usize;
}

impl SpanLike for Span {
    fn get_start(&self) -> usize {
        self.start
    }

    fn get_end(&self) -> usize {
        self.end
    }

    fn get_whitespace_end(&self) -> usize {
        self.whitespace_end
    }
}

impl<T, U> From<(&T, &U)> for Span
where
    T: SpanLike,
    U: SpanLike,
{
    fn from((start_span, end_span): (&T, &U)) -> Self {
        Self::calc_span(start_span, end_span)
    }
}

impl<T> From<&T> for Span
where
    T: SpanLike,
{
    fn from(span: &T) -> Self {
        Self::new(span.get_start(), span.get_end(), span.get_whitespace_end())
    }
}
