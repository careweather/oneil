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
