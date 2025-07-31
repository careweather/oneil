use oneil_error::{AsOneilError, AsOneilErrorWithSource, Context, ErrorLocation};
use oneil_ir::{reference::PythonPath, span::Span};

/// Represents an error that occurred during Python import validation.
///
/// This error type is used when a Python import declaration cannot be validated,
/// typically because the referenced Python file does not exist or cannot be imported.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportResolutionError {
    ident_span: Span,
    python_path: PythonPath,
}

impl ImportResolutionError {
    /// Creates a new import resolution error.
    ///
    /// # Returns
    ///
    /// A new `ImportResolutionError` instance.
    pub fn new(ident_span: Span, python_path: PythonPath) -> Self {
        Self {
            ident_span,
            python_path,
        }
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
        let path = self.python_path.as_ref().display();
        format!("unable to import python file `{}`", path)
    }
}

impl AsOneilError for ImportResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn context(&self) -> Vec<Context> {
        vec![]
    }
}

impl AsOneilErrorWithSource for ImportResolutionError {
    fn error_location(&self, source: &str) -> oneil_error::ErrorLocation {
        let offset = self.ident_span.start();
        let length = self.ident_span.length();
        ErrorLocation::from_source_and_span(source, offset, length)
    }

    fn context_with_source(
        &self,
        _source: &str,
    ) -> Vec<(Context, Option<oneil_error::ErrorLocation>)> {
        vec![]
    }
}
