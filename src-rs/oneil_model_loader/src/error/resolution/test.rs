use oneil_error::{AsOneilError, Context, ErrorLocation};

use crate::error::VariableResolutionError;

/// Represents an error that occurred during test resolution.
///
/// This error type is used when a test cannot be resolved, typically due
/// to variable resolution errors within the test's expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct TestResolutionError(VariableResolutionError);

impl TestResolutionError {
    /// Creates a new test resolution error.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `TestResolutionError` instance.
    pub fn new(error: VariableResolutionError) -> Self {
        Self(error)
    }

    /// Returns a reference to the variable resolution error that occurred.
    ///
    /// # Returns
    ///
    /// A reference to the variable resolution error.
    pub fn get_error(&self) -> &VariableResolutionError {
        &self.0
    }

    /// Converts the test resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the test resolution error.
    pub fn to_string(&self) -> String {
        self.get_error().to_string()
    }
}

impl From<VariableResolutionError> for TestResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::new(error)
    }
}

impl AsOneilError for TestResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        self.get_error().error_location(source)
    }

    fn context(&self) -> Vec<Context> {
        self.get_error().context()
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        self.get_error().context_with_source(source)
    }
}
