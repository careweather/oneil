use std::ops::Deref;

// TODO: add docs stating that this represents a span of text in a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    length: usize,
}

impl Span {
    pub fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn end(&self) -> usize {
        self.start + self.length
    }
}

// TODO: add docs stating that this associates a span of text with a value.
//       However, the span may not represent the value exactly. It may indicate
//       where the value was derived from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WithSpan<T> {
    value: T,
    span: Span,
}

impl<T> WithSpan<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    /// Creates a new `WithSpan<T>` with a dummy span for testing purposes.
    ///
    /// This is useful for creating test data where the exact span doesn't matter.
    /// In production code, you should use `new()` with proper spans.
    pub fn test_new(value: T) -> Self {
        Self::new(value, Span::new(0, 0))
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

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
