use std::fmt;

use oneil_ir::{self as ir, IrSpan};
use oneil_shared::error::{AsOneilError, Context, ErrorLocation};

use crate::error::VariableResolutionError;

/// Represents an error that occurred during parameter resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterResolutionError {
    /// A circular dependency was detected during parameter resolution.
    CircularDependency {
        /// The list of parameter identifiers that form the circular dependency.
        circular_dependency: Vec<ir::Identifier>,
        /// The span of the parameter that caused the circular dependency.
        reference_span: IrSpan,
    },
    /// A variable resolution error occurred within the parameter's value.
    VariableResolution(VariableResolutionError),
    /// A duplicate parameter was detected.
    DuplicateParameter {
        /// The identifier of the parameter.
        identifier: ir::Identifier,
        /// The span of the original parameter.
        original_span: IrSpan,
        /// The span of the duplicate parameter.
        duplicate_span: IrSpan,
    },
}

impl ParameterResolutionError {
    /// Creates a new error indicating a circular dependency in parameter resolution.
    #[must_use]
    pub const fn circular_dependency(
        circular_dependency: Vec<ir::Identifier>,
        reference_span: IrSpan,
    ) -> Self {
        Self::CircularDependency {
            circular_dependency,
            reference_span,
        }
    }

    /// Creates a new error indicating a variable resolution error within a parameter.
    #[must_use]
    pub const fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }

    /// Creates a new error indicating a duplicate parameter was detected.
    #[must_use]
    pub const fn duplicate_parameter(
        identifier: ir::Identifier,
        original_span: IrSpan,
        duplicate_span: IrSpan,
    ) -> Self {
        Self::DuplicateParameter {
            identifier,
            original_span,
            duplicate_span,
        }
    }
}

impl fmt::Display for ParameterResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircularDependency {
                circular_dependency,
                reference_span: _,
            } => {
                let dependency_chain = circular_dependency
                    .iter()
                    .map(ir::Identifier::as_str)
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(
                    f,
                    "circular dependency detected in parameters - {dependency_chain}",
                )
            }
            Self::VariableResolution(variable_error) => variable_error.fmt(f),
            Self::DuplicateParameter { identifier, .. } => {
                write!(f, "duplicate parameter `{}`", identifier.as_str())
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

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::CircularDependency {
                circular_dependency: _,
                reference_span,
            } => {
                let offset_start = reference_span.start();
                let length = reference_span.length();
                let location = ErrorLocation::from_source_and_span(source, offset_start, length);
                Some(location)
            }
            Self::VariableResolution(error) => error.error_location(source),
            Self::DuplicateParameter { duplicate_span, .. } => {
                let offset_start = duplicate_span.start();
                let length = duplicate_span.length();
                let location = ErrorLocation::from_source_and_span(source, offset_start, length);
                Some(location)
            }
        }
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        match self {
            Self::DuplicateParameter { original_span, .. } => {
                let original_location = ErrorLocation::from_source_and_span(
                    source,
                    original_span.start(),
                    original_span.length(),
                );
                let context = Context::Note("original parameter found here".to_string());
                vec![(context, Some(original_location))]
            }
            Self::CircularDependency { .. } => vec![],
            Self::VariableResolution(error) => error.context_with_source(source),
        }
    }
}
