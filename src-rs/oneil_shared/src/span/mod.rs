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
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The start offset is greater than the end offset
    /// - The start line is greater than the end line
    /// - The lines are equal but the start column is greater than the end column
    #[must_use]
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
    #[must_use]
    pub const fn start(&self) -> &SourceLocation {
        &self.start
    }

    /// Returns the end source location
    #[must_use]
    pub const fn end(&self) -> &SourceLocation {
        &self.end
    }

    /// Creates a span from the start of the start span to the end of the end span
    #[must_use]
    pub fn from_start_and_end(start: &Self, end: &Self) -> Self {
        Self::new(*start.start(), *end.end())
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

impl PartialOrd for SourceLocation {
    /// Custom partial ordering that is based on the offset
    ///
    /// If two source locations have the same offset but different line or column,
    /// there is no order between them. They likely come from different files.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let same_offset = self.offset == other.offset;
        let same_line = self.line == other.line;
        let same_column = self.column == other.column;

        if same_offset && (!same_line || !same_column) {
            None
        } else {
            Some(self.offset.cmp(&other.offset))
        }
    }
}
