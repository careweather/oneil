//! Shared assertion helpers for evaluator tests.

use std::collections::BTreeMap;

use oneil_output::util::is_close;
use oneil_output::{
    Dimension, DimensionMap, EvalError, ExpectedType, Number, Unit, Value, ValueType,
};

use crate::eval_parameter::EvalParameterResult;

/// Formats a table-row (or other case) label as a panic-message prefix.
fn case_prefix(case: &str) -> String {
    if case.is_empty() {
        String::new()
    } else {
        format!("{case}: ")
    }
}

/// Asserts that two floats are close (`is_close`).
///
/// # Panics
///
/// Panics if the values are not close.
#[track_caller]
pub fn assert_is_close(expected: f64, actual: f64) {
    assert_is_close_case("", expected, actual);
}

/// Like [`assert_is_close`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if the values are not close.
#[track_caller]
pub fn assert_is_close_case(case: &str, expected: f64, actual: f64) {
    assert!(
        is_close(expected, actual),
        "{}expected: {expected}, actual: {actual}",
        case_prefix(case)
    );
}

/// Asserts that a unit has the expected dimension map.
///
/// # Panics
///
/// Panics if the dimension maps do not match.
#[track_caller]
pub fn assert_units_dimensionally_eq(
    expected_unit_list: impl IntoIterator<Item = (Dimension, f64)>,
    actual_unit: &Unit,
) {
    assert_units_dimensionally_eq_case("", expected_unit_list, actual_unit);
}

/// Like [`assert_units_dimensionally_eq`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if the dimension maps do not match.
#[track_caller]
pub fn assert_units_dimensionally_eq_case(
    case: &str,
    expected_unit_list: impl IntoIterator<Item = (Dimension, f64)>,
    actual_unit: &Unit,
) {
    let expected = DimensionMap::new(BTreeMap::from_iter(expected_unit_list));
    assert_eq!(
        expected,
        actual_unit.dimension_map,
        "{}dimension map mismatch",
        case_prefix(case)
    );
}

/// Asserts magnitude, dimensions, and dB flag for a unit.
///
/// # Panics
///
/// Panics if any checked field differs.
#[track_caller]
pub fn assert_unit_eq(
    unit: &Unit,
    expected_magnitude: f64,
    expected_dims: &[(Dimension, f64)],
    expected_is_db: bool,
) {
    assert_unit_eq_case("", unit, expected_magnitude, expected_dims, expected_is_db);
}

/// Like [`assert_unit_eq`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if any checked field differs.
#[track_caller]
pub fn assert_unit_eq_case(
    case: &str,
    unit: &Unit,
    expected_magnitude: f64,
    expected_dims: &[(Dimension, f64)],
    expected_is_db: bool,
) {
    assert_is_close_case(case, expected_magnitude, unit.magnitude);
    assert_units_dimensionally_eq_case(case, expected_dims.iter().copied(), unit);
    assert_eq!(
        expected_is_db,
        unit.is_db,
        "{}is_db mismatch",
        case_prefix(case)
    );
}

/// Asserts that `value` is a scalar number close to `expected`.
///
/// # Panics
///
/// Panics if the value is not a scalar number or is not close.
#[track_caller]
pub fn assert_scalar_close(expected: f64, value: &Value) {
    assert_scalar_close_case("", expected, value);
}

/// Like [`assert_scalar_close`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if the value is not a scalar number or is not close.
#[track_caller]
pub fn assert_scalar_close_case(case: &str, expected: f64, value: &Value) {
    let Value::Number(Number::Scalar(actual)) = value else {
        panic!("{}expected scalar number, got {value:?}", case_prefix(case));
    };
    assert_is_close_case(case, expected, *actual);
}

/// Asserts that `value` is the given boolean.
///
/// # Panics
///
/// Panics if the values differ.
#[track_caller]
pub fn assert_boolean(expected: bool, value: &Value) {
    assert_eq!(value, &Value::Boolean(expected));
}

