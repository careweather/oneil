use std::{cmp::Ordering, ops};

use crate::interval::classification::IntervalClass;

mod classification;

// TODO: maybe add more comparison functions for
//       intervals into the standard library (
//       such as `contains` or `overlaps`)

// TODO: add note linking arithmetic operations to
//       documentation in docs folder

// TODO: write fuzz tests for interval arithmetic

// NOTE: it may be worthwhile to take a look at
//       IEEE 1788-2015 and IEEE 1788.1-2017. They
//       contain standards for interval arithmetic,
//       and it might be a selling point for Oneil
//       to claim that it is compliant with these
//       standards.
//
//       For now, we are just implementing what is
//       needed so that we can get to a working
//       implementation.

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    min: f64,
    max: f64,
}

impl Interval {
    /// Creates a new interval without checking the validity of the interval.
    #[must_use]
    pub const fn new_unchecked(min: f64, max: f64) -> Self {
        let min = if min == 0.0 { 0.0 } else { min };
        let max = if max == 0.0 { -0.0 } else { max };
        Self { min, max }
    }

    #[must_use]
    pub fn new(min: f64, max: f64) -> Self {
        assert!(!min.is_nan(), "min must not be NaN in ({min:?}, {max:?})");
        assert!(!max.is_nan(), "max must not be NaN in ({min:?}, {max:?})");
        assert!(
            min <= max,
            "min must be less than or equal to max in ({min:?}, {max:?})"
        );

        Self::new_unchecked(min, max)
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new_unchecked(0.0, 0.0)
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self::new_unchecked(f64::NAN, f64::NAN)
    }

    #[must_use]
    pub const fn min(&self) -> f64 {
        self.min
    }

    #[must_use]
    pub const fn max(&self) -> f64 {
        self.max
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.min.is_nan() && self.max.is_nan()
    }

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.is_empty() || self.min <= self.max
    }

    /// Exponentiation of intervals
    ///
    /// This is defined based on the implementation in the
    /// [inari crate](https://docs.rs/inari/latest/src/inari/elementary.rs.html#588)
    #[must_use]
    pub fn pow(&self, exponent: &Self) -> Self {
        const DOMAIN: Interval = Interval::new_unchecked(0.0, f64::INFINITY);

        let base = self.intersection(&DOMAIN);

        if base.is_empty() || exponent.is_empty() {
            return Self::empty();
        }

        let base_min = base.min;
        let base_max = base.max;
        let exp_min = exponent.min;
        let exp_max = exponent.max;

        if exp_max <= 0.0 {
            if base_max == 0.0 {
                Self::empty()
            } else if base_max < 1.0 {
                let min = base_max.powf(exp_max);
                let max = base_min.powf(exp_min);
                Self::new(min, max)
            } else if base_min > 1.0 {
                let min = base_max.powf(exp_min);
                let max = base_min.powf(exp_max);
                Self::new(min, max)
            } else {
                let min = base_max.powf(exp_min);
                let max = base_min.powf(exp_min);
                Self::new(min, max)
            }
        } else if exp_min > 0.0 {
            if base_max < 1.0 {
                let min = base_min.powf(exp_max);
                let max = base_max.powf(exp_min);
                Self::new(min, max)
            } else if base_min > 1.0 {
                let min = base_min.powf(exp_min);
                let max = base_max.powf(exp_max);
                Self::new(min, max)
            } else {
                let min = base_min.powf(exp_max);
                let max = base_max.powf(exp_max);
                Self::new(min, max)
            }
        } else if base_max == 0.0 {
            let min = 0.0;
            let max = 0.0;
            Self::new(min, max)
        } else {
            let min_min = base_min.powf(exp_min);
            let min_max = base_min.powf(exp_max);
            let max_min = base_max.powf(exp_min);
            let max_max = base_max.powf(exp_max);
            Self::new(f64::min(min_max, max_min), f64::max(min_min, max_max))
        }
    }

    #[must_use]
    pub fn intersection(&self, rhs: &Self) -> Self {
        if self.is_empty() || rhs.is_empty() {
            return Self::empty();
        }

        if self.min > rhs.max || rhs.min > self.max {
            return Self::empty();
        }

        let min = f64::max(self.min, rhs.min);
        let max = f64::min(self.max, rhs.max);
        Self::new(min, max)
    }

    /// Returns the tightest interval that contains both self and rhs as its subsets.
    ///
    /// # Examples
    ///
    /// ```
    /// use oneil_eval::interval::Interval;
    ///
    /// let interval1 = Interval::new(1.0, 2.0);
    /// let interval2 = Interval::new(3.0, 4.0);
    /// let interval3 = interval1.tightest_enclosing_interval(&interval2);
    ///
    /// assert_eq!(interval3, Interval::new(1.0, 4.0));
    /// ```
    #[must_use]
    pub fn tightest_enclosing_interval(&self, rhs: &Self) -> Self {
        if self.is_empty() {
            return *rhs;
        }

        if rhs.is_empty() {
            return *self;
        }

        let min = f64::min(self.min, rhs.min);
        let max = f64::max(self.max, rhs.max);

        Self::new(f64::min(self.min, rhs.min), f64::max(self.max, rhs.max))
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
        // if the min and max are the same, they are equal
        //
        //       |--- self ---|
        //       |--- other --|
        if self.min == other.min && self.max == other.max {
            return Some(Ordering::Equal);
        }

        // if the min of self is greater than the max of other, self is greater than other
        //
        //                  |--- self ---|
        // |--- other ---|
        if self.min.partial_cmp(&other.max) == Some(Ordering::Greater) {
            return Some(Ordering::Greater);
        }

        // if the max of self is less than the min of other, self is less than other
        //
        // |--- self ---|
        //                  |--- other ---|
        if self.max.partial_cmp(&other.min) == Some(Ordering::Less) {
            return Some(Ordering::Less);
        }

        // otherwise, there is not a clear ordering
        None
    }
}

