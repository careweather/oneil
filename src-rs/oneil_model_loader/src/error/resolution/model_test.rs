use oneil_error::{AsOneilError, AsOneilErrorWithSource};

use crate::error::VariableResolutionError;

/// Represents an error that occurred during model test resolution.
///
/// This error type is used when a model test cannot be resolved, typically due
/// to variable resolution errors within the test's expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTestResolutionError(VariableResolutionError);

impl ModelTestResolutionError {
    /// Creates a new model test resolution error.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `ModelTestResolutionError` instance.
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

    /// Converts the model test resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the model test resolution error.
    pub fn to_string(&self) -> String {
        self.get_error().to_string()
    }
}

impl From<VariableResolutionError> for ModelTestResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::new(error)
    }
}

impl AsOneilError for ModelTestResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }
}

impl AsOneilErrorWithSource for ModelTestResolutionError {
    fn error_location(&self, source: &str) -> oneil_error::ErrorLocation {
        self.get_error().error_location(source)
    }
}
