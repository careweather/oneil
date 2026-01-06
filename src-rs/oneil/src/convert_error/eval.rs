use std::fs;

use oneil_eval::ModelError;
use oneil_shared::error::OneilError;

/// Converts a model evaluation error into a unified CLI error format
///
/// Takes a `ModelError` from the evaluation process and converts it into a
/// unified format suitable for display in the CLI. Attempts to read the source
/// file to provide location information when available.
///
/// # Arguments
///
/// * `error` - The model evaluation error to convert
///
/// # Returns
///
/// Returns a new `OneilError` instance with the error message, context, and
/// location information (if the source file could be read).
pub fn convert(error: &ModelError) -> OneilError {
    let source = fs::read_to_string(&error.model_path).ok();
    OneilError::from_error_with_optional_source(
        &error.error,
        error.model_path.clone(),
        source.as_deref(),
    )
}
