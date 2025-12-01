use std::{cmp::Ordering, ops};

use crate::value::{Interval, Unit, ValueError};

#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredNumber {
    pub value: Number,
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
    pub fn checked_partial_cmp(&self, rhs: &Self) -> Result<Option<Ordering>, ValueError> {
        if self.unit != rhs.unit {
            return Err(ValueError::InvalidUnit);
        }

        Ok(self.value.partial_cmp(&rhs.value))
    }

    /// Checks if two dimensional numbers are equal.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, ValueError> {
        self.checked_partial_cmp(rhs)
            .map(|ordering| ordering == Some(Ordering::Equal))
    }

    /// Adds two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_add(self, rhs: &Self) -> Result<Self, ValueError> {
        if self.unit != rhs.unit {
            return Err(ValueError::InvalidUnit);
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
    pub fn checked_sub(self, rhs: &Self) -> Result<Self, ValueError> {
        if self.unit != rhs.unit {
            return Err(ValueError::InvalidUnit);
        }

        Ok(Self {
            value: self.value - rhs.value,
            unit: self.unit,
        })
    }

    /// Multiplies two dimensional numbers.
    ///
    /// # Errors
    ///
    /// This function never returns an error.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, ValueError> {
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
    pub fn checked_div(self, rhs: Self) -> Result<Self, ValueError> {
        Ok(Self {
            value: self.value / rhs.value,
            unit: self.unit / rhs.unit,
        })
    }

    /// Computes the remainder of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_rem(self, rhs: &Self) -> Result<Self, ValueError> {
        if self.unit != rhs.unit {
            return Err(ValueError::InvalidUnit);
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
    pub fn checked_pow(self, rhs: &Self) -> Result<Self, ValueError> {
        if !self.unit.is_unitless() {
            return Err(ValueError::HasExponentWithUnits);
        }

        match rhs.value {
            Number::Scalar(exponent) => Ok(Self {
                value: self.value.pow(rhs.value),
                unit: self.unit.pow(exponent),
            }),
            Number::Interval(_) => Err(ValueError::HasIntervalExponent),
        }
    }

    /// Returns the tightest enclosing interval of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_min_max(self, rhs: &Self) -> Result<Self, ValueError> {
        if self.unit != rhs.unit {
            return Err(ValueError::InvalidUnit);
        }

        Ok(Self {
            value: self.value.tightest_enclosing_interval(rhs.value),
            unit: self.unit,
        })
    }

    /// Negates a number value.
    pub fn checked_neg(self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }
}

// TODO: in number value docs mention that for the outside world,
//       a number value is essentially an interval. The fact that
//       it is sometimes stored as a scalar is an implementation detail.
#[derive(Debug, Clone, Copy)]
pub enum Number {
    Scalar(f64),
    Interval(Interval),
}

impl Number {
    #[must_use]
    pub const fn new_scalar(value: f64) -> Self {
        Self::Scalar(value)
    }

    #[must_use]
    pub fn new_interval(min: f64, max: f64) -> Self {
        Self::Interval(Interval::new(min, max))
    }

    #[must_use]
    pub const fn new_empty() -> Self {
        Self::Interval(Interval::empty())
    }

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

    #[must_use]
    pub fn inside(self, rhs: Self) -> bool {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs == rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => lhs >= rhs.min() && lhs <= rhs.max(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => lhs.min() == rhs && lhs.max() == rhs,
            (Self::Interval(lhs), Self::Interval(rhs)) => rhs.contains(lhs),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs == rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => lhs == &rhs.min() && lhs == &rhs.max(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => &lhs.min() == rhs && &lhs.max() == rhs,
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
