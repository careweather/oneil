//! Error handling for the Oneil model loader.

use std::fmt;

use oneil_error::AsOneilError;
use oneil_ir as ir;

pub mod collection;
pub mod resolution;
pub mod util;

pub use resolution::{
    ImportResolutionError, ModelImportResolutionError, ParameterResolutionError, ResolutionErrors,
    TestResolutionError, VariableResolutionError,
};
pub use util::{combine_error_list, combine_errors, convert_errors, split_ok_and_errors};

/// Represents errors that can occur during model loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError<Ps> {
    /// Error that occurred during AST parsing of a model file.
    ParseError(Ps),
    /// Errors that occurred during dependency resolution.
    ResolutionErrors(Box<resolution::ResolutionErrors>),
}

impl<Ps> LoadError<Ps> {
    /// Creates a new parse error.
    #[must_use]
    pub const fn parse_error(parse_error: Ps) -> Self {
        Self::ParseError(parse_error)
    }

    /// Creates a new resolution error.
    #[must_use]
    pub fn resolution_errors(resolution_errors: resolution::ResolutionErrors) -> Self {
        Self::ResolutionErrors(Box::new(resolution_errors))
    }
}

/// Represents a circular dependency detected during model loading.
///
/// A circular dependency occurs when model A depends on model B, which depends on
/// model C, which depends back on model A (or any other cycle). This error contains
/// the complete cycle of model paths that form the circular dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircularDependencyError(Vec<ir::ModelPath>);

impl CircularDependencyError {
    /// Creates a new circular dependency error.
    #[must_use]
    pub const fn new(circular_dependency: Vec<ir::ModelPath>) -> Self {
        Self(circular_dependency)
    }

    /// Returns the circular dependency path.
    #[must_use]
    pub const fn circular_dependency(&self) -> &[ir::ModelPath] {
        self.0.as_slice()
    }
}

impl fmt::Display for CircularDependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let circular_dependency_str = self
            .0
            .iter()
            .map(|path| path.as_ref().display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        write!(
            f,
            "circular dependency found in models - {circular_dependency_str}"
        )
    }
}

impl AsOneilError for CircularDependencyError {
    fn message(&self) -> String {
        self.to_string()
    }
}
