use std::{cmp::Ordering, ops};

use crate::value::{EvalError, Interval, NumberType, Unit, util::is_close};

/// A number with a unit
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredNumber {
    /// The value of the measured number.
    pub value: Number,
    /// The unit of the measured number.
    pub unit: Unit,
}

impl MeasuredNumber {
    #[must_use]
    /// Creates a new measured number.
    pub const fn new(value: Number, unit: Unit) -> Self {
        Self { value, unit }
    }

    /// Compares two measured numbers for ordering.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the units don't match.
    pub fn checked_partial_cmp(&self, rhs: &Self) -> Result<Option<Ordering>, EvalError> {
        if self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        Ok(self.value.partial_cmp(&rhs.value))
    }

    /// Checks if two dimensional numbers are equal.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, EvalError> {
        self.checked_partial_cmp(rhs)
            .map(|ordering| ordering == Some(Ordering::Equal))
    }

    /// Adds two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_add(self, rhs: &Self) -> Result<Self, EvalError> {
        if self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            value: self.value + rhs.value,
            unit: self.unit,
        })
    }

    /// Subtracts two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_sub(self, rhs: &Self) -> Result<Self, EvalError> {
        if self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            value: self.value - rhs.value,
            unit: self.unit,
        })
    }

    /// Subtracts two dimensional numbers. This does not apply the
    /// standard rules of interval arithmetic. Instead, it subtracts the minimum
    /// from the minimum and the maximum from the maximum.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_escaped_sub(self, rhs: &Self) -> Result<Self, EvalError> {
        if self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            value: self.value.escaped_sub(rhs.value),
            unit: self.unit,
        })
    }

    /// Multiplies two dimensional numbers.
    ///
    /// # Errors
    ///
    /// This function never returns an error.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, EvalError> {
        Ok(Self {
            value: self.value * rhs.value,
            unit: self.unit * rhs.unit,
        })
    }

    /// Divides two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_div(self, rhs: Self) -> Result<Self, EvalError> {
        Ok(Self {
            value: self.value / rhs.value,
            unit: self.unit / rhs.unit,
        })
    }

    /// Divides two dimensional numbers. This does not apply the
    /// standard rules of interval arithmetic. Instead, it divides the minimum
    /// by the minimum and the maximum by the maximum.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_escaped_div(self, rhs: &Self) -> Result<Self, EvalError> {
        if self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            value: self.value.escaped_div(rhs.value),
            unit: self.unit,
        })
    }

    /// Computes the remainder of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_rem(self, rhs: &Self) -> Result<Self, EvalError> {
        if self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            value: self.value % rhs.value,
            unit: self.unit,
        })
    }

    /// Raises a dimensional number to the power of another.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_pow(self, exponent: &Self) -> Result<Self, EvalError> {
        if !exponent.unit.is_unitless() {
            return Err(EvalError::HasExponentWithUnits);
        }

        match exponent.value {
            Number::Scalar(exponent_value) => Ok(Self {
                value: self.value.pow(Number::Scalar(exponent_value)),
                unit: self.unit.pow(exponent_value),
            }),
            Number::Interval(_) => Err(EvalError::HasIntervalExponent),
        }
    }

    /// Returns the tightest enclosing interval of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_min_max(self, rhs: &Self) -> Result<Self, EvalError> {
        // check that the units match (or are unitless)
        if !self.unit.is_unitless() && !rhs.unit.is_unitless() && self.unit != rhs.unit {
            return Err(EvalError::InvalidUnit);
        }

        // if the left unit is unitless, use the right unit, otherwise use the left unit
        let unit = if self.unit.is_unitless() {
            rhs.unit.clone()
        } else {
            self.unit
        };

        Ok(Self {
            value: self.value.tightest_enclosing_interval(rhs.value),
            unit,
        })
    }

    /// Negates a number value.
    #[must_use]
    pub fn checked_neg(self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }
}

/// A number value in Oneil.
///
/// A number value is either a scalar or an interval.
#[derive(Debug, Clone, Copy)]
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
    /// ```text
    /// |--- a ---|
    ///        |--- b ---|
    /// |---- result ----|
    /// ```
    ///
    /// ```text
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
    pub fn contains(self, rhs: Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => is_close(lhs, rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                is_close(lhs, rhs.min()) && is_close(lhs, rhs.max())
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => rhs >= lhs.min() && rhs <= lhs.max(),
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.contains(rhs),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => is_close(*lhs, *rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                is_close(*lhs, rhs.min()) && is_close(*lhs, rhs.max())
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                is_close(lhs.min(), *rhs) && is_close(lhs.max(), *rhs)
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs == rhs,
        }
    }
}

impl PartialOrd for Number {
    /// Partial ordering for number values
    ///
    /// For scalar values, we use the partial ordering of f64.
    ///
    /// An interval is less than a scalar if both the min and max are less than the
    /// scalar. Same goes for greater than and equal to.
    ///
    /// An interval is less than another interval if both the min and max are less
    /// than the other interval. Same goes for greater than and equal to.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs.partial_cmp(rhs),
            (Self::Scalar(lhs), Self::Interval(rhs)) => Interval::from(lhs).partial_cmp(rhs),
            (Self::Interval(lhs), Self::Scalar(rhs)) => lhs.partial_cmp(&Interval::from(rhs)),
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs.partial_cmp(rhs),
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
