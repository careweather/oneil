use std::{error::Error, fmt};

use oneil_shared::{
    error::{AsOneilError, ErrorLocation},
    span::Span,
};

/// Represents an error that occurred during unit resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitResolutionError {
    /// The full unit name that could not be resolved
    unit_name: String,
    /// The span of the unit name in the source
    unit_name_span: Span,
}

impl UnitResolutionError {
    /// Creates a new error indicating that a unit could not be resolved.
    #[must_use]
    pub const fn new(unit_name: String, unit_name_span: Span) -> Self {
        Self {
            unit_name,
            unit_name_span,
        }
    }

    /// Returns the name of the unit that could not be resolved.
    #[must_use]
    pub fn unit_name(&self) -> &str {
        &self.unit_name
    }

    /// Returns the span of the unit name in the source.
    #[must_use]
    pub const fn unit_name_span(&self) -> Span {
        self.unit_name_span
    }
}

impl fmt::Display for UnitResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown unit `{}`", self.unit_name)
    }
}

impl Error for UnitResolutionError {}

impl AsOneilError for UnitResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        let location = ErrorLocation::from_source_and_span(source, self.unit_name_span);
        Some(location)
    }
}
