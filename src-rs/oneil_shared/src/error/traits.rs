use crate::error::{Context, ErrorLocation};

/// Trait for types that can be converted to Oneil error messages.
///
/// This trait provides a standardized interface for error types to expose
/// their error message and associated context. It's used throughout the
/// Oneil compiler and parser to ensure consistent error reporting.
pub trait AsOneilError {
    /// Returns the primary error message.
    ///
    /// This should be a concise, user-friendly description of what went wrong.
    /// The message should be clear enough for users to understand the issue
    /// without requiring additional context.
    fn message(&self) -> String;

    /// Returns additional context information about the error.
    ///
    /// Context provides supplementary information that can help users understand
    /// the error better or suggest how to fix it. This might include:
    /// - Notes with additional context about the error
    /// - Help text with suggestions for fixing the error
    /// - References to related code locations
    /// - Examples of correct usage
    ///
    /// Returns an empty vector if no context is available.
    fn context(&self) -> Vec<Context> {
        vec![]
    }

    /// Returns the location of the error in the source code.
    ///
    /// This method should analyze the provided source code and return the
    /// precise location (line and column) where the error occurred. The
    /// location information is used for highlighting errors in the source
    /// code during error reporting.
    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        let _ = source;
        None
    }

    /// Returns context with optional source code locations.
    ///
    /// Similar to `context()`, but each context item can optionally include a specific
    /// location in the source code. This is useful when context refers to
    /// specific parts of the code (e.g., "variable 'x' was declared here").
    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        let _ = source;
        vec![]
    }
}