/// Asserts that `value` is a measured scalar with the given normalized value and unit fields.
///
/// # Panics
///
/// Panics if the value shape or fields do not match.
#[track_caller]
pub fn assert_measured_scalar(
    value: &Value,
    expected_normalized: f64,
    expected_dims: &[(Dimension, f64)],
    expected_magnitude: f64,
    expected_is_db: bool,
) {
    assert_measured_scalar_case(
        "",
        value,
        expected_normalized,
        expected_dims,
        expected_magnitude,
        expected_is_db,
    );
}

/// Like [`assert_measured_scalar`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if the value shape or fields do not match.
#[track_caller]
pub fn assert_measured_scalar_case(
    case: &str,
    value: &Value,
    expected_normalized: f64,
    expected_dims: &[(Dimension, f64)],
    expected_magnitude: f64,
    expected_is_db: bool,
) {
    let Value::MeasuredNumber(number) = value else {
        panic!(
            "{}expected measured number, got {value:?}",
            case_prefix(case)
        );
    };
    let Number::Scalar(actual) = *number.normalized_value().as_number() else {
        panic!(
            "{}expected scalar normalized value, got {value:?}",
            case_prefix(case)
        );
    };
    assert_is_close_case(case, expected_normalized, actual);
    assert_unit_eq_case(
        case,
        number.unit(),
        expected_magnitude,
        expected_dims,
        expected_is_db,
    );
}

/// Asserts that a successful parameter evaluation produced a measured scalar.
///
/// # Panics
///
/// Panics if the value shape or fields do not match.
#[track_caller]
pub fn assert_param_measured_scalar(
    result: &EvalParameterResult,
    expected_normalized: f64,
    expected_dims: &[(Dimension, f64)],
    expected_magnitude: f64,
    expected_is_db: bool,
) {
    assert_param_measured_scalar_case(
        "",
        result,
        expected_normalized,
        expected_dims,
        expected_magnitude,
        expected_is_db,
    );
}

/// Like [`assert_param_measured_scalar`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if the value shape or fields do not match.
#[track_caller]
pub fn assert_param_measured_scalar_case(
    case: &str,
    result: &EvalParameterResult,
    expected_normalized: f64,
    expected_dims: &[(Dimension, f64)],
    expected_magnitude: f64,
    expected_is_db: bool,
) {
    assert_measured_scalar_case(
        case,
        &result.value,
        expected_normalized,
        expected_dims,
        expected_magnitude,
        expected_is_db,
    );
}

/// Asserts that a successful parameter evaluation produced a scalar number.
///
/// # Panics
///
/// Panics if the value is not a scalar number or is not close.
#[track_caller]
pub fn assert_param_scalar_close(result: &EvalParameterResult, expected: f64) {
    assert_param_scalar_close_case("", result, expected);
}

/// Like [`assert_param_scalar_close`], but prefixes failure messages with `case`.
///
/// # Panics
///
/// Panics if the value is not a scalar number or is not close.
#[track_caller]
pub fn assert_param_scalar_close_case(case: &str, result: &EvalParameterResult, expected: f64) {
    assert_scalar_close_case(case, expected, &result.value);
}

/// Asserts that `error` is [`EvalError::InvalidType`] with the given types.
///
/// # Panics
///
/// Panics if the error variant or types differ.
#[track_caller]
pub fn assert_invalid_type(error: &EvalError, expected: &ExpectedType, found: &ValueType) {
    assert!(
        matches!(
            error,
            EvalError::InvalidType {
                expected_type,
                found_type,
                ..
            } if expected_type == expected && found_type == found
        ),
        "expected InvalidType {{ expected: {expected:?}, found: {found:?} }}, got {error:?}"
    );
}

/// Asserts that `error` is [`EvalError::TypeMismatch`] with the given types.
///
/// # Panics
///
/// Panics if the error variant or types differ.
#[track_caller]
pub fn assert_type_mismatch(error: &EvalError, expected: &ExpectedType, found: &ValueType) {
    assert!(
        matches!(
            error,
            EvalError::TypeMismatch {
                expected_type,
                found_type,
                ..
            } if expected_type == expected && found_type == found
        ),
        "expected TypeMismatch {{ expected: {expected:?}, found: {found:?} }}, got {error:?}"
    );
}
