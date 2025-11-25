mod error;
mod interval;
mod number_value;
mod unit;

pub use self::error::ValueError;
pub use self::interval::Interval;
pub use self::number_value::NumberValue;
pub use self::unit::{ComplexDimension, Unit};

use std::{cmp::Ordering, ops};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number { value: NumberValue, unit: Unit },
}

impl Value {
    pub fn checked_compare(&self, rhs: &Self) -> Result<Ordering, ValueError> {
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