impl ops::Neg for Interval {
    type Output = Self;

    /// Negation of intervals
    ///
    /// This is defined according to the research in the
    /// [interval arithmetic paper](/docs/research/2025-11-13-interval-arithmetic-paper-review.md).
    fn neg(self) -> Self::Output {
        Self {
            min: -self.max,
            max: -self.min,
        }
    }
}

impl ops::Add for Interval {
    type Output = Self;

    /// Addition of intervals
    ///
    /// This is defined according to the research in the
    /// [interval arithmetic paper](/docs/research/2025-11-13-interval-arithmetic-paper-review.md).
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

impl ops::Sub for Interval {
    type Output = Self;

    /// Subtraction of intervals
    ///
    /// This is defined according to the research in the
    /// [interval arithmetic paper](/docs/research/2025-11-13-interval-arithmetic-paper-review.md).
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            min: self.min - rhs.max,
            max: self.max - rhs.min,
        }
    }
}

impl ops::Mul for Interval {
    type Output = Self;

    /// Multiplication of intervals
    ///
    /// This is defined according to the research in the
    /// [interval arithmetic paper](/docs/research/2025-11-13-interval-arithmetic-paper-review.md).
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
            ) => Self::zero(),
            (
                IntervalClass::Zero,
                IntervalClass::Positive1
                | IntervalClass::Positive0
                | IntervalClass::Mixed
                | IntervalClass::Negative0
                | IntervalClass::Negative1,
            ) => Self::zero(),
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
                let max = lhs.min * rhs.min;
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
                let min = lhs.min * rhs.max;
                let max = lhs.min * rhs.min;
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

    /// Division of intervals
    ///
    /// This is defined according to the research in the
    /// [interval arithmetic paper](/docs/research/2025-11-13-interval-arithmetic-paper-review.md).
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

impl ops::Rem for Interval {
    type Output = Self;

