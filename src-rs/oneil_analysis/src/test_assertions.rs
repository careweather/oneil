//! Shared assertion helpers for analysis tests.

use oneil_frontend::{InstanceGraph, InstanceValidationError, InstanceValidationErrorKind};
use oneil_shared::symbols::ParameterName;

use crate::output::{DependencyName, DependencyTreeValue, Tree, error::TreeErrors};

/// Asserts that a tree-errors collection is empty.
///
/// # Panics
///
/// Panics if any model path has recorded tree errors.
#[track_caller]
pub fn assert_no_tree_errors(errors: &TreeErrors) {
    assert!(
        errors.model_paths().next().is_none(),
        "expected no tree errors, got {errors:?}"
    );
}

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

/// Returns the dependency names of direct children, in tree order.
#[must_use]
pub fn child_dependency_names(tree: &Tree<DependencyTreeValue>) -> Vec<&DependencyName> {
    tree.children()
        .iter()
        .map(|child| &child.value().dependency_name)
        .collect()
}

/// Returns kinds of all validation errors, in order.
#[must_use]
pub fn validation_error_kinds(
    errors: &[InstanceValidationError],
) -> Vec<&InstanceValidationErrorKind> {
    errors.iter().map(InstanceValidationError::kind).collect()
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

/// Asserts an independents map entry equals the expected `(name, value)` pairs.
///
/// # Panics
///
/// Panics if the maps differ.
#[track_caller]
pub fn assert_independent_params(
    actual: &indexmap::IndexMap<ParameterName, oneil_output::Value>,
    expected: &[(&str, f64)],
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "independents length mismatch: {actual:?} vs {expected:?}"
    );
    for &(name, value) in expected {
        assert_eq!(
            actual.get(&ParameterName::from(name)),
            Some(&oneil_output::Value::from(value)),
            "missing or wrong independent `{name}` in {actual:?}"
        );
    }
}
