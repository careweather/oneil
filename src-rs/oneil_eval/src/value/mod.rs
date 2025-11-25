mod error;
mod interval;
mod number;
mod unit;

pub use self::error::ValueError;
pub use self::interval::Interval;
pub use self::number::{DimensionalNumber, Number};
pub use self::unit::ComplexDimension;

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number(DimensionalNumber),
}

impl Value {
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

    pub fn checked_lt(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Less)),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

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

    pub fn checked_gt(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs
                .checked_partial_cmp(rhs)
                .map(|ordering| ordering == Some(Ordering::Greater)),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

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

    pub fn checked_add(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_add(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_sub(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_mul(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_mul(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_div(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_div(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_rem(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_rem(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_pow(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_pow(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_and(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs && rhs)),
            (Self::Boolean(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_or(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(lhs || rhs)),
            (Self::Boolean(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_min_max(self, rhs: Self) -> Result<Self, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs.checked_min_max(rhs).map(Self::Number),
            (Self::Number(_), _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }
}
