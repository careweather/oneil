//! Error type for file loading failures.

use oneil_shared::error::OneilError;

/// Error type for file loading failures.
///
/// Holds a single [`OneilError`] (e.g. from I/O failure) in a [`Box`].
#[derive(Clone, Debug)]
pub struct FileError {
    pub error: Box<OneilError>,
}

impl FileError {
    /// Returns the underlying error(s) as a list of [`OneilError`]s.
    #[must_use]
    pub fn to_vec(&self) -> Vec<OneilError> {
        vec![(*self.error).clone()]
    }
}
