use std::{cmp::Ordering, ops};

use crate::value::{
    EvalError, Interval, NumberType, Unit,
    util::{db_to_linear, is_close, linear_to_db},
};

// TODO: document how this guarantees that the number is
//       correctly constructed

/// A number with a unit
///
/// The only way to construct a `MeasuredNumber` is to use `from_number_and_unit`,
/// which performs adjustment based on the magnitude/dB of the unit.
#[derive(Debug, Clone)]
pub struct MeasuredNumber {
    /// The value of the measured number.
    ///
    /// Note that this is stored in a "normalized" form,
    /// where the magnitude/dB of the unit is already applied.
    ///
    /// See `from_number_and_unit` for more details.
    normalized_value: NormalizedNumber,
    /// The unit of the measured number.
    unit: Unit,
}

impl MeasuredNumber {
    /// Converts a number and a unit to a measured number.
    ///
    /// Note that this performs adjustment based on the magnitude/dB of the unit.
    ///
    /// A measured number stores the number in a "normalized" form,
    /// where the magnitude/dB of the unit is already applied.
    ///
    /// For example, `value = 1` and `unit = km` will result in `1000 m`.
    #[must_use]
    pub fn from_number_and_unit(value: Number, unit: Unit) -> Self {
        let normalized_value = NormalizedNumber::from_number_and_unit(value, &unit);
        Self {
            normalized_value,
            unit,
        }
    }

    /// Converts a measured number to a number and a unit.
    ///
    /// This is the inverse of `from_number_and_unit`. See that function
    /// for a description of the conversion.
    #[must_use]
    pub fn into_number_and_unit(self) -> (Number, Unit) {
        let Self {
            normalized_value,
            unit,
        } = self;

        let value = normalized_value.into_number_using_unit(&unit);

        (value, unit)
    }

    /// Converts a measured number to a number based on the given unit.
    ///
    /// This performs adjustment based on the magnitude/dB of the unit.
    ///
    /// See `from_number_and_unit` for more details.
    #[must_use]
    pub fn into_number_using_unit(self, unit: &Unit) -> Number {
        self.normalized_value.into_number_using_unit(unit)
    }

    /// Updates the unit of the measured number.
    ///
    ///
    /// # Panics
    ///
    /// This panics if the new unit is not dimensionally equivalent
    /// to the old unit.
    #[must_use]
    pub fn with_unit(self, unit: Unit) -> Self {
        debug_assert!(
            !self.unit.dimensionally_eq(&unit),
            "old unit {} is not dimensionally equivalent to new unit {}",
            self.unit.display_unit,
            unit.display_unit,
        );

        Self { unit, ..self }
    }

    /// Returns the normalized value of the measured number.
    ///
    /// The normalized value can be compared with other normalized
    /// values, regardless of unit magnitude.
    ///
    /// For example, `1000 m` and `1 km` are normalized to the
    /// same value.
    #[must_use]
    pub const fn normalized_value(&self) -> &NormalizedNumber {
        &self.normalized_value
    }

    /// Returns the unit of the measured number.
    #[must_use]
    pub const fn unit(&self) -> &Unit {
        &self.unit
    }

