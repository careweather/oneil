use crate::{Context, ErrorLocation};

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
    fn context(&self) -> Vec<Context>;
}

/// Trait for error types that can provide source code location information.
///
/// This trait extends `AsOneilError` to include source code location capabilities.
/// It's used when errors need to be displayed with specific line and column
/// information from the source code, enabling better error reporting with
/// precise location highlighting.
///
pub trait AsOneilErrorWithSource: AsOneilError {
    /// Returns the location of the error in the source code.
    ///
    /// This method should analyze the provided source code and return the
    /// precise location (line and column) where the error occurred. The
    /// location information is used for highlighting errors in the source
    /// code during error reporting.
    ///
    /// # Arguments
    ///
    /// * `source` - The complete source code string where the error occurred
    ///
    /// # Returns
    ///
    /// An `ErrorLocation` containing the line and column information for
    /// the error position.
    fn error_location(&self, source: &str) -> ErrorLocation;

    /// Returns context with optional source code locations.
    ///
    /// Similar to `context()`, but each context item can optionally include a specific
    /// location in the source code. This is useful when context refers to
    /// specific parts of the code (e.g., "variable 'x' was declared here").
    ///
    /// # Arguments
    ///
    /// * `source` - The complete source code string where the error occurred
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the context and an optional location.
    /// If a context item doesn't have a specific location, the location should be `None`.
    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)>;
}
