mod error;
mod interval;
mod number;
mod unit;

pub use self::error::ValueError;
pub use self::interval::Interval;
pub use self::number::{MeasuredNumber, Number};
pub use self::unit::{SizedUnit, Unit};

use std::cmp::Ordering;

// TODO: document the layers of a value

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number(MeasuredNumber),
}

impl Value {
    /// Checks if two values are equal.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the values have incompatible types.
    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(lhs == rhs),
            (Self::String(lhs), Self::String(rhs)) => Ok(lhs == rhs),
            (Self::Number(lhs), Self::Number(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Equal)),
            _ => Err(ValueError::InvalidType),
        }
    }

    /// Checks if two values are not equal.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the values have incompatible types.
    pub fn checked_ne(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(lhs != rhs),
            (Self::String(lhs), Self::String(rhs)) => Ok(lhs != rhs),
            (Self::Number(lhs), Self::Number(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering != Some(Ordering::Equal)),
            _ => Err(ValueError::InvalidType),
        }
    }

    /// Checks if the left value is less than the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_lt(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Less)),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Checks if the left value is less than or equal to the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_lte(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => {
                lhs.checked_partial_cmp(rhs).map(|ordering| {
                    ordering == Some(Ordering::Less) || ordering == Some(Ordering::Equal)
                })
            }
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Checks if the left value is greater than the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_gt(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Greater)),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Checks if the left value is greater than or equal to the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_gte(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => {
                lhs.checked_partial_cmp(rhs).map(|ordering| {
                    ordering == Some(Ordering::Greater) || ordering == Some(Ordering::Equal)
                })
            }
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Adds two values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_add(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_add(&rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Subtracts the right value from the left value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_sub(&rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Multiplies two values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_mul(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Divides the left value by the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_div(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_div(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Computes the remainder of dividing the left value by the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_rem(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_rem(&rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Raises the left value to the power of the right value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_pow(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_pow(&rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Performs logical AND on two boolean values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a boolean.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a boolean.
    pub fn checked_and(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs && rhs)),
            (Self::Boolean(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Performs logical OR on two boolean values.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a boolean.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a boolean.
    pub fn checked_or(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs || rhs)),
            (Self::Boolean(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Computes the minimum and maximum of two values as an interval.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidType` if the right operand is not a number.
    ///
    /// Returns `ValueError::InvalidOperation` if the left operand is not a number.
    pub fn checked_min_max(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_min_max(&rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Negates a number value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidOperation` if the value is not a number.
    pub fn checked_neg(self) -> Result<Self, ValueError> {
        match self {
            Self::Number(number) => Ok(Self::Number(number.checked_neg())),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    /// Performs logical NOT on a boolean value.
    ///
    /// # Errors
    ///
    /// Returns `ValueError::InvalidOperation` if the value is not a boolean.
    pub fn checked_not(self) -> Result<Self, ValueError> {
        match self {
            Self::Boolean(boolean) => Ok(Self::Boolean(!boolean)),
            _ => Err(ValueError::InvalidOperation),
        }
    }
}
