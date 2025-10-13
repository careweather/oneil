//! Source location spans for error reporting and debugging

/// Represents a span of source code with start, end, and whitespace end positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstSpan {
    start: usize,
    length: usize,
    whitespace_length: usize,
}

impl AstSpan {
    /// Creates a new span with the given positions
    #[must_use]
    pub const fn new(start: usize, length: usize, whitespace_length: usize) -> Self {
        Self {
            start,
            length,
            whitespace_length,
        }
    }

    /// Returns the start position of the span
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the length of the span
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Returns the length of the whitespace after the span
    #[must_use]
    pub const fn whitespace_length(&self) -> usize {
        self.whitespace_length
    }

    /// Returns the end position of the span
    #[must_use]
    pub const fn end(&self) -> usize {
        self.start + self.length
    }

    /// Returns the position where whitespace ends after the span
    #[must_use]
    pub const fn whitespace_end(&self) -> usize {
        self.start + self.length + self.whitespace_length
    }

    /// Calculates a span from two span-like objects
    #[must_use]
    pub fn calc_span<T, U>(start_span: &T, end_span: &U) -> Self
    where
        T: SpanLike,
        U: SpanLike,
    {
        Self::new(
            start_span.get_start(),
            end_span.get_end() - start_span.get_start(),
            end_span.get_whitespace_length(),
        )
    }

    /// Calculates a span from three span-like objects
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
            end_span.get_end() - start_span.get_start(),
            whitespace_span.get_length() + whitespace_span.get_whitespace_length(),
        )
    }
}

/// Trait for objects that can provide span information
pub trait SpanLike {
    /// Returns the start position
    fn get_start(&self) -> usize;
    /// Returns the length of the span
    fn get_length(&self) -> usize;
    /// Returns the length of the whitespace after the span
    fn get_whitespace_length(&self) -> usize;

    /// Returns the end position of the span
    fn get_end(&self) -> usize {
        self.get_start() + self.get_length()
    }

    /// Returns the position where whitespace ends after the span
    fn get_whitespace_end(&self) -> usize {
        self.get_end() + self.get_whitespace_length()
    }
}

impl SpanLike for AstSpan {
    fn get_start(&self) -> usize {
        self.start
    }

    fn get_length(&self) -> usize {
        self.length
    }

    fn get_whitespace_length(&self) -> usize {
        self.whitespace_length
    }
}

impl<T> From<&T> for AstSpan
where
    T: SpanLike,
{
    fn from(span: &T) -> Self {
        Self::new(
            span.get_start(),
            span.get_length(),
            span.get_whitespace_length(),
        )
    }
}
