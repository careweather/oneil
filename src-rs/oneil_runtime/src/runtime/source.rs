//! Source loading for the runtime.

use std::path::Path;
use std::{io::Error as IoError, path::PathBuf};

use oneil_shared::LoadResult;
use oneil_shared::error::{AsOneilError, OneilError};

use super::Runtime;

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
    pub fn load_source(&mut self, path: impl AsRef<Path>) -> &LoadResult<String, FileLoadingError> {
        let path = path.as_ref();

        // Read the source code from the file
        let load_result = match std::fs::read_to_string(path) {
            Ok(source) => LoadResult::success(source),
            Err(e) => {
                let error = FileLoadingError::new(path.to_path_buf(), e);
                let error = OneilError::from_error(&error, path.to_path_buf());

                LoadResult::failure(error)
            }
        };

        self.source_cache.insert(path.to_path_buf(), load_result);

        self.source_cache.get(path).expect("it was just inserted")
    }
}

/// Error type for file loading failures.
pub(super) struct FileLoadingError {
    path: PathBuf,
    error: IoError,
}

impl FileLoadingError {
    /// Creates a new file error from a path and I/O error.
    pub const fn new(path: PathBuf, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for FileLoadingError {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}
