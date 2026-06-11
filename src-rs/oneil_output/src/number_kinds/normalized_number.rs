//! Normalized number value (magnitude/dB applied).

use std::ops;

use crate::{
    Unit,
    util::{db_to_linear, linear_to_db},
};

use super::Number;

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
    /// Creates a normalized number from an already-normalized value.
    ///
    /// Use this when the value is already in normalized form (e.g. when
    /// extracting min/max from an existing normalized number).
    #[must_use]
    pub const fn from_normalized_value(value: Number) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn from_number_and_unit(value: Number, unit: &Unit) -> Self {
        // convert the number from a logarithmic unit
        // to a linear unit if it is a dB unit
        let value = if unit.is_db {
            db_to_linear(value)
        } else {
            value
        };

        // adjust the magnitude based on the unit
        let value = value * unit.magnitude;

        Self(value)
    }

    #[must_use]
    pub fn into_number_using_unit(self, unit: &Unit) -> Number {
        let Self(value) = self;

        // adjust the magnitude based on the unit
        let value = value / unit.magnitude;

        // convert the number from a linear unit
        // to a logarithmic unit if it is a dB unit
        if unit.is_db {
            linear_to_db(value)
        } else {
            value
        }
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

    /// Returns the intersection of two normalized numbers.
    #[must_use]
    pub fn intersection(self, rhs: Self) -> Self {
        Self(self.0.intersection(rhs.0))
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
    pub const fn type_(&self) -> crate::NumberType {
        self.0.type_()
    }

    /// Returns a reference to the inner number.
    #[must_use]
    pub const fn as_number(&self) -> &Number {
        &self.0
    }

    /// Returns the square root of the normalized number.
    #[must_use]
    pub fn sqrt(self) -> Self {
        Self(self.0.sqrt())
    }

    /// Returns the natural logarithm of the normalized number.
    #[must_use]
    pub fn ln(self) -> Self {
        Self(self.0.ln())
    }

    /// Returns the base 10 logarithm of the normalized number.
    #[must_use]
    pub fn log10(self) -> Self {
        Self(self.0.log10())
    }

    /// Returns the base 2 logarithm of the normalized number.
    #[must_use]
    pub fn log2(self) -> Self {
        Self(self.0.log2())
    }

    /// Returns the absolute value of the normalized number.
    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Returns the normalized number rounded down to the nearest integer.
    #[must_use]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    /// Returns the normalized number rounded up to the nearest integer.
    #[must_use]
    pub fn ceiling(self) -> Self {
        Self(self.0.ceiling())
    }

    /// Returns true if this normalized number is equal to `rhs`.
    #[must_use]
    pub fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }

    /// Returns true if `self` is strictly less than `rhs`.
    #[must_use]
    pub fn lt(&self, rhs: &Self) -> bool {
        self.0.lt(&rhs.0)
    }

    /// Returns true if `self` is strictly greater than `rhs`.
    #[must_use]
    pub fn gt(&self, rhs: &Self) -> bool {
        self.0.gt(&rhs.0)
    }

    /// Returns true if `self` is less than or equal to `rhs`.
    #[must_use]
    pub fn lte(&self, rhs: &Self) -> bool {
        self.0.lte(&rhs.0)
    }

    /// Returns true if `self` is greater than or equal to `rhs`.
    #[must_use]
    pub fn gte(&self, rhs: &Self) -> bool {
        self.0.gte(&rhs.0)
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

impl ops::Sub for NormalizedNumber {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
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
