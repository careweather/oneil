use std::io::Error as IoError;

use oneil_shared::{
    error::{AsOneilDiagnostic, DiagnosticKind},
    paths::SourcePath,
};

/// Error type for source loading failures.
#[derive(Debug)]
pub struct SourceError {
    path: SourcePath,
    error: IoError,
}

impl SourceError {
    /// Creates a new source error from a path and I/O error.
    #[must_use]
    pub const fn new(path: SourcePath, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilDiagnostic for SourceError {
    fn kind(&self) -> DiagnosticKind {
        DiagnosticKind::Error
    }

    fn message(&self) -> String {
        format!(
            "couldn't read `{}` - {}",
            self.path.as_path().display(),
            self.error
        )
    }
}

/// Error for a Python import that failed before or during loading.
///
/// Distinguishes failure to load the source (e.g. file not found) from
/// Python/loader errors. The source error is not stored; use the source cache
/// or path for details.
#[derive(Debug)]
pub enum PythonImportError {
    /// Source could not be loaded (e.g. file not found); the error is not stored here.
    HasSourceError,
    /// Python or the loader reported an error.
    LoadFailed(oneil_python::LoadPythonImportError),
}
