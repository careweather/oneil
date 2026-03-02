use std::{io::Error as IoError, path::PathBuf};

use oneil_output::UnitConversionError;
use oneil_shared::{
    error::{AsOneilError, Context, ErrorLocation},
    span::Span,
};

/// Error type for source loading failures.
#[derive(Debug)]
pub struct SourceError {
    path: PathBuf,
    error: IoError,
}

impl SourceError {
    /// Creates a new source error from a path and I/O error.
    #[must_use]
    pub const fn new(path: PathBuf, error: IoError) -> Self {
        Self { path, error }
    }
}

impl AsOneilError for SourceError {
    fn message(&self) -> String {
        format!("couldn't read `{}` - {}", self.path.display(), self.error)
    }
}

/// Unit conversion error augmented with source spans for CLI/runtime reporting.
#[derive(Debug, Clone)]
pub struct RuntimeUnitConversionError {
    error: UnitConversionError,
    value_expr_span: Span,
    unit_span: Span,
}

impl RuntimeUnitConversionError {
    /// Creates a new runtime unit conversion error with source spans.
    #[must_use]
    pub const fn new(error: UnitConversionError, value_expr_span: Span, unit_span: Span) -> Self {
        Self {
            error,
            value_expr_span,
            unit_span,
        }
    }
}

impl AsOneilError for RuntimeUnitConversionError {
    fn message(&self) -> String {
        match &self.error {
            UnitConversionError::UnitMismatch {
                value_unit,
                target_unit,
            } => format!("cannot convert from unit `{value_unit}` to `{target_unit}`"),
            UnitConversionError::InvalidType {
                value_type,
                target_unit,
            } => format!("cannot convert value of type `{value_type}` to unit `{target_unit}`"),
        }
    }

    fn error_location(&self, source: &str) -> Option<ErrorLocation> {
        Some(ErrorLocation::from_source_and_span(source, self.unit_span))
    }

    fn context(&self) -> Vec<Context> {
        match &self.error {
            UnitConversionError::UnitMismatch { .. } => Vec::new(),
            UnitConversionError::InvalidType {
                value_type: _,
                target_unit: _,
            } => vec![Context::Note(
                "only numbers and measured numbers can be converted to a unit".to_string(),
            )],
        }
    }

    fn context_with_source(&self, source: &str) -> Vec<(Context, Option<ErrorLocation>)> {
        match &self.error {
            UnitConversionError::UnitMismatch {
                value_unit,
                target_unit: _,
            } => vec![(
                Context::Note(format!("value has unit `{value_unit}`")),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    self.value_expr_span,
                )),
            )],
            UnitConversionError::InvalidType {
                value_type,
                target_unit: _,
            } => vec![(
                Context::Note(format!("value has type `{value_type}`")),
                Some(ErrorLocation::from_source_and_span(
                    source,
                    self.value_expr_span,
                )),
            )],
        }
    }
}

/// Error for a Python import that failed before or during loading.
///
/// Distinguishes failure to load the source (e.g. file not found) from
/// Python/loader errors. The source error is not stored; use the source cache
/// or path for details.
#[cfg(feature = "python")]
#[derive(Debug)]
pub enum PythonImportError {
    /// Source could not be loaded (e.g. file not found); the error is not stored here.
    HasSourceError,
    /// Python or the loader reported an error.
    LoadFailed(oneil_python::LoadPythonImportError),
}
