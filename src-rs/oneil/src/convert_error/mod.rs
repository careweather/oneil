pub mod file;
pub mod loader;
pub mod parser;

pub use file::convert as convert_file_error;

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    path: PathBuf,
    message: String,
    location: Option<ErrorLocation>,
}

impl Error {
    pub fn new(path: PathBuf, message: String) -> Self {
        Self {
            path,
            message,
            location: None,
        }
    }

    pub fn new_from_offset(
        path: PathBuf,
        message: String,
        location: Option<(&str, usize)>,
    ) -> Self {
        let location = location
            .map(|(contents, offset)| ErrorLocation::from_source_and_offset(contents, offset));
        Self {
            path,
            message,
            location,
        }
    }

    pub fn new_from_span(
        path: PathBuf,
        message: String,
        location: Option<(&str, usize, usize)>,
    ) -> Self {
        let location = location.map(|(contents, offset, length)| {
            ErrorLocation::from_source_and_span(contents, offset, length)
        });
        Self {
            path,
            message,
            location,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn location(&self) -> Option<&ErrorLocation> {
        self.location.as_ref()
    }
}

// note that line and column are 1-indexed
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorLocation {
    offset: usize,
    line: usize,
    column: usize,
    length: usize,
    line_source: String,
}

impl ErrorLocation {
    pub fn from_source_and_offset(source: &str, offset: usize) -> Self {
        Self::from_source_and_span(source, offset, 1)
    }

    pub fn from_source_and_span(source: &str, offset: usize, length: usize) -> Self {
        assert!(length > 0, "length must be greater than 0");
        assert!(
            offset < source.len(),
            "offset must be less than the length of the source"
        );
        assert!(
            offset + length <= source.len(),
            "offset + length must be less than or equal to the length of the source"
        );

        // Find the offset of the first newline before the given offset.
        // The beginning of the file (offset 0) is assumed if there is no
        // newline before the offset.
        let line_start = source[..offset]
            .rfind('\n')
            .map_or(0, |newline_idx| newline_idx + 1);

        // The column is the offset of the error from the beginning of the line
        // (+ 1 because the column is 1-indexed)
        let column = offset - line_start + 1;

        // Count the number of newlines before the offset to get the line number
        // (+ 1 because the line is 1-indexed)
        let line = source[..offset].chars().filter(|c| *c == '\n').count() + 1;

        // The line is 1-indexed, so we need to subtract 1 to get the correct line
        let line_source = source.lines().nth(line - 1).unwrap().to_string();

        Self {
            offset,
            line,
            column,
            length,
            line_source,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn line_source(&self) -> &str {
        &self.line_source
    }
}
