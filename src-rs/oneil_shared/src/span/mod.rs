//! Source location spans for mapping data structures to source code

/// A span of source code
///
/// A span is a pair of source locations, representing the start and end of the
/// span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: SourceLocation,
    end: SourceLocation,
}

impl Span {
    /// Creates a new span from a start and end source location
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        assert!(
            start.offset <= end.offset,
            "start offset must be before end offset"
        );

        assert!(
            start.line < end.line || (start.line == end.line && start.column <= end.column),
            "start line and column must be before end line and column"
        );

        Self { start, end }
    }

    /// Returns the start source location
    pub fn start(&self) -> &SourceLocation {
        &self.start
    }

    /// Returns the end source location
    pub fn end(&self) -> &SourceLocation {
        &self.end
    }
}

/// A source location
///
/// A source location is a position in the source code, represented by an
/// offset, line, and column.
///
/// Note that it is assumed that the offset corresponds to the line and column.
/// If this assumption is not correct, any code that relies on the line and
/// column for display purposes will be incorrect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// The offset from the beginning of the source code (0-indexed)
    pub offset: usize,
    /// The line number (1-indexed)
    pub line: usize,
    /// The column number (1-indexed)
    pub column: usize,
}
