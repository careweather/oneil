//! Utility functions for the value module

use oneil_shared::span::Span;

use crate::{
    EvalError,
    error::ExpectedType,
    value::{DimensionMap, DisplayUnit, MeasuredNumber, Number, Unit, Value},
};

const TOLERANCE: f64 = 1e-10;

/// Checks if two floating point numbers are close to each other.
///
/// This function uses the `Strong` comparison method defined in the
/// `is_close` crate as reference. See
/// <https://github.com/PM4Rs/is_close/blob/8475cd292946b6e5461375a41160153ce32e31c6/src/lib.rs#L183>
/// for more details.
///
/// In the future, we may want to implement other methods
/// from the `is_close` crate.
///
/// The tolerance is fixed at 1e-10.
#[must_use]
pub const fn is_close(a: f64, b: f64) -> bool {
    #[expect(
        clippy::float_cmp,
        reason = "this is a part of implementing better floating point comparison"
    )]
    if a == b {
        return true;
    }

    if a.is_infinite() || b.is_infinite() {
        return false;
    }

    if a.is_nan() || b.is_nan() {
        return false;
    }

    let difference = (a - b).abs();
    let relative_tolerance = TOLERANCE * f64::min(a.abs(), b.abs());
    let absolute_tolerance = TOLERANCE;

    difference <= relative_tolerance || difference <= absolute_tolerance
}

/// Converts a decibel number to a linear number.
#[must_use]
pub fn db_to_linear(value: Number) -> Number {
    Number::Scalar(10.0).pow(value / Number::Scalar(10.0))
}

/// Converts a linear number to a decibel number.
#[must_use]
pub fn linear_to_db(value: Number) -> Number {
    value.log10() * Number::Scalar(10.0)
}

/// A list of homogeneous numbers.
///
/// A homogeneous number list is a list of numbers that are all either
/// measured numbers with dimensionally equivalent units or all numbers.
pub enum HomogeneousNumberList<'a> {
    /// A list of numbers
    Numbers(Vec<&'a Number>),
    /// A list of measured numbers that are dimensionally equivalent
    MeasuredNumbers(Vec<&'a MeasuredNumber>),
}

/// Ensures that all of the numbers in the list are either all measured numbers
/// with dimensionally equivalent units or all numbers. The list must not be empty.
///
/// If the numbers are all measured numbers, then the function returns a list of
/// the numbers converted to the same unit (using `MeasuredNumber::into_number_using_unit`).
///
/// # Errors
///
/// Returns an eval error if:
/// - the units are not dimensionally equivalent
/// - the values are not numbers or measured numbers
/// - the values are not homogeneous (i.e. all measured numbers or all numbers)
///
/// # Panics
///
/// Panics if the list is empty.
#[expect(
    clippy::panic_in_result_fn,
    reason = "callers are expected to enforce the invariant that the list is not empty so that they can provide the correct error message"
)]
pub fn extract_homogeneous_numbers_list(
    values: &[(Value, Span)],
) -> Result<HomogeneousNumberList<'_>, Vec<EvalError>> {
    // Only used within this function
    enum ListResult<'a> {
        Numbers {
            numbers: Vec<&'a Number>,
            first_number_span: &'a Span,
        },
        MeasuredNumbers {
            numbers: Vec<&'a MeasuredNumber>,
            expected_unit: &'a Unit,
            expected_unit_value_span: &'a Span,
        },
    }

    assert!(!values.is_empty());

    let mut list_result: Option<ListResult<'_>> = None;
    let mut errors = Vec::new();

    for (value, value_span) in values {
        match value {
            Value::MeasuredNumber(number) => {
                match &mut list_result {
                    Some(ListResult::MeasuredNumbers {
                        numbers,
                        expected_unit,
                        expected_unit_value_span,
                    }) => {
                        if number.unit().dimensionally_eq(expected_unit) {
                            numbers.push(number);
                        } else {
                            errors.push(EvalError::UnitMismatch {
                                expected_unit: expected_unit.display_unit.clone(),
                                expected_source_span: **expected_unit_value_span,
                                found_unit: number.unit().display_unit.clone(),
                                found_span: *value_span,
                            });
                        }
                    }
                    Some(ListResult::Numbers {
                        numbers: _,
                        first_number_span,
                    }) => {
                        errors.push(EvalError::UnitMismatch {
                            expected_unit: DisplayUnit::Unitless,
                            expected_source_span: **first_number_span,
                            found_unit: number.unit().display_unit.clone(),
                            found_span: *value_span,
                        });
                    }
                    None => {
                        // the first argument is the expected output
                        list_result = Some(ListResult::MeasuredNumbers {
                            numbers: vec![number],
                            expected_unit: number.unit(),
                            expected_unit_value_span: value_span,
                        });
                    }
                }
            }
            Value::Number(number) => {
                match &mut list_result {
                    Some(ListResult::MeasuredNumbers {
                        numbers: _numbers,
                        expected_unit: expected_unit,
                        expected_unit_value_span: expected_unit_value_span,
                    }) => {
                        errors.push(EvalError::UnitMismatch {
                            expected_unit: expected_unit.display_unit.clone(),
                            expected_source_span: **expected_unit_value_span,
                            found_unit: DisplayUnit::Unitless,
                            found_span: *value_span,
                        });
                    }
                    Some(ListResult::Numbers {
                        numbers,
                        first_number_span: _,
                    }) => {
                        numbers.push(number);
                    }
                    None => {
                        // the first argument is the expected output
                        list_result = Some(ListResult::Numbers {
                            numbers: vec![&number],
                            first_number_span: value_span,
                        });
                    }
                }
            }
            Value::String(_) | Value::Boolean(_) => errors.push(EvalError::InvalidType {
                expected_type: ExpectedType::NumberOrMeasuredNumber,
                found_type: value.type_(),
                found_span: *value_span,
            }),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let list_result =
        list_result.expect("there should be at least one number, which means this must be set");

    let homogeneous_number_list = match list_result {
        ListResult::Numbers {
            numbers,
            first_number_span: _,
        } => HomogeneousNumberList::Numbers(numbers),
        ListResult::MeasuredNumbers {
            numbers,
            expected_unit: _,
            expected_unit_value_span: _,
        } => HomogeneousNumberList::MeasuredNumbers(numbers),
    };

    Ok(homogeneous_number_list)
}
