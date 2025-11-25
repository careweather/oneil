mod error;
mod interval;
mod number;
mod unit;

pub use self::error::ValueError;
pub use self::interval::Interval;
pub use self::number::{Number, NumberValue};
pub use self::unit::{ComplexDimension, Unit};

use std::{cmp::Ordering, ops};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number(Number),
}

impl Value {
    // ====== COMPARISON FUNCTIONS ======
    pub fn checked_compare(&self, rhs: &Self) -> Result<Ordering, ValueError> {
        match (self, rhs) {
            (Self::Number(lhs), Self::Number(rhs)) => {
                lhs.partial_cmp(rhs).ok_or(ValueError::InvalidUnit)
            }
            (Self::Number { .. }, _) => Err(ValueError::InvalidType),
            _ => Err(ValueError::InvalidOperation),
        }
    }

    pub fn checked_eq(&self, rhs: &Self) -> Result<bool, ValueError> {
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(lhs == rhs),
            (Self::String(lhs), Self::String(rhs)) => Ok(lhs == rhs),
            (Self::Number { .. }, Self::Number { .. }) => self
                .checked_compare(rhs)
                .map(|ordering| ordering == Ordering::Equal),
            _ => Err(ValueError::InvalidType),
        }
    }

    pub fn checked_ne(&self, rhs: &Self) -> Result<bool, ValueError> {
        todo!()
    }

    pub fn checked_lt(&self, rhs: &Self) -> Result<bool, ValueError> {
        todo!()
    }

    pub fn checked_lte(&self, rhs: &Self) -> Result<bool, ValueError> {
        todo!()
    }

    pub fn checked_gt(&self, rhs: &Self) -> Result<bool, ValueError> {
        todo!()
    }

    pub fn checked_gte(&self, rhs: &Self) -> Result<bool, ValueError> {
        todo!()
    }
}

impl ops::Neg for Value {
    type Output = Result<Self, ValueError>;

    fn neg(self) -> Self::Output {
        todo!()
    }
}

impl ops::Not for Value {
    type Output = Result<Self, ValueError>;

    fn not(self) -> Self::Output {
        todo!()
    }
}

impl ops::Add for Value {
    type Output = Result<Self, ValueError>;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl ops::Sub for Value {
    type Output = Result<Self, ValueError>;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl ops::Mul for Value {
    type Output = Result<Self, ValueError>;

    fn mul(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl ops::Div for Value {
    type Output = Result<Self, ValueError>;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl ops::Rem for Value {
    type Output = Result<Self, ValueError>;

    fn rem(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
