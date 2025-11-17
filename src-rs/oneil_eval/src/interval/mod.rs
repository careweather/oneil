use std::{cmp::Ordering, ops};

use crate::interval::classification::IntervalClass;

mod classification;

// TODO: add note linking arithmetic operations to
//       documentation in docs folder

// TODO: write fuzz tests for interval arithmetic

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    min: f64,
    max: f64,
}

impl Interval {
    pub fn new(min: f64, max: f64) -> Self {
        assert!(!min.is_nan(), "min must not be NaN in ({min}, {max})");
        assert!(!max.is_nan(), "max must not be NaN in ({min}, {max})");
        assert!(
            min <= max,
            "min must be less than or equal to max in ({min}, {max})"
        );

        Self { min, max }
    }

    pub const fn empty() -> Self {
        Self {
            min: f64::NAN,
            max: f64::NAN,
        }
    }

    pub const fn min(&self) -> f64 {
        self.min
    }

    pub const fn max(&self) -> f64 {
        self.max
    }

    pub const fn is_empty(&self) -> bool {
        self.min.is_nan() && self.max.is_nan()
    }

    pub const fn is_valid(&self) -> bool {
        !self.is_empty() && self.min <= self.max
    }
}

impl From<f64> for Interval {
    fn from(value: f64) -> Self {
        Self::new(value, value)
    }
}

impl From<&f64> for Interval {
    fn from(value: &f64) -> Self {
        Self::from(*value)
    }
}

impl PartialEq for Interval {
    fn eq(&self, other: &Self) -> bool {
        self.is_empty() && other.is_empty() || self.min == other.min && self.max == other.max
    }
}

impl PartialOrd for Interval {
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
        match (
            self.min.partial_cmp(&other.min),
            self.max.partial_cmp(&other.max),
        ) {
            (Some(Ordering::Less), Some(Ordering::Less)) => Some(Ordering::Less),
            (Some(Ordering::Greater), Some(Ordering::Greater)) => Some(Ordering::Greater),
            (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

impl ops::Neg for Interval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            min: -self.max,
            max: -self.min,
        }
    }
}

impl ops::Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

impl ops::Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            min: self.min - rhs.max,
            max: self.max - rhs.min,
        }
    }
}

impl ops::Mul for Interval {
    type Output = Self;

    #[expect(
        clippy::match_same_arms,
        reason = "the different cases should remain distinct so that they can easily be checked against the table in the documentation"
    )]
    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self;

        let lhs_class = classification::classify(&lhs);
        let rhs_class = classification::classify(&rhs);

        match (lhs_class, rhs_class) {
            (IntervalClass::Empty, _) | (_, IntervalClass::Empty) => Self::empty(),
            (
                IntervalClass::Positive1
                | IntervalClass::Positive0
                | IntervalClass::Mixed
                | IntervalClass::Negative0
                | IntervalClass::Negative1
                | IntervalClass::Zero,
                IntervalClass::Zero,
            ) => Self::empty(),
            (
                IntervalClass::Zero,
                IntervalClass::Positive1
                | IntervalClass::Positive0
                | IntervalClass::Mixed
                | IntervalClass::Negative0
                | IntervalClass::Negative1,
            ) => {
                let min = 0.0;
                let max = 0.0;
                Self::new(min, max)
            }
            (
                IntervalClass::Positive1 | IntervalClass::Positive0,
                IntervalClass::Positive1 | IntervalClass::Positive0,
            ) => {
                let min = lhs.min * rhs.min;
                let max = lhs.max * rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Positive1 | IntervalClass::Positive0, IntervalClass::Mixed) => {
                let min = lhs.max * rhs.min;
                let max = lhs.max * rhs.max;
                Self::new(min, max)
            }
            (
                IntervalClass::Positive1 | IntervalClass::Positive0,
                IntervalClass::Negative1 | IntervalClass::Negative0,
            ) => {
                let min = lhs.max * rhs.min;
                let max = lhs.min * rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Positive1 | IntervalClass::Positive0) => {
                let min = lhs.min * rhs.max;
                let max = lhs.max * rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Mixed) => {
                let min = f64::min(lhs.min * rhs.max, lhs.max * rhs.min);
                let max = f64::max(lhs.min * rhs.min, lhs.max * rhs.max);
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Negative1 | IntervalClass::Negative0) => {
                let min = lhs.max * rhs.min;
                let max = lhs.min * rhs.max;
                Self::new(min, max)
            }
            (
                IntervalClass::Negative1 | IntervalClass::Negative0,
                IntervalClass::Positive1 | IntervalClass::Positive0,
            ) => {
                let min = lhs.min * rhs.max;
                let max = lhs.max * rhs.min;
                Self::new(min, max)
            }
            (IntervalClass::Negative1 | IntervalClass::Negative0, IntervalClass::Mixed) => {
                let min = lhs.min * rhs.min;
                let max = lhs.min * rhs.max;
                Self::new(min, max)
            }
            (
                IntervalClass::Negative0 | IntervalClass::Negative1,
                IntervalClass::Negative1 | IntervalClass::Negative0,
            ) => {
                let min = lhs.max * rhs.max;
                let max = lhs.min * rhs.min;
                Self::new(min, max)
            }
        }
    }
}

