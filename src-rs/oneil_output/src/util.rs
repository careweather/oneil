//! Utility functions for the value module.

use crate::Number;

/// The default number of significant figures to use when displaying a number.
///
/// This is used when displaying a number in a string format.
pub const DEFAULT_SIG_FIGS: usize = 4;

const DEFAULT_RELATIVE_TOLERANCE: f64 = 1e-15;
const DEFAULT_ABSOLUTE_TOLERANCE: f64 = 32.0 * f64::MIN_POSITIVE;

/// Checks if two floating point numbers are close to each other using
/// a default relative tolerance of `1e-15` and a default absolute tolerance of
/// `32.0 * f64::MIN_POSITIVE`.
#[must_use]
pub const fn is_close(a: f64, b: f64) -> bool {
    is_close_with_tolerances(a, b, DEFAULT_RELATIVE_TOLERANCE, DEFAULT_ABSOLUTE_TOLERANCE)
}

/// Checks if two floating point numbers are close to each other.
///
/// This function uses the `Strong` comparison method defined in the
/// `is_close` crate as reference. See
/// <https://github.com/PM4Rs/is_close/blob/8475cd292946b6e5461375a41160153ce32e31c6/src/lib.rs#L183>
/// for more details.
///
/// In the future, we may want to implement other methods
/// from the `is_close` crate.
#[must_use]
pub const fn is_close_with_tolerances(
    a: f64,
    b: f64,
    relative_tolerance: f64,
    absolute_tolerance: f64,
) -> bool {
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
    let relative_tolerance = relative_tolerance * f64::min(a.abs(), b.abs());

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
    Number::Scalar(10.0) * value.log10()
}

/// Converts a floating point number to a string
///
/// Mimics `%g` as described here: <https://stackoverflow.com/a/54162153>
///
/// Built by referencing `prettyfloat.rs` from the `sad-monte-carlo` crate:
/// <https://github.com/droundy/sad-monte-carlo/blob/master/src/prettyfloat.rs>
#[must_use]
pub fn float_to_string(value: f64, sig_figs: usize) -> String {
    if value.is_infinite() || value.is_nan() {
        return value.to_string();
    }

    let standard_precision = number_of_digits_after_decimal(value, sig_figs);
    let scientific_precision = sig_figs - 1;

    let standard_float = trim_trailing_zeros(format!("{value:.standard_precision$}"));
    let scientific_float = trim_trailing_zeros(format!("{value:.scientific_precision$e}"));

    if standard_float.len() <= scientific_float.len() {
        standard_float
    } else {
        scientific_float
    }
}

/// Returns the number of digits after the decimal point that
/// are significant.
fn number_of_digits_after_decimal(value: f64, sig_figs: usize) -> usize {
    // the value `0` has no sig figs
    if value == 0.0 {
        return 0;
    }

    // get the place value of the first significant digit
    let log10_value = value.abs().log10();

    #[expect(
        clippy::cast_precision_loss,
        reason = "although `sig_figs` is a `usize`, we will realistically never have a value that causes precision loss"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "we confirm that `sig_figs` is greater than or equal to `log10_value`"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "we don't care about the fractional part"
    )]
    if log10_value > sig_figs as f64 {
        // if the place value of the first significant digit is greater than the number of sig figs,
        // then there are no digits after the decimal point that are significant.
        0
    } else {
        (sig_figs as f64 - log10_value) as usize
    }
}

/// Trims the trailing zeros from a float string.
fn trim_trailing_zeros(value: String) -> String {
    // if there isn't a decimal point,
    // then trailing zeros can't be trimmed
    if !value.contains('.') {
        return value;
    }

    if let Some((base, exponent)) = value.split_once('e') {
        // handle scientific notation

        // remove trailing zeros from the base and trailing '.'
        // from the base if it exists
        let base = base.trim_end_matches('0').trim_end_matches('.');

        format!("{base}e{exponent}")
    } else {
        // handle regular floating point notation
        value
            // remove trailing zeros and '.' if they exist
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
