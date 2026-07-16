//! Shared assertion helpers for analysis tests.

use oneil_frontend::{InstanceGraph, InstanceValidationError, InstanceValidationErrorKind};

/// Asserts that validation produced no errors.
///
/// # Panics
///
/// Panics if `graph.validation_errors` is non-empty.
#[track_caller]
pub fn assert_no_validation_errors(graph: &InstanceGraph) {
    assert!(
        graph.validation_errors.is_empty(),
        "expected no validation errors, got {:?}",
        graph.validation_errors
    );
}

/// Asserts that `errors` contains a [`ParameterCycle`] for `parameter_name`.
///
/// # Panics
///
/// Panics if no matching cycle error is found.
#[track_caller]
pub fn assert_has_parameter_cycle(errors: &[InstanceValidationError], parameter_name: &str) {
    let found = errors.iter().any(|err| {
        matches!(
            &err.kind,
            InstanceValidationErrorKind::ParameterCycle {
                parameter_name: name,
                ..
            } if name.as_str() == parameter_name
        )
    });
    assert!(
        found,
        "expected ParameterCycle for `{parameter_name}`, got {errors:?}"
    );
}

/// Collects cycle member parameter names from all cycle errors.
#[must_use]
pub fn parameter_cycle_member_names(errors: &[InstanceValidationError]) -> Vec<&str> {
    errors
        .iter()
        .filter_map(|err| match &err.kind {
            InstanceValidationErrorKind::ParameterCycle { parameter_name, .. } => {
                Some(parameter_name.as_str())
            }
            InstanceValidationErrorKind::UndefinedParameter { .. }
            | InstanceValidationErrorKind::UndefinedReference { .. }
            | InstanceValidationErrorKind::UndefinedReferenceParameter { .. }
            | InstanceValidationErrorKind::ReferenceHasError { .. } => None,
        })
        .collect()
}