    /// Compares two measured numbers for ordering.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the units don't match.
    pub fn checked_partial_cmp(&self, rhs: &Self) -> Result<Option<Ordering>, EvalError> {
        if !self.unit.dimensionally_eq(&rhs.unit) {
            return Err(EvalError::InvalidUnit);
        }

        Ok(self.normalized_value.partial_cmp(&rhs.normalized_value))
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
        if !self.unit.dimensionally_eq(&rhs.unit) {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            normalized_value: self.normalized_value + rhs.normalized_value,
            unit: self.unit,
        })
    }

    /// Subtracts two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_sub(self, rhs: &Self) -> Result<Self, EvalError> {
        if !self.unit.dimensionally_eq(&rhs.unit) {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            normalized_value: self.normalized_value - rhs.normalized_value,
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
        if !self.unit.dimensionally_eq(&rhs.unit) {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            normalized_value: self.normalized_value.escaped_sub(rhs.normalized_value),
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
            normalized_value: self.normalized_value * rhs.normalized_value,
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
            normalized_value: self.normalized_value / rhs.normalized_value,
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
    pub fn checked_escaped_div(self, rhs: Self) -> Result<Self, EvalError> {
        Ok(Self {
            normalized_value: self.normalized_value.escaped_div(rhs.normalized_value),
            unit: self.unit / rhs.unit,
        })
    }

    /// Computes the remainder of two dimensional numbers.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_rem(self, rhs: &Self) -> Result<Self, EvalError> {
        if !self.unit.dimensionally_eq(&rhs.unit) {
            return Err(EvalError::InvalidUnit);
        }

        Ok(Self {
            normalized_value: self.normalized_value % rhs.normalized_value,
            unit: self.unit,
        })
    }

    /// Raises a dimensional number to the power of another.
    ///
    /// # Errors
    ///
    /// Returns `Err(ValueError::InvalidUnit)` if the dimensions don't match.
    pub fn checked_pow(self, exponent: &Number) -> Result<Self, EvalError> {
        match exponent {
            Number::Scalar(exponent_value) => Ok(Self {
                normalized_value: self.normalized_value.pow(Number::Scalar(*exponent_value)),
                unit: self.unit.pow(*exponent_value),
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
        if !self.unit.dimensionally_eq(&rhs.unit) {
            return Err(EvalError::InvalidUnit);
        }

        // if the left unit is unitless, use the right unit, otherwise use the left unit
        let unit = if self.unit.is_unitless() {
            rhs.unit.clone()
        } else {
            self.unit
        };

        Ok(Self {
            normalized_value: self
                .normalized_value
                .tightest_enclosing_interval(rhs.normalized_value),
            unit,
        })
    }

    /// Negates a number value.
    #[must_use]
    pub fn checked_neg(self) -> Self {
        Self {
            normalized_value: -self.normalized_value,
            unit: self.unit,
        }
    }

    /// Performs a min/max operation on a measured number and a regular number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    ///
    /// This operation is associative.
    #[must_use]
    pub fn min_max_number(self, rhs: Number) -> Self {
        Self {
            normalized_value: self
                .normalized_value
                .tightest_enclosing_interval_number(rhs),
            unit: self.unit,
        }
    }

    /// Returns the minimum value of the measured number.
    #[must_use]
    pub fn min(&self) -> Self {
        Self {
            normalized_value: NormalizedNumber(Number::Scalar(self.normalized_value.min())),
            unit: self.unit.clone(),
        }
    }

    /// Returns the maximum value of the measured number.
    #[must_use]
    pub fn max(&self) -> Self {
        Self {
            normalized_value: NormalizedNumber(Number::Scalar(self.normalized_value.max())),
            unit: self.unit.clone(),
        }
    }
}

impl PartialEq for MeasuredNumber {
    /// Compares two measured numbers for equality.
    ///
    /// This treats units as equal if they have the same dimensions.
    fn eq(&self, other: &Self) -> bool {
        self.normalized_value == other.normalized_value && self.unit.dimensionally_eq(&other.unit)
    }
}

impl ops::Add<Number> for MeasuredNumber {
    type Output = Self;

    /// Add a measured number to a number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    fn add(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value + rhs,
            unit: self.unit,
        }
    }
}

impl ops::Add<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Add a number to a measured number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    fn add(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self + rhs.normalized_value,
            unit: rhs.unit,
        }
    }
}

impl ops::Sub<Number> for MeasuredNumber {
    type Output = Self;

    /// Subtract a measured number from a number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    fn sub(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value - rhs,
            unit: self.unit,
        }
    }
}

impl ops::Sub<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Subtract a number from a measured number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    fn sub(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self - rhs.normalized_value,
            unit: rhs.unit,
        }
    }
}

impl ops::Mul<Number> for MeasuredNumber {
    type Output = Self;

    /// Multiply a measured number by a number.
    ///
    /// The `Number` is implicitly coerced to a unitless
    /// measured number.
    fn mul(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value * rhs,
            unit: self.unit,
        }
    }
}

impl ops::Mul<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Multiply a number by a measured number.
    ///
    /// The `Number` is implicitly coerced to a unitless
    /// measured number.
    fn mul(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self * rhs.normalized_value,
            unit: rhs.unit,
        }
    }
}

impl ops::Div<Number> for MeasuredNumber {
    type Output = Self;

    /// Divide a measured number by a number.
    ///
    /// The `Number` is implicitly coerced to a unitless
    /// measured number.
    fn div(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value / rhs,
            unit: self.unit,
        }
    }
}

impl ops::Div<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Divide a number by a measured number.
    ///
    /// The `Number` is implicitly coerced to a unitless
    /// measured number.
    fn div(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self / rhs.normalized_value,
            unit: Unit::unitless() / rhs.unit,
        }
    }
}

impl ops::Rem<Number> for MeasuredNumber {
    type Output = Self;

    /// Computes the remainder of dividing a measured number by a number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    fn rem(self, rhs: Number) -> Self::Output {
        Self {
            normalized_value: self.normalized_value % rhs,
            unit: self.unit,
        }
    }
}

impl ops::Rem<MeasuredNumber> for Number {
    type Output = MeasuredNumber;

    /// Computes the remainder of dividing a number by a measured number.
    ///
    /// The `Number` is implicitly coerced to a measured
    /// number with the same unit as the `MeasuredNumber`.
    fn rem(self, rhs: MeasuredNumber) -> Self::Output {
        MeasuredNumber {
            normalized_value: self % rhs.normalized_value,
            unit: rhs.unit,
        }
    }
}

/// A normalized number value.
///
/// This is a wrapper around a `Number` that has been normalized
/// based on the magnitude/dB of the unit.
///
/// The main purpose of this wrapper is to ensure that the normalized
/// number is not used when a number value is expected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedNumber(Number);

impl NormalizedNumber {
    #[must_use]
    pub fn from_number_and_unit(value: Number, unit: &Unit) -> Self {
        // adjust the magnitude based on the unit
        let value = value * unit.magnitude;

        // convert the number from a logarithmic unit
        // to a linear unit if it is a dB unit
        let value = if unit.is_db {
            db_to_linear(value)
        } else {
            value
        };

        Self(value)
    }

