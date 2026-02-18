//! Source loading for the runtime.

use std::path::Path;

use oneil_shared::error::OneilError;

use super::Runtime;
use crate::error::SourceError;

impl Runtime {
    /// Loads source code from a file.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeErrors`] (via [`get_model_errors`](super::Runtime::get_model_errors)) if the file could not be read.
    pub fn load_source(&mut self, path: impl AsRef<Path>) -> Result<&str, Box<OneilError>> {
        let path = path.as_ref();

        self.load_source_internal(path)
            .as_ref()
            .map(String::as_str)
            .map_err(|e| Box::new(OneilError::from_error(e, path.to_path_buf())))
    }

    pub(super) fn load_source_internal(
        &mut self,
        path: impl AsRef<Path>,
    ) -> &Result<String, SourceError> {
        let path = path.as_ref();

        let result = match std::fs::read_to_string(path) {
            Ok(source) => Ok(source),
            Err(e) => Err(SourceError::new(path.to_path_buf(), e)),
        };

        self.source_cache.insert(path.to_path_buf(), result);

        self.source_cache
            .get_entry(path)
            .expect("it was just inserted")
    }
}
