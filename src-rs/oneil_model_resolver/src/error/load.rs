//! Load-level errors for the Oneil model loader.

use crate::error::resolution;

/// Represents errors that can occur during model loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The model source could not be parsed.
    ParseError,
    /// Errors that occurred during dependency resolution.
    ResolutionErrors(Box<resolution::ResolutionErrors>),
}

impl LoadError {
    /// Creates a new parse error.
    #[must_use]
    pub const fn parse_error() -> Self {
        Self::ParseError
    }

    /// Creates a new resolution error.
    #[must_use]
    pub fn resolution_errors(resolution_errors: resolution::ResolutionErrors) -> Self {
        Self::ResolutionErrors(Box::new(resolution_errors))
    }
}
