//! Source loading for the runtime.

use std::path::Path;

use oneil_shared::load_result::LoadResult;

use super::Runtime;
use crate::output::error::SourceError;

impl Runtime {
    /// Loads source code from a file.
    ///
    /// # Errors
    ///
    /// Returns a [`SourceError`](output::error::SourceError) if the file could not be read.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the panic only happens if an internal invariant is violated"
    )]
    pub fn load_source(&mut self, path: impl AsRef<Path>) -> &LoadResult<String, SourceError> {
        let path = path.as_ref();

        // Read the source code from the file
        let load_result = match std::fs::read_to_string(path) {
            Ok(source) => LoadResult::success(source),
            Err(e) => {
                let error = SourceError::new(path.to_path_buf(), e);

                LoadResult::failure(error)
            }
        };

        self.source_cache.insert(path.to_path_buf(), load_result);

        self.source_cache.get(path).expect("it was just inserted")
    }
}