    /// Modulo an interval
    ///
    /// This is defined by Brendon's reasoning and therefore
    /// may have incorrect behavior
    fn rem(self, rhs: Self) -> Self::Output {
        let lhs = self;

        if rhs.is_empty() {
            return Self::empty();
        }

        let abs_rhs = Self::new(
            f64::min(rhs.min.abs(), rhs.max.abs()),
            f64::max(rhs.min.abs(), rhs.max.abs()),
        );

        let rhs_includes_zero = rhs.min <= 0.0 && rhs.max >= 0.0;

        match classification::classify(&lhs) {
            IntervalClass::Empty => Self::empty(),
            IntervalClass::Zero => Self::zero(),
            IntervalClass::Positive0 | IntervalClass::Positive1 => {
                if lhs.max < abs_rhs.min {
                    if rhs_includes_zero {
                        let min = 0.0;
                        let max = lhs.max;
                        Self::new(min, max)
                    } else {
                        lhs
                    }
                } else {
                    let min = 0.0;
                    let max = abs_rhs.max;
                    Self::new(min, max)
                }
            }
            IntervalClass::Mixed => {
                let max = if lhs.max < abs_rhs.min {
                    lhs.max
                } else {
                    abs_rhs.max
                };

                let min = if lhs.min > -abs_rhs.min {
                    lhs.min
                } else {
                    -abs_rhs.max
                };

                Self::new(min, max)
            }
            IntervalClass::Negative0 | IntervalClass::Negative1 => {
                if lhs.min > -abs_rhs.min {
                    if rhs_includes_zero {
                        let min = lhs.min;
                        let max = 0.0;
                        Self::new(min, max)
                    } else {
                        lhs
                    }
                } else {
                    let min = -abs_rhs.max;
                    let max = 0.0;
                    Self::new(min, max)
                }
            }
        }
    }
}

impl ops::Rem<f64> for Interval {
    type Output = Self;

    /// Modulo an interval by a scalar
    ///
    /// This is a more specialized version of the modulo operation
    /// that uses the fact that the modulo is scalar to provide a
    /// more tight interval.
    ///
    /// This is defined by Brendon's reasoning and therefore
    /// may have incorrect behavior
    fn rem(self, rhs: f64) -> Self::Output {
        let lhs = self;

        if rhs.is_nan() {
            return Self::empty();
        }

        if rhs == 0.0 {
            return Self::empty();
        }

        let rhs = rhs.abs();

        let lhs_distance = lhs.max - lhs.min;
        let lhs_min_mod = lhs.min % rhs;
        let lhs_max_mod = lhs.max % rhs;

        match classification::classify(&lhs) {
            IntervalClass::Empty => Self::empty(),
            IntervalClass::Zero => Self::zero(),
            IntervalClass::Positive0 | IntervalClass::Positive1 => {
                if lhs_distance < rhs && lhs_min_mod <= lhs_max_mod {
                    Self::new(lhs_min_mod, lhs_max_mod)
                } else {
                    Self::new(0.0, rhs)
                }
            }
            IntervalClass::Mixed => {
                let max = if lhs.max < rhs { lhs.max } else { rhs };

                let min = if lhs.min > -rhs { lhs.min } else { -rhs };

                Self::new(min, max)
            }
            IntervalClass::Negative0 | IntervalClass::Negative1 => {
                if lhs_distance < rhs && lhs_min_mod <= lhs_max_mod {
                    Self::new(lhs_min_mod, lhs_max_mod)
                } else {
                    Self::new(-rhs, 0.0)
                }
            }
        }
    }
}

#[cfg(feature = "arbitrary")]
use arbitrary::{Result, Unstructured};

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Interval {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let f = u.arbitrary::<f64>()?;
        let g = u.arbitrary::<f64>()?;

        if f.is_nan() || g.is_nan() {
            return Ok(Self::empty());
        }

        let min = f64::min(f, g);
        let max = f64::max(f, g);
        Ok(Self::new_unchecked(min, max))
    }
}
