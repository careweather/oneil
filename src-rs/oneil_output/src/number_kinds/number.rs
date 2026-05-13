//! Number value type (scalar or interval).

use std::{fmt, ops};

use serde::{Deserialize, Serialize};

use crate::{
    Interval, NumberType,
    util::{DEFAULT_SIG_FIGS, float_to_string, is_close},
};

/// A number value in Oneil.
///
/// A number value is either a scalar or an interval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    /// A scalar number value.
    Scalar(f64),
    /// An interval number value.
    Interval(Interval),
}

impl Number {
    /// Creates a new scalar number value.
    #[must_use]
    pub const fn new_scalar(value: f64) -> Self {
        Self::Scalar(value)
    }

    /// Creates a new interval number value.
    #[must_use]
    pub fn new_interval(min: f64, max: f64) -> Self {
        Self::Interval(Interval::new(min, max))
    }

    /// Creates a new empty interval number value.
    #[must_use]
    pub const fn new_empty() -> Self {
        Self::Interval(Interval::empty())
    }

    /// Returns the type of the number value.
    #[must_use]
    pub const fn type_(&self) -> NumberType {
        match self {
            Self::Scalar(_) => NumberType::Scalar,
            Self::Interval(_) => NumberType::Interval,
        }
    }

    /// Returns the minimum value of the number value.
    ///
    /// For scalar values, this is simply the value itself.
    #[must_use]
    pub const fn min(&self) -> f64 {
        match self {
            Self::Scalar(value) => *value,
            Self::Interval(interval) => interval.min(),
        }
    }

    /// Returns the maximum value of the number value.
    ///
    /// For scalar values, this is simply the value itself.
    #[must_use]
    pub const fn max(&self) -> f64 {
        match self {
            Self::Scalar(value) => *value,
            Self::Interval(interval) => interval.max(),
        }
    }

    /// Subtracts two number values. This does not apply the
    /// standard rules of interval arithmetic. Instead, it subtracts the minimum
    /// from the minimum and the maximum from the maximum.
    #[must_use]
    pub fn escaped_sub(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs - rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).escaped_sub(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.escaped_sub(Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs.escaped_sub(rhs)),
        }
    }

    /// Divides two number values. This does not apply the
    /// standard rules of interval arithmetic. Instead, it divides the minimum
    /// by the minimum and the maximum by the maximum.
    #[must_use]
    pub fn escaped_div(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs / rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).escaped_div(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.escaped_div(Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs.escaped_div(rhs)),
        }
    }

    /// Raises a number value to the power of another.
    #[must_use]
    pub fn pow(self, exponent: Self) -> Self {
        match (self, exponent) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs.powf(rhs)),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).pow(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.pow(Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs.pow(rhs)),
        }
    }

    /// Returns the intersection of two number values.
    ///
    /// This operation converts both number values to intervals
    /// (if they are not already) and then returns the
    /// intersection of the two intervals.
    ///
    /// This operation always returns an interval number value.
    #[must_use]
    pub fn intersection(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => {
                Self::Interval(Interval::from(lhs).intersection(Interval::from(rhs)))
            }
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).intersection(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.intersection(Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs.intersection(rhs)),
        }
    }

    /// Returns the smallest interval that contains both number values.
    ///
    /// For example:
    ///
    /// ```oneil-eval-output
    /// |--- a ---|
    ///        |--- b ---|
    /// |---- result ----|
    /// ```
    ///
    /// ```oneil-eval-output
    ///              |--- a ---|
    /// |--- b ---|
    /// |------- result -------|
    /// ```
    ///
    /// This operation always returns an interval number value.
    #[must_use]
    pub fn tightest_enclosing_interval(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => {
                let min = f64::min(lhs, rhs);
                let max = f64::max(lhs, rhs);
                Self::Interval(Interval::new(min, max))
            }
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).tightest_enclosing_interval(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.tightest_enclosing_interval(Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => {
                Self::Interval(lhs.tightest_enclosing_interval(rhs))
            }
        }
    }

    /// Checks if `self` contains another number value.
    ///
    /// If `self` is a scalar, then the other value must be equal to `self`.
    ///
    /// If `self` is an interval, then the other value must be contained in `self`.
    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => is_close(*lhs, *rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                is_close(*lhs, rhs.min()) && is_close(*lhs, rhs.max())
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => *rhs >= lhs.min() && *rhs <= lhs.max(),
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.contains(rhs),
        }
    }

    /// Returns the square root of the number value.
    #[must_use]
    pub fn sqrt(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.sqrt()),
            Self::Interval(interval) => Self::Interval(interval.sqrt()),
        }
    }

    /// Returns the sine of the number value (angle in radians).
    #[must_use]
    pub fn sin(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.sin()),
            Self::Interval(interval) => Self::Interval(interval.sin()),
        }
    }

    /// Returns the cosine of the number value (angle in radians).
    #[must_use]
    pub fn cos(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.cos()),
            Self::Interval(interval) => Self::Interval(interval.cos()),
        }
    }

    /// Returns the tangent of the number value (angle in radians).
    #[must_use]
    pub fn tan(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.tan()),
            Self::Interval(interval) => Self::Interval(interval.tan()),
        }
    }

    /// Returns the arc sine of the number value (result in radians).
    #[must_use]
    pub fn asin(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.asin()),
            Self::Interval(interval) => Self::Interval(interval.asin()),
        }
    }

    /// Returns the arc cosine of the number value (result in radians).
    #[must_use]
    pub fn acos(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.acos()),
            Self::Interval(interval) => Self::Interval(interval.acos()),
        }
    }

    /// Returns the arc tangent of the number value (result in radians).
    #[must_use]
    pub fn atan(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.atan()),
            Self::Interval(interval) => Self::Interval(interval.atan()),
        }
    }

    /// Returns the natural logarithm of the number value.
    #[must_use]
    pub fn ln(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.ln()),
            Self::Interval(interval) => Self::Interval(interval.ln()),
        }
    }

    /// Returns the base 10 logarithm of the number value.
    #[must_use]
    pub fn log10(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.log10()),
            Self::Interval(interval) => Self::Interval(interval.log10()),
        }
    }

    /// Returns the base 2 logarithm of the number value.
    #[must_use]
    pub fn log2(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.log2()),
            Self::Interval(interval) => Self::Interval(interval.log2()),
        }
    }

    /// Returns the absolute value of the number.
    #[must_use]
    pub fn abs(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.abs()),
            Self::Interval(interval) => Self::Interval(interval.abs()),
        }
    }

    /// Returns the sign of the number: -1 for negative, 0 for zero, or 1 for positive.
    ///
    /// For an interval, returns the tightest interval containing the possible sign values.
    #[must_use]
    pub fn sign(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.signum()),
            Self::Interval(interval) => Self::Interval(interval.sign()),
        }
    }

    /// Returns the number rounded down to the nearest integer.
    #[must_use]
    pub fn floor(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.floor()),
            Self::Interval(interval) => Self::Interval(interval.floor()),
        }
    }

    /// Returns the number rounded up to the nearest integer.
    #[must_use]
    pub fn ceiling(self) -> Self {
        match self {
            Self::Scalar(value) => Self::Scalar(value.ceil()),
            Self::Interval(interval) => Self::Interval(interval.ceiling()),
        }
    }

    /// Returns true if `self` is strictly less than `rhs`.
    ///
    /// For scalars, uses the built-in `<` operator. For intervals, delegates to [`Interval::lt`].
    #[must_use]
    pub fn lt(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs < rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => *lhs < rhs.min(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => lhs.max() < *rhs,
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.lt(rhs),
        }
    }

    /// Returns true if `self` is strictly greater than `rhs`.
    ///
    /// For scalars, uses the built-in `>` operator. For intervals, delegates to [`Interval::gt`].
    #[must_use]
    pub fn gt(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs > rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => *lhs > rhs.max(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => lhs.min() > *rhs,
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.gt(rhs),
        }
    }

    /// Returns true if `self` is less than or equal to `rhs`.
    ///
    /// For scalars, uses the built-in `<=` operator. For intervals, delegates to [`Interval::lte`].
    ///
    /// Note that this must be implemented seperately from `lt` because `lte`
    /// differs from `lt` for intervals. See [`Interval::lte`] for more details.
    #[must_use]
    pub fn lte(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs <= rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => *lhs <= rhs.min(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => lhs.max() <= *rhs,
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.lte(rhs),
        }
    }

    /// Returns true if `self` is greater than or equal to `rhs`.
    ///
    /// For scalars, uses the built-in `>=` operator. For intervals, delegates to [`Interval::gte`].
    ///
    /// Note that this must be implemented seperately from `gt` because `gte`
    /// differs from `gt` for intervals. See [`Interval::gte`] for more details.
    #[must_use]
    pub fn gte(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs >= rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => *lhs >= rhs.max(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => lhs.min() >= *rhs,
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.gte(rhs),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => is_close(*lhs, *rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                is_close(*lhs, rhs.min()) && is_close(*lhs, rhs.max())
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                is_close(lhs.min(), *rhs) && is_close(lhs.max(), *rhs)
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.eq(rhs),
        }
    }
}