    #[must_use]
    pub fn into_number_using_unit(self, unit: &Unit) -> Number {
        let Self(value) = self;

        // convert the number from a linear unit
        // to a logarithmic unit if it is a dB unit
        let value = if unit.is_db {
            linear_to_db(value)
        } else {
            value
        };

        // adjust the magnitude based on the unit

        value / unit.magnitude
    }

    /// Subtracts two normalized numbers. This does not apply the
    /// standard rules of interval arithmetic. Instead, it subtracts the minimum
    /// from the minimum and the maximum from the maximum.
    #[must_use]
    pub fn escaped_sub(self, rhs: Self) -> Self {
        Self(self.0.escaped_sub(rhs.0))
    }

    /// Divides two normalized numbers. This does not apply the
    /// standard rules of interval arithmetic. Instead, it divides the minimum
    /// by the minimum and the maximum by the maximum.
    #[must_use]
    pub fn escaped_div(self, rhs: Self) -> Self {
        Self(self.0.escaped_div(rhs.0))
    }

    /// Raises a normalized number to the power of another.
    #[must_use]
    pub fn pow(self, exponent: Number) -> Self {
        Self(self.0.pow(exponent))
    }

    /// Returns the smallest interval that contains both normalized numbers.
    #[must_use]
    pub fn tightest_enclosing_interval(self, rhs: Self) -> Self {
        Self(self.0.tightest_enclosing_interval(rhs.0))
    }

    /// Returns the smallest interval that contains both a normalized number and a number.
    #[must_use]
    pub fn tightest_enclosing_interval_number(self, rhs: Number) -> Self {
        Self(self.0.tightest_enclosing_interval(rhs))
    }

    /// Returns true if the normalized number contains the other normalized number.
    #[must_use]
    pub fn contains(&self, rhs: &Self) -> bool {
        self.0.contains(&rhs.0)
    }

    /// Returns the minimum value of the normalized number.
    #[must_use]
    pub const fn min(&self) -> f64 {
        self.0.min()
    }

    /// Returns the maximum value of the normalized number.
    #[must_use]
    pub const fn max(&self) -> f64 {
        self.0.max()
    }

    /// Returns the type of the normalized number.
    #[must_use]
    pub const fn type_(&self) -> NumberType {
        self.0.type_()
    }

    /// Returns a reference to the inner number.
    #[must_use]
    pub const fn as_number(&self) -> &Number {
        &self.0
    }
}

impl PartialOrd for NormalizedNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq<Number> for NormalizedNumber {
    fn eq(&self, other: &Number) -> bool {
        self.0 == *other
    }
}

impl PartialEq<NormalizedNumber> for Number {
    fn eq(&self, other: &NormalizedNumber) -> bool {
        *self == other.0
    }
}

impl PartialOrd<Number> for NormalizedNumber {
    fn partial_cmp(&self, other: &Number) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<NormalizedNumber> for Number {
    fn partial_cmp(&self, other: &NormalizedNumber) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl ops::Neg for NormalizedNumber {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl ops::Add for NormalizedNumber {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Add<Number> for NormalizedNumber {
    type Output = Self;

    fn add(self, rhs: Number) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl ops::Add<NormalizedNumber> for Number {
    type Output = NormalizedNumber;

    fn add(self, rhs: NormalizedNumber) -> Self::Output {
        NormalizedNumber(self + rhs.0)
    }
}

impl ops::Sub for NormalizedNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::Sub<Number> for NormalizedNumber {
    type Output = Self;

    fn sub(self, rhs: Number) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl ops::Sub<NormalizedNumber> for Number {
    type Output = NormalizedNumber;

    fn sub(self, rhs: NormalizedNumber) -> Self::Output {
        NormalizedNumber(self - rhs.0)
    }
}

impl ops::Mul for NormalizedNumber {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<Number> for NormalizedNumber {
    type Output = Self;

    fn mul(self, rhs: Number) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Mul<NormalizedNumber> for Number {
    type Output = NormalizedNumber;

    fn mul(self, rhs: NormalizedNumber) -> Self::Output {
        NormalizedNumber(self * rhs.0)
    }
}

impl ops::Div for NormalizedNumber {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<Number> for NormalizedNumber {
    type Output = Self;

    fn div(self, rhs: Number) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl ops::Div<NormalizedNumber> for Number {
    type Output = NormalizedNumber;

    fn div(self, rhs: NormalizedNumber) -> Self::Output {
        NormalizedNumber(self / rhs.0)
    }
}

impl ops::Rem for NormalizedNumber {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl ops::Rem<Number> for NormalizedNumber {
    type Output = Self;

    fn rem(self, rhs: Number) -> Self::Output {
        Self(self.0 % rhs)
    }
}

impl ops::Rem<NormalizedNumber> for Number {
    type Output = NormalizedNumber;

    fn rem(self, rhs: NormalizedNumber) -> Self::Output {
        NormalizedNumber(self % rhs.0)
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
