mod location;

use std::path::PathBuf;

pub use location::ErrorLocation;

/// Unified error representation for Oneil
///
/// This struct represents errors in a format suitable for display to users.
/// It includes the file path where the error occurred, a human-readable message,
/// and optional source location information for precise error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct OneilError {
    /// The path to the file where the error occurred
    path: PathBuf,
    /// Human-readable error message
    message: String,
    /// Optional source location information for precise error reporting
    location: Option<ErrorLocation>,
}

impl OneilError {
    /// Creates a new error without source location information
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file where the error occurred
    /// * `message` - Human-readable error message
    ///
    /// # Returns
    ///
    /// Returns a new `Error` instance without location information.
    pub fn new(path: PathBuf, message: String) -> Self {
        Self {
            path,
            message,
            location: None,
        }
    }

    /// Creates a new error with source location information from an offset
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file where the error occurred
    /// * `message` - Human-readable error message
    /// * `location` - Optional tuple of (source_contents, offset) for location calculation
    ///
    /// # Returns
    ///
    /// Returns a new `Error` instance with calculated location information.
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

    /// Creates a new error with source location information from a span
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file where the error occurred
    /// * `message` - Human-readable error message
    /// * `location` - Optional tuple of (source_contents, offset, length) for span calculation
    ///
    /// # Returns
    ///
    /// Returns a new `Error` instance with calculated span location information.
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

    /// Returns the path to the file where the error occurred
    ///
    /// # Returns
    ///
    /// Returns a reference to the `PathBuf` containing the file path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the human-readable error message
    ///
    /// # Returns
    ///
    /// Returns a reference to the error message string.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the optional source location information
    ///
    /// # Returns
    ///
    /// Returns an optional reference to the `ErrorLocation` if available.
    pub fn location(&self) -> Option<&ErrorLocation> {
        self.location.as_ref()
    }
}
