/// Represents an error that occurred during Python import validation.
///
/// This error type is used when a Python import declaration cannot be validated,
/// typically because the referenced Python file does not exist or cannot be imported.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResolutionError;

impl ImportResolutionError {
    /// Creates a new import resolution error.
    ///
    /// # Returns
    ///
    /// A new `ImportResolutionError` instance.
    pub fn new() -> Self {
        Self
    }

    /// Converts the import resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the import resolution error.
    pub fn to_string(&self) -> String {
        "python import validation failed".to_string()
    }
}
