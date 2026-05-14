//! Source location spans for mapping data structures to source code

use std::{path::Path, sync::Arc};

/// A span of source code.
///
/// A `Span` is a pair of [`SourceLocation`]s (start and end), coupled with
/// the file path and full source text the span was parsed from.
///
/// Embedding the path and source directly in the span keeps error reporting
/// self-contained: callers no longer need to pass the correct source string
/// alongside a span, and there is no risk of mismatching the span from one
/// file with the source of another.  This is especially important in the LSP
/// (where the source may change while the language server is running) and when
/// IR from multiple files is combined — for example when a design overrides a
/// parameter equation.
///
/// Because [`Rc`] is not [`Copy`], `Span` is deliberately `Clone`-only.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Span {
    start: SourceLocation,
    end: SourceLocation,
    /// Path of the file this span was parsed from.
    #[serde(skip)]
    path: Arc<Path>,
    /// Full source text of the file this span was parsed from.
    #[serde(skip)]
    source: Arc<str>,
}

impl Span {
    /// Creates a new span from a start and end source location.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The start offset is greater than the end offset
    /// - The start line is greater than the end line
    /// - The lines are equal but the start column is greater than the end column
    #[must_use]
    pub fn new(
        start: SourceLocation,
        end: SourceLocation,
        path: Arc<Path>,
        source: Arc<str>,
    ) -> Self {
        assert!(
            start.offset <= end.offset,
            "start offset must be before end offset"
        );

        assert!(
            start.line < end.line || (start.line == end.line && start.column <= end.column),
            "start line and column must be before end line and column"
        );

        Self {
            start,
            end,
            path,
            source,
        }
    }

    /// Creates an empty (zero-length) span at the given source location.
    #[must_use]
    pub const fn empty(source_location: SourceLocation, path: Arc<Path>, source: Arc<str>) -> Self {
        Self {
            start: source_location,
            end: source_location,
            path,
            source,
        }
    }

    /// Creates a span that runs from the start of `start` to the end of `end`.
    ///
    /// The path and source are taken from `start`; both spans are assumed to
    /// belong to the same source file.
    #[must_use]
    pub fn from_start_and_end(start: &Self, end: &Self) -> Self {
        Self::new(
            *start.start(),
            *end.end(),
            Arc::clone(&start.path),
            Arc::clone(&start.source),
        )
    }

    /// Returns the start source location.
    #[must_use]
    pub const fn start(&self) -> &SourceLocation {
        &self.start
    }

    /// Returns the end source location.
    #[must_use]
    pub const fn end(&self) -> &SourceLocation {
        &self.end
    }

    /// Returns the path of the file this span was parsed from.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the full source text of the file this span was parsed from.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Creates a synthetic span that does not correspond to any real source location.
    ///
    /// Use this when a `Span` is required for a value that was computed or
    /// inferred (e.g. a default value) rather than parsed from a source file.
    /// The resulting span has an empty path and empty source text.
    #[must_use]
    pub fn synthetic() -> Self {
        let loc = SourceLocation {
            offset: 0,
            line: 1,
            column: 1,
        };
        Self {
            start: loc,
            end: loc,
            path: Arc::from(Path::new("")),
            source: Arc::from(""),
        }
    }

    /// Generates a random span for testing purposes.
    ///
    /// The returned span uses an empty path and empty source string.
    #[cfg(feature = "random_span")]
    #[must_use]
    pub fn random_span() -> Self {
        use rand::RngExt;
        let mut rng = rand::rng();

        let start_offset = usize::from(rng.random::<u16>());
        let start_line = usize::from(rng.random::<u16>());
        let start_column = usize::from(rng.random::<u16>());

        let end_offset = start_offset + usize::from(rng.random::<u16>());
        let end_line = start_line + usize::from(rng.random::<u16>());
        let end_column = start_column + usize::from(rng.random::<u16>());

        let start_loc = SourceLocation {
            offset: start_offset,
            line: start_line,
            column: start_column,
        };

        let end_loc = SourceLocation {
            offset: end_offset,
            line: end_line,
            column: end_column,
        };

        Self::new(start_loc, end_loc, Arc::from(Path::new("")), Arc::from(""))
    }
}

/// A source location.
///
/// A source location is a position in the source code, represented by an
/// offset, line, and column.
///
/// Note that it is assumed that the offset corresponds to the line and column.
/// If this assumption is not correct, any code that relies on the line and
/// column for display purposes will be incorrect.
// TODO: determine whether it would be worthwile to use utf-8 offset for the
//       column instead of bytes. If we choose to do so, we will need to do
//       benchmarks to see if
//       [`LocatedSpan::get_utf8_column`](https://docs.rs/nom_locate/5.0.0/nom_locate/struct.LocatedSpan.html#method.get_utf8_column)
//       or
//       [`LocatedSpan::naive_get_utf8_column`](https://docs.rs/nom_locate/5.0.0/nom_locate/struct.LocatedSpan.html#method.naive_get_utf8_column)
//       is faster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    /// The offset (in bytes) from the beginning of the source code (0-indexed)
    pub offset: usize,
    /// The line number (1-indexed)
    pub line: usize,
    /// The column number (in bytes) (1-indexed)
    pub column: usize,
}

impl PartialOrd for SourceLocation {
    /// Custom partial ordering that is based on the offset.
    ///
    /// If two source locations have the same offset but different line or
    /// column, there is no order between them — they likely come from different
    /// files.
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
