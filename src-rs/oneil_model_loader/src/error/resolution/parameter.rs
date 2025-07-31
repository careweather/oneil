use oneil_error::{AsOneilError, AsOneilErrorWithSource, ErrorLocation};
use oneil_ir::{reference::Identifier, span::Span};

use crate::error::VariableResolutionError;

/// Represents an error that occurred during parameter resolution.
///
/// This error type is used when a parameter reference cannot be resolved to its
/// actual parameter definition. This can happen due to circular dependencies or
/// variable resolution errors within the parameter's value.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterResolutionError {
    /// A circular dependency was detected during parameter resolution.
    CircularDependency {
        /// The list of parameter identifiers that form the circular dependency.
        circular_dependency: Vec<Identifier>,
        /// The span of the parameter that caused the circular dependency.
        reference_span: Span,
    },
    /// A variable resolution error occurred within the parameter's value.
    VariableResolution(VariableResolutionError),
}

impl ParameterResolutionError {
    /// Creates a new error indicating a circular dependency in parameter resolution.
    ///
    /// # Arguments
    ///
    /// * `circular_dependency` - The list of parameter identifiers that form the circular dependency
    ///
    /// # Returns
    ///
    /// A new `ParameterResolutionError::CircularDependency` variant.
    pub fn circular_dependency(circular_dependency: Vec<Identifier>, reference_span: Span) -> Self {
        Self::CircularDependency {
            circular_dependency,
            reference_span,
        }
    }

    /// Creates a new error indicating a variable resolution error within a parameter.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `ParameterResolutionError::VariableResolution` variant.
    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }

    /// Converts the parameter resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the parameter resolution error.
    pub fn to_string(&self) -> String {
        match self {
            ParameterResolutionError::CircularDependency {
                circular_dependency,
                reference_span: _,
            } => {
                let dependency_chain = circular_dependency
                    .iter()
                    .map(|id| format!("{}", id.as_str()))
                    .collect::<Vec<_>>()
                    .join(" -> ");
                format!(
                    "circular dependency detected in parameters - {}",
                    dependency_chain
                )
            }
            ParameterResolutionError::VariableResolution(variable_error) => {
                variable_error.to_string()
            }
        }
    }
}

impl From<VariableResolutionError> for ParameterResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

impl AsOneilError for ParameterResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }
}

impl AsOneilErrorWithSource for ParameterResolutionError {
    fn error_location(&self, source: &str) -> oneil_error::ErrorLocation {
        match self {
            ParameterResolutionError::CircularDependency {
                circular_dependency: _,
                reference_span,
            } => {
                let offset_start = reference_span.start();
                let length = reference_span.length();
                ErrorLocation::from_source_and_span(source, offset_start, length)
            }
            ParameterResolutionError::VariableResolution(error) => error.error_location(source),
        }
    }
}