impl ops::Div for Interval {
    type Output = Self;

    #[expect(
        clippy::too_many_lines,
        reason = "the implementation is an implementation of a table, which is long but straightforward"
    )]
    #[expect(
        clippy::match_same_arms,
        reason = "the different cases should remain distinct so that they can easily be checked against the table in the documentation"
    )]
    fn div(self, rhs: Self) -> Self::Output {
        let lhs = self;

        let lhs_class = classification::classify(&lhs);
        let rhs_class = classification::classify(&rhs);

        match (lhs_class, rhs_class) {
            (IntervalClass::Empty, _) | (_, IntervalClass::Empty) => Self::empty(),
            (
                IntervalClass::Positive1
                | IntervalClass::Positive0
                | IntervalClass::Mixed
                | IntervalClass::Negative0
                | IntervalClass::Negative1
                | IntervalClass::Zero,
                IntervalClass::Zero,
            ) => Self::empty(),
            (
                IntervalClass::Zero,
                IntervalClass::Positive1
                | IntervalClass::Positive0
                | IntervalClass::Mixed
                | IntervalClass::Negative0
                | IntervalClass::Negative1,
            ) => {
                let min = 0.0;
                let max = 0.0;
                Self::new(min, max)
            }
            (IntervalClass::Positive1, IntervalClass::Positive1) => {
                let min = lhs.min / rhs.max;
                let max = lhs.max / rhs.min;
                Self::new(min, max)
            }
            (IntervalClass::Positive1, IntervalClass::Positive0) => {
                let min = lhs.min / rhs.max;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
            (IntervalClass::Positive0, IntervalClass::Positive1) => {
                let min = 0.0;
                let max = lhs.max / rhs.min;
                Self::new(min, max)
            }
            (IntervalClass::Positive0, IntervalClass::Positive0) => {
                let min = 0.0;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Positive1) => {
                let min = lhs.min / rhs.min;
                let max = lhs.max / rhs.min;
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Positive0) => {
                let min = f64::NEG_INFINITY;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
            (IntervalClass::Negative0, IntervalClass::Positive1) => {
                let min = lhs.min / rhs.min;
                let max = 0.0;
                Self::new(min, max)
            }
            (IntervalClass::Negative0, IntervalClass::Positive0) => {
                let min = f64::NEG_INFINITY;
                let max = 0.0;
                Self::new(min, max)
            }
            (IntervalClass::Negative1, IntervalClass::Positive1) => {
                let min = lhs.min / rhs.min;
                let max = lhs.max / rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Negative1, IntervalClass::Positive0) => {
                let min = f64::NEG_INFINITY;
                let max = lhs.max / rhs.max;
                Self::new(min, max)
            }
            (
                IntervalClass::Positive1
                | IntervalClass::Positive0
                | IntervalClass::Mixed
                | IntervalClass::Negative0
                | IntervalClass::Negative1,
                IntervalClass::Mixed,
            ) => {
                let min = f64::NEG_INFINITY;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
            (IntervalClass::Positive1, IntervalClass::Negative1) => {
                let min = lhs.max / rhs.max;
                let max = lhs.min / rhs.min;
                Self::new(min, max)
            }
            (IntervalClass::Positive1, IntervalClass::Negative0) => {
                let min = f64::NEG_INFINITY;
                let max = lhs.min / rhs.min;
                Self::new(min, max)
            }
            (IntervalClass::Positive0, IntervalClass::Negative1) => {
                let min = lhs.max / rhs.max;
                let max = 0.0;
                Self::new(min, max)
            }
            (IntervalClass::Positive0, IntervalClass::Negative0) => {
                let min = f64::NEG_INFINITY;
                let max = 0.0;
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Negative1) => {
                let min = lhs.max / rhs.max;
                let max = lhs.min / rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Mixed, IntervalClass::Negative0) => {
                let min = f64::NEG_INFINITY;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
            (IntervalClass::Negative0, IntervalClass::Negative1) => {
                let min = 0.0;
                let max = lhs.min / rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Negative0, IntervalClass::Negative0) => {
                let min = 0.0;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
            (IntervalClass::Negative1, IntervalClass::Negative1) => {
                let min = lhs.max / rhs.min;
                let max = lhs.min / rhs.max;
                Self::new(min, max)
            }
            (IntervalClass::Negative1, IntervalClass::Negative0) => {
                let min = lhs.max / rhs.min;
                let max = f64::INFINITY;
                Self::new(min, max)
            }
        }
    }
}