impl ops::Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(-value),
            Self::Interval(interval) => Self::Interval(-interval),
        }
    }
}

impl ops::Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs + rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => Self::Interval(Interval::from(lhs) + rhs),
            (Self::Interval(lhs), Self::Scalar(rhs)) => Self::Interval(lhs + Interval::from(rhs)),
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs + rhs),
        }
    }
}

impl ops::Sub for Number {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs - rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => Self::Interval(Interval::from(lhs) - rhs),
            (Self::Interval(lhs), Self::Scalar(rhs)) => Self::Interval(lhs - Interval::from(rhs)),
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs - rhs),
        }
    }
}

impl ops::Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs * rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => Self::Interval(Interval::from(lhs) * rhs),
            (Self::Interval(lhs), Self::Scalar(rhs)) => Self::Interval(lhs * Interval::from(rhs)),
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs * rhs),
        }
    }
}

impl ops::Mul<f64> for Number {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(value * rhs),
            Self::Interval(interval) => Self::Interval(interval * Interval::from(rhs)),
        }
    }
}

impl ops::Div for Number {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs / rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => Self::Interval(Interval::from(lhs) / rhs),
            (Self::Interval(lhs), Self::Scalar(rhs)) => Self::Interval(lhs / Interval::from(rhs)),
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs / rhs),
        }
    }
}

impl ops::Div<f64> for Number {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(value / rhs),
            Self::Interval(interval) => Self::Interval(interval / Interval::from(rhs)),
        }
    }
}

impl ops::Rem for Number {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs % rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => Self::Interval(Interval::from(lhs) % rhs),
            (Self::Interval(lhs), Self::Scalar(rhs)) => Self::Interval(lhs % rhs), // use the specialized version of the modulo operation
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs % rhs),
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scalar(value) => {
                let value = float_to_string(*value, DEFAULT_SIG_FIGS);
                write!(f, "{value}")
            }
            Self::Interval(interval) => {
                let min = float_to_string(interval.min(), DEFAULT_SIG_FIGS);
                let max = float_to_string(interval.max(), DEFAULT_SIG_FIGS);
                write!(f, "{min} | {max}")
            }
        }
    }
}
