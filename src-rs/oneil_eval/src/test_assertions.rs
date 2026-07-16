//! Shared assertion helpers for evaluator tests.

use std::collections::BTreeMap;

use oneil_output::util::is_close;
use oneil_output::{
    Dimension, DimensionMap, EvalError, ExpectedType, Number, Unit, Value, ValueType,
};

use crate::eval_parameter::EvalParameterResult;

/// The outcome of checking a test assertion.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub enum Assertion {
    /// The checked condition is valid.
    Valid,
    /// The checked condition is invalid, with a description of the mismatch.
    Invalid(String),
}

impl Assertion {
    /// Lazily evaluates the next assertion when this assertion is valid.
    fn and_then(self, next: impl FnOnce() -> Self) -> Self {
        match self {
            Self::Valid => next(),
            invalid @ Self::Invalid(_) => invalid,
        }
    }

    /// Panics when this assertion is invalid.
    ///
    /// # Panics
    ///
    /// Panics with the assertion message when this is [`Assertion::Invalid`].
    #[track_caller]
    pub fn assert(self) {
        if let Self::Invalid(message) = self {
            panic!("{message}");
        }
    }

    /// Panics with a case name when this assertion is invalid.
    ///
    /// # Panics
    ///
    /// Panics with the case name and assertion message when this is
    /// [`Assertion::Invalid`].
    #[track_caller]
    pub fn assert_with_name(self, name: &str) {
        if let Self::Invalid(message) = self {
            panic!("{name}: {message}");
        }
    }
}

/// Checks that two floats are close (`is_close`).
pub fn check_is_close(expected: f64, actual: f64) -> Assertion {
    if is_close(expected, actual) {
        Assertion::Valid
    } else {
        Assertion::Invalid(format!("expected: {expected}, actual: {actual}"))
    }
}

/// Checks that a unit has the expected dimension map.
pub fn check_units_dimensionally_eq(
    expected_unit_list: impl IntoIterator<Item = (Dimension, f64)>,
    actual_unit: &Unit,
) -> Assertion {
    let expected = DimensionMap::new(BTreeMap::from_iter(expected_unit_list));
    if expected == actual_unit.dimension_map {
        Assertion::Valid
    } else {
        Assertion::Invalid(format!(
            "dimension map mismatch: expected {expected:?}, actual {:?}",
            actual_unit.dimension_map
        ))
    }
}

/// Checks magnitude, dimensions, and dB flag for a unit.
pub fn check_unit_eq(
    unit: &Unit,
    expected_magnitude: f64,
    expected_dims: &[(Dimension, f64)],
    expected_is_db: bool,
) -> Assertion {
    check_is_close(expected_magnitude, unit.magnitude)
        .and_then(|| check_units_dimensionally_eq(expected_dims.iter().copied(), unit))
        .and_then(|| {
            if expected_is_db == unit.is_db {
                Assertion::Valid
            } else {
                Assertion::Invalid(format!(
                    "is_db mismatch: expected {expected_is_db}, actual {}",
                    unit.is_db
                ))
            }
        })
}

/// Checks that `value` is a scalar number close to `expected`.
pub fn check_scalar_close(expected: f64, value: &Value) -> Assertion {
    let Value::Number(Number::Scalar(actual)) = value else {
        return Assertion::Invalid(format!("expected scalar number, got {value:?}"));
    };
    check_is_close(expected, *actual)
}

/// Checks that `value` is the given boolean.
pub fn check_boolean(expected: bool, value: &Value) -> Assertion {
    if value == &Value::Boolean(expected) {
        Assertion::Valid
    } else {
        Assertion::Invalid(format!("expected boolean {expected}, got {value:?}"))
    }
}

/// Checks that `value` is a measured scalar with the given normalized value and unit fields.
pub fn check_measured_scalar(
    value: &Value,
    expected_normalized: f64,
    expected_dims: &[(Dimension, f64)],
    expected_magnitude: f64,
    expected_is_db: bool,
) -> Assertion {
    let Value::MeasuredNumber(number) = value else {
        return Assertion::Invalid(format!("expected measured number, got {value:?}"));
    };
    let Number::Scalar(actual) = *number.normalized_value().as_number() else {
        return Assertion::Invalid(format!("expected scalar normalized value, got {value:?}"));
    };
    check_is_close(expected_normalized, actual).and_then(|| {
        check_unit_eq(
            number.unit(),
            expected_magnitude,
            expected_dims,
            expected_is_db,
        )
    })
}

/// Checks that a successful parameter evaluation produced a measured scalar.
pub fn check_param_measured_scalar(
    result: &EvalParameterResult,
    expected_normalized: f64,
    expected_dims: &[(Dimension, f64)],
    expected_magnitude: f64,
    expected_is_db: bool,
) -> Assertion {
    check_measured_scalar(
        &result.value,
        expected_normalized,
        expected_dims,
        expected_magnitude,
        expected_is_db,
    )
}

/// Checks that a successful parameter evaluation produced a scalar number.
pub fn check_param_scalar_close(result: &EvalParameterResult, expected: f64) -> Assertion {
    check_scalar_close(expected, &result.value)
}

/// Checks that `error` is [`EvalError::InvalidType`] with the given types.
pub fn check_invalid_type(
    error: &EvalError,
    expected: &ExpectedType,
    found: &ValueType,
) -> Assertion {
    if matches!(
        error,
        EvalError::InvalidType {
            expected_type,
            found_type,
            ..
        } if expected_type == expected && found_type == found
    ) {
        Assertion::Valid
    } else {
        Assertion::Invalid(format!(
            "expected InvalidType {{ expected: {expected:?}, found: {found:?} }}, got {error:?}"
        ))
    }
}

/// Checks that `error` is [`EvalError::TypeMismatch`] with the given types.
pub fn check_type_mismatch(
    error: &EvalError,
    expected: &ExpectedType,
    found: &ValueType,
) -> Assertion {
    if matches!(
        error,
        EvalError::TypeMismatch {
            expected_type,
            found_type,
            ..
        } if expected_type == expected && found_type == found
    ) {
        Assertion::Valid
    } else {
        Assertion::Invalid(format!(
            "expected TypeMismatch {{ expected: {expected:?}, found: {found:?} }}, got {error:?}"
        ))
    }
}
