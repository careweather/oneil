use std::fmt::Display;

use oneil_error::{AsOneilError, Context, ErrorLocation};
use oneil_ir::{reference::PythonPath, span::Span};

/// Represents an error that occurred during Python import validation.
///
/// This error type is used when a Python import declaration cannot be validated,
/// typically because the referenced Python file does not exist or cannot be imported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportResolutionError {
    /// A duplicate import was detected.
    DuplicateImport {
        /// The span of the original import declaration.
        original_span: Span,
        /// The span of the duplicate import declaration.
        duplicate_span: Span,
        /// The Python path of the duplicate import.
        python_path: PythonPath,
    },
    /// A validation error occurred during import resolution.
    FailedValidation {
        /// The span of the import declaration that caused the validation error.
        ident_span: Span,
        /// The Python path of the import that failed validation.
        python_path: PythonPath,
    },
}

impl ImportResolutionError {
    /// Creates a new import resolution error indicating that a duplicate import was detected.
    ///
    /// # Arguments
    ///
    /// * `original_span` - The span of the original import declaration.
    /// * `duplicate_span` - The span of the duplicate import declaration.
    /// * `python_path` - The Python path of the duplicate import.
    ///
    /// # Returns
    ///
    /// A new `ImportResolutionError::DuplicateImport` variant.
    #[must_use]
    pub const fn duplicate_import(
        original_span: Span,
        duplicate_span: Span,
        python_path: PythonPath,
    ) -> Self {
        Self::DuplicateImport {
            original_span,
            duplicate_span,
            python_path,
        }
    }

    /// Creates a new import resolution error indicating that validation failed for a Python import.
    ///
    /// # Arguments
    ///
    /// * `ident_span` - The span of the import declaration that failed validation
    /// * `python_path` - The Python path that could not be validated
    ///
    /// # Returns
    ///
    /// A new `ImportResolutionError::FailedValidation` variant.
    #[must_use]
    pub const fn failed_validation(ident_span: Span, python_path: PythonPath) -> Self {
        Self::FailedValidation {
            ident_span,
            python_path,
        }
    }
}

impl Display for ImportResolutionError {
    /// Converts the import resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the import resolution error.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateImport { python_path, .. } => {
                let path = python_path.as_ref().display();
                write!(f, "duplicate import of `{path}`")
            }
            Self::FailedValidation { python_path, .. } => {
                let path = python_path.as_ref().display();
                write!(f, "unable to import python file `{path}`")
            }
        }
    }
}

impl AsOneilError for ImportResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::DuplicateImport { duplicate_span, .. } => {
                let offset = duplicate_span.start();
                let length = duplicate_span.length();
                let location = ErrorLocation::from_source_and_span(source, offset, length);
                Some(location)
            }
            Self::FailedValidation { ident_span, .. } => {
                let offset = ident_span.start();
                let length = ident_span.length();
                let location = ErrorLocation::from_source_and_span(source, offset, length);
                Some(location)
            }
        }
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        match self {
            Self::DuplicateImport { original_span, .. } => {
                let offset = original_span.start();
                let length = original_span.length();
                let location = ErrorLocation::from_source_and_span(source, offset, length);
                vec![(
                    Context::Note("original import found here".to_string()),
                    Some(location),
                )]
            }
            Self::FailedValidation { .. } => vec![],
        }
    }
}
