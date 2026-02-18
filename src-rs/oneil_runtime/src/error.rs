use std::{io::Error as IoError, path::PathBuf};

use oneil_shared::error::AsOneilError;

/// Error type for source loading failures.
#[derive(Debug)]
pub struct SourceError {
    path: PathBuf,
    error: IoError,
}

impl SourceError {
    /// Creates a new source error from a path and I/O error.
    #[must_use]
    pub const fn new(path: PathBuf, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for SourceError {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}

/// Error for a Python import that failed before or during loading.
///
/// Distinguishes failure to load the source (e.g. file not found) from
/// Python/loader errors. The source error is not stored; use the source cache
/// or path for details.
#[cfg(feature = "python")]
#[derive(Debug)]
pub enum PythonImportError {
    /// Source could not be loaded (e.g. file not found); the error is not stored here.
    HasSourceError,
    /// Python or the loader reported an error.
    LoadFailed(oneil_python::LoadPythonImportError),
}

#[cfg(feature = "python")]
impl oneil_shared::error::AsOneilError for PythonImportError {
    fn message(&self) -> String {
        match self {
            Self::HasSourceError => "Could not load source for Python import".to_string(),
            Self::LoadFailed(e) => e.message(),
        }
    }
}
