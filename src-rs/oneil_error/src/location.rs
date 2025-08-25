/// Source location information for error reporting
///
/// This struct provides detailed information about where an error occurred
/// in the source code, including line and column numbers, character offset,
/// and the source line content. Line and column numbers are 1-indexed.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorLocation {
    /// Character offset from the beginning of the source file
    offset: usize,
    /// Line number where the error occurred (1-indexed)
    line: usize,
    /// Column number where the error occurred (1-indexed)
    column: usize,
    /// Optional length of the error span in characters
    length: Option<usize>,
    /// The source line content where the error occurred
    line_source: String,
}

impl ErrorLocation {
    /// Creates a new error location from source content and position information
    ///
    /// # Arguments
    ///
    /// * `source` - The complete source file content
    /// * `offset` - Character offset from the beginning of the source
    /// * `length` - Optional length of the error span
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `offset` is greater than the source length
    /// - `length` is provided and is 0
    /// - `length` is provided and `offset + length` exceeds the source length
    /// - `length` is provided and the span contains newlines
    fn new(source: &str, offset: usize, length: Option<usize>) -> Self {
        // offset must be less than or equal to the length of the source because
        // the offset may be at the very end of the source (after the last
        // character), and the length may be 1 (for a single character).
        assert!(
            offset <= source.len(),
            "offset ({}) must be less than or equal to the length of the source ({})",
            offset,
            source.len()
        );

        if let Some(length) = length {
            assert!(length > 0, "length must not be 0");

            // if an offset and length are provided, the offset + length must be
            // less than or equal to the length of the source because including
            // a length indicates that the error is attempting to highlight a
            // range of characters, and the range must be within the source
            assert!(
                offset + length <= source.len(),
                "offset + length ({}) must be less than or equal to the length of the source ({})",
                offset + length,
                source.len()
            );

            // make sure that there are no newlines in the range, since
            // multi-line errors are not currently supported
            assert!(
                !source[offset..offset + length].contains('\n'),
                "span ({:?}) must not contain newlines",
                &source[offset..offset + length]
            );
        }

        // Find the offset of the first newline before the given offset.
        // The beginning of the file (offset 0) is assumed if there is no
        // newline before the offset.
        let line_start = source[..offset]
            .rfind('\n')
            .map_or(0, |newline_idx| newline_idx + 1);

        // Count the number of tabs before the offset
        let num_tabs = source[line_start..offset]
            .chars()
            .filter(|c| *c == '\t')
            .count();

        // The column is the offset of the error from the beginning of the line
        // (+ 1 because the column is 1-indexed)
        let column_without_tabs = offset - line_start + 1;

        // The tab characters are already counted as 1 character, so we need to
        // add 3 spaces for each tab, for a total of 4 characters per tab
        let column = column_without_tabs + num_tabs * 3;

        // Count the number of newlines before the offset to get the line number
        // (+ 1 because the line is 1-indexed)
        let line = source[..offset].chars().filter(|c| *c == '\n').count() + 1;

        // The line is 1-indexed, so we need to subtract 1 to get the correct line
        let line_source = source.lines().nth(line - 1).unwrap().to_string();

        // Replace tabs with 4 spaces
        let line_source = line_source.replace('\t', "    ");

        Self {
            offset,
            line,
            column,
            length,
            line_source,
        }
    }

    /// Creates a new error location from source content and offset
    ///
    /// # Arguments
    ///
    /// * `source` - The complete source file content
    /// * `offset` - Character offset from the beginning of the source
    ///
    /// # Returns
    ///
    /// Returns a new `ErrorLocation` with the calculated line and column information.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `offset` is greater than the source length
    pub fn from_source_and_offset(source: &str, offset: usize) -> Self {
        Self::new(source, offset, None)
    }

    /// Creates a new error location from source content and span
    ///
    /// # Arguments
    ///
    /// * `source` - The complete source file content
    /// * `offset` - Character offset from the beginning of the source
    /// * `length` - Length of the error span in characters
    ///
    /// # Returns
    ///
    /// Returns a new `ErrorLocation` with the calculated line, column, and span information.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `offset` is greater than the source length
    /// - `length` is 0
    /// - `offset + length` exceeds the source length
    /// - the span delineated by `offset` and `length` contains newlines
    pub fn from_source_and_span(source: &str, offset: usize, length: usize) -> Self {
        Self::new(source, offset, Some(length))
    }

    /// Returns the character offset from the beginning of the source file
    ///
    /// # Returns
    ///
    /// Returns the character offset as a `usize`.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the line number where the error occurred (1-indexed)
    ///
    /// # Returns
    ///
    /// Returns the line number as a `usize`.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the column number where the error occurred (1-indexed)
    ///
    /// # Returns
    ///
    /// Returns the column number as a `usize`.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Returns the length of the error span in characters
    ///
    /// If no length was specified during creation, defaults to 1 character.
    ///
    /// # Returns
    ///
    /// Returns the span length as a `usize`.
    pub fn length(&self) -> usize {
        // if no length is provided, assume a single character
        self.length.unwrap_or(1)
    }

    /// Returns the source line content where the error occurred
    ///
    /// # Returns
    ///
    /// Returns a reference to the source line string.
    pub fn line_source(&self) -> &str {
        &self.line_source
    }
}
