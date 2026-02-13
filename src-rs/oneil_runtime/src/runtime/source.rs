//! Source loading for the runtime.

use std::path::Path;

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
    pub fn load_source(
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
