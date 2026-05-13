use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    MeasuredNumber, Number, Unit, ValueType,
    error::{BinaryEvalError, ExpectedType, UnaryEvalError, UnitConversionError},
};

// TODO: document the layers of a value

/// Represents a value in Oneil
///
/// A value is one of:
/// - a boolean
/// - a string
/// - a number
/// - a measured number
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// A boolean value
    Boolean(bool),
    /// A string value
    String(String),
    /// A number value
    Number(Number),
    /// A measured number value, which is a number with a unit
    MeasuredNumber(MeasuredNumber),
}

impl Value {
    /// Checks if two values are equal.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the values have incompatible types.
    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(lhs == rhs),
            (Self::String(lhs), Self::String(rhs)) => Ok(lhs == rhs),
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs == rhs),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs.checked_eq(rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(*lhs, Unit::one());
                lhs_num.checked_eq(rhs)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(*rhs, Unit::one());
                lhs.checked_eq(&rhs_num)
            }
            (lhs, rhs) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                    &lhs.type_(),
                ),
                rhs_type: Box::new(rhs.type_()),
            }),
        }
    }

    /// Checks if two values are not equal.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the values have incompatible types.
    pub fn checked_ne(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        self.checked_eq(rhs).map(|eq| !eq)
    }

    /// Checks if the left value is less than the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_lt(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs.lt(rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs.checked_lt(rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(*lhs, Unit::one());
                lhs_num.checked_lt(rhs)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(*rhs, Unit::one());
                lhs.checked_lt(&rhs_num)
            }
            (Self::Number(_) | Self::MeasuredNumber(_), _) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                    &self.type_(),
                ),
                rhs_type: Box::new(rhs.type_()),
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Checks if the left value is less than or equal to the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_lte(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs.lte(rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs.checked_lte(rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(*lhs, Unit::one());
                lhs_num.checked_lte(rhs)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(*rhs, Unit::one());
                lhs.checked_lte(&rhs_num)
            }
            (Self::MeasuredNumber(_) | Self::Number(_), _) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                    &self.type_(),
                ),
                rhs_type: Box::new(rhs.type_()),
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Checks if the left value is greater than the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_gt(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs.gt(rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs.checked_gt(rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(*lhs, Unit::one());
                lhs_num.checked_gt(rhs)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(*rhs, Unit::one());
                lhs.checked_gt(&rhs_num)
            }
            (Self::MeasuredNumber(_) | Self::Number(_), _) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                    &self.type_(),
                ),
                rhs_type: Box::new(rhs.type_()),
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Checks if the left value is greater than or equal to the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_gte(&self, rhs: &Self) -> Result<bool, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs.gte(rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs.checked_gte(rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(*lhs, Unit::one());
                lhs_num.checked_gte(rhs)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(*rhs, Unit::one());
                lhs.checked_gte(&rhs_num)
            }
            (Self::MeasuredNumber(_) | Self::Number(_), _) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                    &self.type_(),
                ),
                rhs_type: Box::new(rhs.type_()),
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Adds two values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_add(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs + rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_add(&rhs).map(Self::MeasuredNumber)
            }
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(lhs, Unit::one());
                lhs_num.checked_add(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(rhs, Unit::one());
                lhs.checked_add(&rhs_num).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Subtracts the right value from the left value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_sub(&rhs).map(Self::MeasuredNumber)
            }
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(lhs, Unit::one());
                lhs_num.checked_sub(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(rhs, Unit::one());
                lhs.checked_sub(&rhs_num).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Subtracts the right value from the left value. This does not apply the
    /// standard rules of interval arithmetic. Instead, it subtracts the minimum
    /// from the minimum and the maximum from the maximum.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_escaped_sub(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_escaped_sub(&rhs).map(Self::MeasuredNumber)
            }
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(lhs, Unit::one());
                lhs_num.checked_escaped_sub(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(rhs, Unit::one());
                lhs.checked_escaped_sub(&rhs_num).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Multiplies two values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs * rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // measured number with unit `1`
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs * rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs * rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_mul(rhs).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Divides the left value by the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_div(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // measured number with unit `1`
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_div(rhs).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Divides the left value by the right value. This does not apply the
    /// standard rules of interval arithmetic. Instead, it divides the minimum
    /// by the minimum and the maximum by the maximum.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_escaped_div(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // measured number with unit `1`
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_escaped_div(rhs).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Computes the remainder of dividing the left value by the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_rem(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs % rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_rem(&rhs).map(Self::MeasuredNumber)
            }
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(lhs, Unit::one());
                lhs_num.checked_rem(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(rhs, Unit::one());
                lhs.checked_rem(&rhs_num).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Raises the left value to the power of the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_pow(self, exponent: Self) -> Result<Self, BinaryEvalError> {
        match (self, exponent) {
            (Self::Number(base), Self::Number(exponent)) => Ok(Self::Number(base.pow(exponent))),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => {
                Ok(Self::MeasuredNumber(lhs.checked_pow(&rhs)?))
            }
            (Self::Number(base), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let rhs_num = rhs.into_number_using_unit(&Unit::one());
                Ok(Self::Number(base.pow(rhs_num)))
            }
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let rhs_num = rhs.into_number_using_unit(&Unit::one());
                Ok(Self::MeasuredNumber(lhs.checked_pow(&rhs_num)?))
            }
            (Self::Number(_) | Self::MeasuredNumber(_), Self::MeasuredNumber(rhs)) => {
                Err(BinaryEvalError::ExponentHasUnits {
                    exponent_unit: rhs.unit().display_unit.clone(),
                })
            }
            (Self::Number(_) | Self::MeasuredNumber(_), exponent) => {
                Err(BinaryEvalError::InvalidRhsType {
                    expected_type: ExpectedType::Number { number_type: None },
                    rhs_type: Box::new(exponent.type_()),
                })
            }
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Performs logical AND on two boolean values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a boolean.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a boolean.
    pub fn checked_and(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs && rhs)),
            (Self::Boolean(_), rhs) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::Boolean,
                rhs_type: Box::new(rhs.type_()),
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::Boolean,
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Performs logical OR on two boolean values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a boolean.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a boolean.
    pub fn checked_or(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs || rhs)),
            (Self::Boolean(_), rhs) => Err(BinaryEvalError::TypeMismatch {
                expected_type_from_lhs: ExpectedType::Boolean,
                rhs_type: Box::new(rhs.type_()),
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::Boolean,
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Computes the minimum and maximum of two values as an interval.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_min_max(self, rhs: Self) -> Result<Self, BinaryEvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => {
                Ok(Self::Number(lhs.tightest_enclosing_interval(rhs)))
            }
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_min_max(&rhs).map(Self::MeasuredNumber)
            }
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) if rhs.is_dimensionless() => {
                let lhs_num = MeasuredNumber::from_number_and_unit(lhs, Unit::one());
                lhs_num.checked_min_max(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) if lhs.is_dimensionless() => {
                let rhs_num = MeasuredNumber::from_number_and_unit(rhs, Unit::one());
                lhs.checked_min_max(&rhs_num).map(Self::MeasuredNumber)
            }
            (lhs @ (Self::Number(_) | Self::MeasuredNumber(_)), rhs) => Err({
                BinaryEvalError::TypeMismatch {
                    expected_type_from_lhs: ExpectedType::matching_value_type_ignoring_number_kind(
                        &lhs.type_(),
                    ),
                    rhs_type: Box::new(rhs.type_()),
                }
            }),
            (lhs, _rhs) => Err(BinaryEvalError::InvalidLhsType {
                expected_type: ExpectedType::NumberOrMeasuredNumber { number_type: None },
                lhs_type: Box::new(lhs.type_()),
            }),
        }
    }

    /// Negates a number value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidOperation` if the value is not a number.
    pub fn checked_neg(self) -> Result<Self, UnaryEvalError> {
        match self {
            Self::Number(number) => Ok(Self::Number(-number)),
            Self::MeasuredNumber(number) => Ok(Self::MeasuredNumber(number.checked_neg())),
            Self::Boolean(_) | Self::String(_) => Err(UnaryEvalError::InvalidNegType {
                value_type: Box::new(self.type_()),
            }),
        }
    }

    /// Performs logical NOT on a boolean value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidOperation` if the value is not a boolean.
    pub fn checked_not(self) -> Result<Self, UnaryEvalError> {
        match self {
            Self::Boolean(boolean) => Ok(Self::Boolean(!boolean)),
            Self::String(_) | Self::Number(_) | Self::MeasuredNumber(_) => {
                Err(UnaryEvalError::InvalidNotType {
                    value_type: Box::new(self.type_()),
                })
            }
        }
    }

    /// Returns the type of the value.
    #[must_use]
    pub fn type_(&self) -> ValueType {
        match self {
            Self::Boolean(_) => ValueType::Boolean,
            Self::String(_) => ValueType::String,
            Self::Number(number) => ValueType::Number {
                number_type: number.type_(),
            },
            Self::MeasuredNumber(number) => ValueType::MeasuredNumber {
                unit: number.unit().clone(),
                number_type: number.normalized_value().type_(),
            },
        }
    }

    /// Converts the value to the target unit.
    ///
    /// # Errors
    ///
    /// Returns `UnitConversionError` if the value is not a measured number
    /// or if the unit dimensions do not match.
    pub fn with_unit(self, target_unit: Unit) -> Result<Self, UnitConversionError> {
        match self {
            Self::MeasuredNumber(measured_number) => {
                if !measured_number.unit().dimensionally_eq(&target_unit) {
                    return Err(UnitConversionError::UnitMismatch {
                        value_unit: measured_number.unit().display_unit.clone(),
                        target_unit: target_unit.display_unit,
                    });
                }

                Ok(Self::MeasuredNumber(measured_number.with_unit(target_unit)))
            }
            Self::Number(number) => Ok(Self::MeasuredNumber(MeasuredNumber::from_number_and_unit(
                number,
                target_unit,
            ))),
            Self::Boolean(_) | Self::String(_) => Err(UnitConversionError::InvalidType {
                value_type: Box::new(self.type_()),
                target_unit: Box::new(target_unit.display_unit),
            }),
        }
    }
}

impl From<f64> for Value {
    /// Converts an `f64` to a unitless number value.
    fn from(value: f64) -> Self {
        Self::Number(Number::Scalar(value))
    }
}

impl From<bool> for Value {
    /// Converts a `bool` to a boolean value.
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&str> for Value {
    /// Converts a `&str` to a string value.
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for Value {
    /// Converts a `String` to a string value.
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(boolean) => write!(f, "<{boolean}>"),
            Self::String(string) => write!(f, "'{string}'"),
            Self::Number(number) => write!(f, "<{number}>"),
            Self::MeasuredNumber(number) => {
                let (number, unit) = number.clone().into_number_and_unit();
                write!(f, "<{number} {unit}>")
            }
        }
    }
}

#[cfg(test)]
mod serde_tests {
    use std::collections::BTreeMap;

    use crate::{Dimension, DimensionMap, DisplayUnit, Interval};

    use super::*;
    use serde_json::json;

    fn sample_unit() -> Unit {
        Unit {
            dimension_map: DimensionMap::new(BTreeMap::from([
                (Dimension::Mass, 1.0),
                (Dimension::Distance, 2.0),
                (Dimension::Time, -3.0),
            ])),
            magnitude: 1.0,
            is_db: false,
            display_unit: DisplayUnit::Unit {
                name: "W".to_string(),
                exponent: 1.0,
            },
        }
    }

    #[test]
    fn bool_round_trip() {
        let v = Value::Boolean(true);
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }

    #[test]
    fn string_round_trip() {
        let v = Value::String("array".into());
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }

    #[test]
    fn scalar_round_trip() {
        let v = Value::Number(Number::Scalar(10.0));
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }

    #[test]
    fn interval_round_trip() {
        let v = Value::Number(Number::new_interval(1.0, 2.0));
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }

    #[test]
    fn measured_scalar_round_trip() {
        let unit = sample_unit();
        let v = Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
            Number::Scalar(100.0),
            unit,
        ));
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }

    #[test]
    fn measured_interval_round_trip() {
        let unit = sample_unit();
        let v = Value::MeasuredNumber(MeasuredNumber::from_number_and_unit(
            Number::Interval(Interval::new(1.0, 2.0)),
            unit,
        ));
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }

    #[test]
    fn unknown_dimension_rejected_on_deserialize() {
        let j = json!({ "value": 1.0, "unit": {
            "dimension_map": { "parsec": 1.0 },
            "magnitude": 1.0,
            "is_db": false,
            "display_unit": "pc"
        }});
        let err: Result<Value, _> = serde_json::from_value(j);
        let err = err.expect_err("unknown dimension");
        assert!(err.to_string().contains("did not match any variant"));
    }

    #[test]
    fn serde_json_round_trip_preserves_value() {
        let v = Value::Number(Number::Scalar(std::f64::consts::PI));
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, v);
    }
}
