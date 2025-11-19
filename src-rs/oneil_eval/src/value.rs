use std::ops;

use crate::{interval::Interval, unit::Unit};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number { value: NumberValue, unit: Unit },
}

// TODO: in number value docs mention that for the outside world,
//       a number value is essentially an interval. The fact that
//       it is sometimes stored as a scalar is an implementation detail.
#[derive(Debug, Clone, Copy)]
pub enum NumberValue {
    Scalar(f64),
    Interval(Interval),
}

impl NumberValue {
    pub const fn new_scalar(value: f64) -> Self {
        Self::Scalar(value)
    }

    pub fn new_interval(min: f64, max: f64) -> Self {
        Self::Interval(Interval::new(min, max))
    }

    pub const fn new_empty() -> Self {
        Self::Interval(Interval::empty())
    }

    pub fn pow(&self, exponent: &Self) -> Self {
        match (self, exponent) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => Self::Scalar(lhs.powf(*rhs)),
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).pow(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.pow(&Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs.pow(rhs)),
        }
    }

    pub fn intersection(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => {
                Self::Interval(Interval::from(lhs).intersection(&Interval::from(rhs)))
            }
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).intersection(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.intersection(&Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => Self::Interval(lhs.intersection(rhs)),
        }
    }

    pub fn tightest_enclosing_interval(&self, rhs: &Self) -> Self {
        match (self, rhs) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => {
                Self::new_interval(f64::min(*lhs, *rhs), f64::max(*lhs, *rhs))
            }
            (Self::Scalar(lhs), Self::Interval(rhs)) => {
                Self::Interval(Interval::from(lhs).tightest_enclosing_interval(rhs))
            }
            (Self::Interval(lhs), Self::Scalar(rhs)) => {
                Self::Interval(lhs.tightest_enclosing_interval(&Interval::from(rhs)))
            }
            (Self::Interval(lhs), Self::Interval(rhs)) => {
                Self::Interval(lhs.tightest_enclosing_interval(rhs))
            }
        }
    }
}

impl PartialEq for NumberValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(lhs), Self::Scalar(rhs)) => lhs == rhs,
            (Self::Scalar(lhs), Self::Interval(rhs)) => lhs == &rhs.min() && lhs == &rhs.max(),
            (Self::Interval(lhs), Self::Scalar(rhs)) => &lhs.min() == rhs && &lhs.max() == rhs,
            (Self::Interval(lhs), Self::Interval(rhs)) => lhs == rhs,
        }
    }
}

impl PartialOrd for NumberValue {
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

impl ops::Neg for NumberValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(-value),
            Self::Interval(interval) => Self::Interval(-interval),
        }
    }
}

impl ops::Add for NumberValue {
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

impl ops::Sub for NumberValue {
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

impl ops::Mul for NumberValue {
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

impl ops::Mul<f64> for NumberValue {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(value * rhs),
            Self::Interval(interval) => Self::Interval(interval * Interval::from(rhs)),
        }
    }
}

impl ops::Div for NumberValue {
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
