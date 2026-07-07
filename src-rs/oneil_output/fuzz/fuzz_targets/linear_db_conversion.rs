#![no_main]

use libfuzzer_sys::{arbitrary, fuzz_target};
use oneil_output::{
    Interval, Number,
    util::{db_to_linear, is_close_with_tolerances, linear_to_db},
};

const ABSOLUTE_TOLERANCE: f64 = 1e-15;
const RELATIVE_TOLERANCE: f64 = 1e-2;
const LARGE_NUMBER_THRESHOLD: f64 = 1e300;

/// Asserts that two floating point numbers are close within fuzz tolerances.
///
/// # Panics
///
/// Panics if the values are not close.
#[track_caller]
fn assert_is_close(expected: f64, actual: f64) {
    assert!(
        is_close_with_tolerances(expected, actual, RELATIVE_TOLERANCE, ABSOLUTE_TOLERANCE),
        "expected: {expected}, actual: {actual}"
    );
}

#[expect(
    clippy::enum_variant_names,
    reason = "it makes it more clear in the test"
)]
#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    StartFromLinearInterval { value: Interval },
    StartFromDbInterval { value: Interval },
    StartFromLinearScalar { value: f64 },
    StartFromDbScalar { value: f64 },
}

fuzz_target!(|data: FuzzData| {
    match data {
        FuzzData::StartFromLinearInterval { value } => {
            if value.min() <= 0.0 {
                return;
            }

            let to_db = linear_to_db(Number::Interval(value));
            let back_to_linear = db_to_linear(to_db);

            let Number::Interval(back_to_linear) = back_to_linear else {
                panic!("expected interval");
            };

            if value.is_empty() && back_to_linear.is_empty() {
                return;
            }

            if !(value.min().abs() > LARGE_NUMBER_THRESHOLD && back_to_linear.min().is_infinite()) {
                assert_is_close(value.min(), back_to_linear.min());
            }

            if !(value.max().abs() > LARGE_NUMBER_THRESHOLD && back_to_linear.max().is_infinite()) {
                assert_is_close(value.max(), back_to_linear.max());
            }
        }
        FuzzData::StartFromDbInterval { value } => {
            let min_linear = 10_f64.powf(value.min() / 10.0);
            if min_linear <= 0.0 || value.max() >= f64::MAX.log10() * 10.0 {
                // this will overflow to infinity
                return;
            }

            let linear = db_to_linear(Number::Interval(value));
            let back_to_db = linear_to_db(linear);

            let Number::Interval(back_to_db) = back_to_db else {
                panic!("expected interval");
            };

            if value.is_empty() && back_to_db.is_empty() {
                return;
            }

            assert_is_close(value.min(), back_to_db.min());
            assert_is_close(value.max(), back_to_db.max());
        }
        FuzzData::StartFromLinearScalar { value } => {
            if value.is_nan() || value <= 0.0 {
                return;
            }

            let to_db = linear_to_db(Number::Scalar(value));
            let back_to_linear = db_to_linear(to_db);

            let Number::Scalar(back_to_linear) = back_to_linear else {
                panic!("expected scalar");
            };

            if !(value.abs() > LARGE_NUMBER_THRESHOLD && back_to_linear.is_infinite()) {
                assert_is_close(value, back_to_linear);
            }
        }
        FuzzData::StartFromDbScalar { value } => {
            let min_linear = 10_f64.powf(value / 10.0);
            if value.is_nan() || min_linear <= 0.0 || value >= f64::MAX.log10() * 10.0 {
                // this is NaN or will overflow to infinity
                return;
            }

            let linear = db_to_linear(Number::Scalar(value));
            let back_to_db = linear_to_db(linear);

            let Number::Scalar(back_to_db) = back_to_db else {
                panic!("expected scalar");
            };

            assert_is_close(value, back_to_db);
        }
    }
});
