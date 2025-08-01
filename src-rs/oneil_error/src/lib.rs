mod context;
mod location;
mod traits;

use std::path::PathBuf;

pub use context::Context;
pub use location::ErrorLocation;
pub use traits::AsOneilError;

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
    /// Optional context information
    context: Vec<Context>,
    /// Optional context information with source location
    context_with_source: Vec<(Context, ErrorLocation)>,
}

impl OneilError {
    pub fn from_error(error: &impl AsOneilError, path: PathBuf) -> Self {
        let message = error.message();
        let location = None;
        let context = error.context();
        let context_with_source = vec![];

        Self {
            path,
            message,
            location,
            context,
            context_with_source,
        }
    }

    pub fn from_error_with_source(error: &impl AsOneilError, path: PathBuf, source: &str) -> Self {
        let message = error.message();
        let location = error.error_location(source);
        // TODO: this can be more efficient, ponder this once the refactoring is done
        let (context_without_source, context_with_source) =
            error.context_with_source(source).into_iter().fold(
                (vec![], vec![]),
                |(mut context_without_source, mut context_with_source), (context, location)| {
                    match location {
                        Some(location) => {
                            context_with_source.push((context, location));
                        }
                        None => {
                            context_without_source.push(context);
                        }
                    }
                    (context_without_source, context_with_source)
                },
            );
        let context = error
            .context()
            .into_iter()
            .chain(context_without_source)
            .collect();

        Self {
            path,
            message,
            location,
            context,
            context_with_source,
        }
    }

    pub fn from_error_with_optional_source(
        error: &impl AsOneilError,
        path: PathBuf,
        source: Option<&str>,
    ) -> Self {
        match source {
            Some(source) => Self::from_error_with_source(error, path, source),
            None => Self::from_error(error, path),
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

    /// Returns the optional context information
    ///
    /// # Returns
    ///
    /// Returns a reference to the context information.
    pub fn context(&self) -> &[Context] {
        &self.context
    }

    /// Returns the optional context information with source location
    ///
    /// # Returns
    ///
    /// Returns a reference to the context information with source location.
    pub fn context_with_source(&self) -> &[(Context, ErrorLocation)] {
        &self.context_with_source
    }
}
