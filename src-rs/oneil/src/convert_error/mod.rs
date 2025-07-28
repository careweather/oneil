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

    pub fn new_with_contents(
        path: PathBuf,
        message: String,
        contents: &str,
        offset: usize,
    ) -> Self {
        let location = ErrorLocation::from_source_and_offset(contents, offset);
        Self {
            path,
            message,
            location: Some(location),
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
    line_source: String,
}

impl ErrorLocation {
    pub fn from_source_and_offset(source: &str, offset: usize) -> Self {
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
            line_source,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn line_source(&self) -> &str {
        &self.line_source
    }
}
