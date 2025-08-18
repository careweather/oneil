use oneil_error::{AsOneilError, Context, ErrorLocation};
use oneil_ir::{reference::Identifier, span::Span};

use crate::error::VariableResolutionError;

/// Represents an error that occurred during test resolution.
///
/// This error type is used when a test cannot be resolved, typically due
/// to variable resolution errors within the test's expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum TestResolutionError {
    /// Error indicating that an input parameter has been declared multiple times.
    DuplicateInput {
        /// The identifier of the duplicate input parameter.
        identifier: Identifier,
        /// The span of the original input parameter declaration.
        original_span: Span,
        /// The span of the duplicate input parameter declaration.
        duplicate_span: Span,
    },
    /// Error indicating that a variable resolution failed within a test.
    VariableResolution(VariableResolutionError),
}

impl TestResolutionError {
    /// Creates a new duplicate input error.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the duplicate input
    /// * `original_span` - The span of the original input
    /// * `duplicate_span` - The span of the duplicate input
    ///
    /// # Returns
    ///
    /// A new `TestResolutionError` instance.
    pub fn duplicate_input(
        identifier: Identifier,
        original_span: Span,
        duplicate_span: Span,
    ) -> Self {
        Self::DuplicateInput {
            identifier,
            original_span,
            duplicate_span,
        }
    }

    /// Creates a new variable resolution error.
    ///
    /// # Arguments
    ///
    /// * `error` - The variable resolution error that occurred
    ///
    /// # Returns
    ///
    /// A new `TestResolutionError` instance.
    pub fn variable_resolution(error: VariableResolutionError) -> Self {
        Self::VariableResolution(error)
    }

    /// Converts the test resolution error to a string representation.
    ///
    /// This method delegates to the display module to format the error message
    /// in a user-friendly way.
    ///
    /// # Returns
    ///
    /// A string representation of the test resolution error.
    pub fn to_string(&self) -> String {
        match self {
            Self::DuplicateInput { identifier, .. } => {
                format!("duplicate input `{}`", identifier.as_str())
            }
            Self::VariableResolution(error) => error.to_string(),
        }
    }
}

impl From<VariableResolutionError> for TestResolutionError {
    fn from(error: VariableResolutionError) -> Self {
        Self::variable_resolution(error)
    }
}

impl AsOneilError for TestResolutionError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        match self {
            Self::DuplicateInput { duplicate_span, .. } => {
                Some(ErrorLocation::from_source_and_span(
                    source,
                    duplicate_span.start(),
                    duplicate_span.length(),
                ))
            }
            Self::VariableResolution(error) => error.error_location(source),
        }
    }

    fn context(&self) -> Vec<Context> {
        match self {
            Self::DuplicateInput { .. } => vec![],
            Self::VariableResolution(error) => error.context(),
        }
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        match self {
            Self::DuplicateInput { original_span, .. } => {
                let context = Context::Note("original input found here".to_string());
                vec![(
                    context,
                    Some(ErrorLocation::from_source_and_span(
                        source,
                        original_span.start(),
                        original_span.length(),
                    )),
                )]
            }
            Self::VariableResolution(error) => error.context_with_source(source),
        }
    }
}
