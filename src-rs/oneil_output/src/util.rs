//! Utility functions for the value module.

use crate::{MeasuredNumber, Number};

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
    /// A list of numbers.
    Numbers(Vec<&'a Number>),
    /// A list of measured numbers that are dimensionally equivalent.
    MeasuredNumbers(Vec<&'a MeasuredNumber>),
}
