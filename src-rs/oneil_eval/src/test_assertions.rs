//! Shared assertion helpers for evaluator tests.

use std::collections::BTreeMap;

use oneil_output::{Dimension, DimensionMap, Unit};
use oneil_output::util::is_close;

/// Asserts that two floating point numbers are close to each other.
///
/// # Panics
///
/// Panics if the values are not close.
///
/// ```
/// use oneil_eval::assert_is_close;
///
/// assert_is_close(0.1 + 0.2, 0.3);
/// ```
#[track_caller]
pub fn assert_is_close(expected: f64, actual: f64) {
    assert!(
        is_close(expected, actual),
        "expected: {expected}, actual: {actual}"
    );
}

/// Asserts that two units are dimensionally equal.
///
/// # Panics
///
/// Panics if the dimension maps do not match.
///
/// ```
/// # use std::collections::BTreeMap;
/// # use oneil_eval::assert_units_dimensionally_eq;
/// # use oneil_output::{Dimension, DimensionMap, DisplayUnit, Unit};
/// let unit = Unit {
///     dimension_map: DimensionMap::new(BTreeMap::from([(Dimension::Time, 1.0)])),
///     magnitude: 1.0,
///     is_db: false,
///     display_unit: DisplayUnit::One,
/// };
/// assert_units_dimensionally_eq([(Dimension::Time, 1.0)], &unit);
/// ```
#[track_caller]
pub fn assert_units_dimensionally_eq(
    expected_unit_list: impl IntoIterator<Item = (Dimension, f64)>,
    actual_unit: &Unit,
) {
    let expected = DimensionMap::new(BTreeMap::from_iter(expected_unit_list));
    assert_eq!(expected, actual_unit.dimension_map);
}
