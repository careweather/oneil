//! File I/O error conversion for the Oneil CLI
//!
//! This module provides functionality for converting file I/O errors into the
//! unified error format used by the Oneil CLI. It handles errors that occur
//! when reading files from disk, such as missing files, permission issues,
//! and other file system errors.

use std::{io::Error as IoError, path::Path};

use oneil_error::Error;

/// Converts a file I/O error into a unified CLI error format
///
/// Takes a file path and an I/O error, then creates a user-friendly error message
/// that includes both the file path and the underlying I/O error description.
///
/// # Arguments
///
/// * `path` - The path to the file that caused the I/O error
/// * `error` - The I/O error that occurred
///
/// # Returns
///
/// Returns a new `Error` instance with a formatted message that includes
/// the file path and the I/O error description.
///
/// # Examples
///
/// ```rust
/// use std::io;
/// use std::path::Path;
/// use oneil::convert_error::file;
///
/// let path = Path::new("nonexistent.on");
/// let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
/// let error = file::convert(path, &io_error);
/// assert!(error.message().contains("couldn't read"));
/// ```
pub fn convert(path: &Path, error: &IoError) -> Error {
    let message = format!("couldn't read `{}` - {}", path.display(), error);
    Error::new(path.to_path_buf(), message)
}
