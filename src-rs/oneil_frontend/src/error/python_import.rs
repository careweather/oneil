use std::fmt::Display;

use oneil_shared::{
    error::{AsOneilDiagnostic, Context, DiagnosticKind, ErrorLocation},
    paths::PythonPath,
    span::Span,
};

/// Represents an error that occurred during Python import validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonImportResolutionError {
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

impl PythonImportResolutionError {
    /// Creates a new import resolution error indicating that a duplicate import was detected.
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
    #[must_use]
    pub const fn failed_validation(ident_span: Span, python_path: PythonPath) -> Self {
        Self::FailedValidation {
            ident_span,
            python_path,
        }
    }
}

impl Display for PythonImportResolutionError {
    /// Converts the import resolution error to a string representation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateImport { python_path, .. } => {
                let path = python_path.as_path().display();
                write!(f, "duplicate import of `{path}`")
            }
            Self::FailedValidation { python_path, .. } => {
                let path = python_path.as_path().display();
                write!(f, "unable to import python file `{path}`")
            }
        }
    }
}

impl AsOneilDiagnostic for PythonImportResolutionError {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn diagnostic_location(&self, _source: &str) -> Option<ErrorLocation> {
        match self {
            Self::DuplicateImport { duplicate_span, .. } => {
                let location = ErrorLocation::from_span(duplicate_span);
                Some(location)
            }
            Self::FailedValidation { ident_span, .. } => {
                let location = ErrorLocation::from_span(ident_span);
                Some(location)
            }
        }
    }

    fn context(&self) -> Vec<Context> {
        match self {
            Self::DuplicateImport { .. } | Self::FailedValidation { .. } => vec![],
        }
    }

    fn context_with_source(&self, _source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        match self {
            Self::DuplicateImport { original_span, .. } => {
                let location = ErrorLocation::from_span(original_span);
                vec![(
                    Context::Note("original import found here".to_string()),
                    Some(location),
                )]
            }
            Self::FailedValidation { .. } => vec![],
        }
    }
}
