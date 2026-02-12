//! Source loading for the runtime.

use std::io::Error as IoError;
use std::path::Path;

use oneil_shared::error::{AsOneilError, OneilError};

use super::Runtime;
use crate::output;

impl Runtime {
    /// Loads source code from a file.
    ///
    /// # Errors
    ///
    /// Returns a [`FileError`](output::error::FileError) if the file could not be read.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_source(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<&str, output::error::FileError> {
        let path = path.as_ref();

        self.watch_paths.insert(path.to_path_buf());

        // Read the source code from the file
        match std::fs::read_to_string(path) {
            Ok(source) => {
                self.source_cache.insert(path.to_path_buf(), source);
                let source = self.source_cache.get(path).expect("it was just inserted");

                Ok(source)
            }
            Err(e) => {
                let error = InternalIoError::new(path, e);
                let error = OneilError::from_error(&error, path.to_path_buf());

                self.errors_cache
                    .insert_file_errors(path.to_path_buf(), vec![error.clone()]);

                Err(output::error::FileError {
                    error: Box::new(error),
                })
            }
        }
    }
}

/// Error type for file loading failures.
pub(super) struct InternalIoError<'a> {
    path: &'a Path,
    error: IoError,
}

impl<'a> InternalIoError<'a> {
    /// Creates a new file error from a path and I/O error.
    pub const fn new(path: &'a Path, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for InternalIoError<'_> {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}
