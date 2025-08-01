use oneil_error::{AsOneilError, ErrorLocation};

use crate::error::VariableResolutionError;

/// Represents an error that occurred during model test input resolution.
///
/// This error type is used when a model test input cannot be resolved, typically
/// due to variable resolution errors within the input's expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelTestInputResolutionError {
    /// A variable resolution error occurred within the test input.
    VariableResolution(VariableResolutionError),
}

impl ModelTestInputResolutionError {
    /// Creates a new error indicating a variable resolution error in a test input.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `ModelTestInputResolutionError::VariableResolution` variant.
    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }

    /// Converts the model test input resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the model test input resolution error.
    pub fn to_string(&self) -> String {
        match self {
            ModelTestInputResolutionError::VariableResolution(variable_error) => {
                variable_error.to_string()
            }
        }
    }
}

impl From<VariableResolutionError> for ModelTestInputResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

impl AsOneilError for ModelTestInputResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            ModelTestInputResolutionError::VariableResolution(variable_error) => {
                variable_error.error_location(source)
            }
        }
    }
}
