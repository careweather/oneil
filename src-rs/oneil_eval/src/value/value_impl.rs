use std::{cmp::Ordering, fmt};

use crate::{
    EvalError,
    value::{MeasuredNumber, Number, ValueType},
};

// TODO: document the layers of a value

/// Represents a value in Oneil
///
/// A value is one of:
/// - a boolean
/// - a string
/// - a number
/// - a measured number
#[derive(Debug, Clone, PartialEq)]
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
    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, EvalError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(lhs == rhs),
            (Self::String(lhs), Self::String(rhs)) => Ok(lhs == rhs),
            // if either number isn't measured, then units don't matter
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs == rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(lhs == rhs.normalized_value()),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(lhs.normalized_value() == rhs),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Equal)),
            _ => Err(EvalError::InvalidType),
        }
    }

    /// Checks if two values are not equal.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the values have incompatible types.
    pub fn checked_ne(&self, rhs: &Self) -> Result<bool, EvalError> {
        self.checked_eq(rhs).map(|eq| !eq)
    }

    /// Checks if the left value is less than the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_lt(&self, rhs: &Self) -> Result<bool, EvalError> {
        match (self, rhs) {
            // if either number isn't measured, then units don't matter
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs < rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(lhs < rhs.normalized_value()),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(lhs.normalized_value() < rhs),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Less)),
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Checks if the left value is less than or equal to the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_lte(&self, rhs: &Self) -> Result<bool, EvalError> {
        match (self, rhs) {
            // if either number isn't measured, then units don't matter
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs <= rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(lhs <= rhs.normalized_value()),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(lhs.normalized_value() <= rhs),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_partial_cmp(rhs).map(|ordering| {
                    ordering == Some(Ordering::Less) || ordering == Some(Ordering::Equal)
                })
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Checks if the left value is greater than the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_gt(&self, rhs: &Self) -> Result<bool, EvalError> {
        match (self, rhs) {
            // if either number isn't measured, then units don't matter
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs > rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(lhs > rhs.normalized_value()),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(lhs.normalized_value() > rhs),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Greater)),
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Checks if the left value is greater than or equal to the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_gte(&self, rhs: &Self) -> Result<bool, EvalError> {
        match (self, rhs) {
            // if either number isn't measured, then units don't matter
            (Self::Number(lhs), Self::Number(rhs)) => Ok(lhs >= rhs),
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(lhs >= rhs.normalized_value()),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(lhs.normalized_value() >= rhs),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_partial_cmp(rhs).map(|ordering| {
                    ordering == Some(Ordering::Greater) || ordering == Some(Ordering::Equal)
                })
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Adds two values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_add(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs + rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a measured number
            // with the same unit as the measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs + rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs + rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_add(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Subtracts the right value from the left value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a measured number
            // with the same unit as the measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs - rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs - rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_sub(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
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
    pub fn checked_escaped_sub(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs - rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a measured number
            // with the same unit as the measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs - rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs - rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_escaped_sub(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Multiplies two values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs * rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // unitless measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs * rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs * rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_mul(rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Divides the left value by the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_div(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // unitless measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_div(rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
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
    pub fn checked_escaped_div(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs / rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // unitless measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs / rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_escaped_div(rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Computes the remainder of dividing the left value by the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_rem(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => Ok(Self::Number(lhs % rhs)),
            // if any of the numbers is not measured, it is implicitly coerced to a
            // unitless measured number
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => Ok(Self::MeasuredNumber(lhs % rhs)),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => Ok(Self::MeasuredNumber(lhs % rhs)),
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_rem(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Raises the left value to the power of the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_pow(self, exponent: Self) -> Result<Self, EvalError> {
        match (self, exponent) {
            (Self::Number(base), Self::Number(exponent)) => Ok(Self::Number(base.pow(exponent))),
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => {
                Ok(Self::MeasuredNumber(lhs.checked_pow(&rhs)?))
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Performs logical AND on two boolean values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a boolean.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a boolean.
    pub fn checked_and(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs && rhs)),
            (Self::Boolean(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Performs logical OR on two boolean values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a boolean.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a boolean.
    pub fn checked_or(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs || rhs)),
            (Self::Boolean(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Computes the minimum and maximum of two values as an interval.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_min_max(self, rhs: Self) -> Result<Self, EvalError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => {
                Ok(Self::Number(lhs.tightest_enclosing_interval(rhs)))
            }
            (Self::Number(lhs), Self::MeasuredNumber(rhs)) => {
                // the operation is associative
                Ok(Self::MeasuredNumber(rhs.min_max_number(lhs)))
            }
            (Self::MeasuredNumber(lhs), Self::Number(rhs)) => {
                Ok(Self::MeasuredNumber(lhs.min_max_number(rhs)))
            }
            (Self::MeasuredNumber(lhs), Self::MeasuredNumber(rhs)) => {
                lhs.checked_min_max(&rhs).map(Self::MeasuredNumber)
            }
            (Self::MeasuredNumber(_), _) => Err(EvalError::InvalidType),
            _ => Err(EvalError::InvalidOperation),
        }
    }

    /// Negates a number value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidOperation` if the value is not a number.
    pub fn checked_neg(self) -> Result<Self, EvalError> {
        match self {
            Self::Number(number) => Ok(Self::Number(-number)),
            Self::MeasuredNumber(number) => Ok(Self::MeasuredNumber(number.checked_neg())),
            Self::Boolean(_) | Self::String(_) => Err(EvalError::InvalidOperation),
        }
    }

    /// Performs logical NOT on a boolean value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidOperation` if the value is not a boolean.
    pub fn checked_not(self) -> Result<Self, EvalError> {
        match self {
            Self::Boolean(boolean) => Ok(Self::Boolean(!boolean)),
            Self::String(_) | Self::Number(_) | Self::MeasuredNumber(_) => {
                Err(EvalError::InvalidOperation)
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
