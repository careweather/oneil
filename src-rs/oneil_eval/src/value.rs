use std::{cmp::Ordering, ops};

use crate::unit::Unit;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number { value: NumberValue, unit: Unit },
}

#[derive(Debug, Clone, Copy)]
pub enum NumberValue {
    Scalar(f64),
    Interval { min: f64, max: f64 },
}

impl NumberValue {
    pub fn range_merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Scalar(a), Self::Scalar(b)) => {
                // TODO: this uses direct f64 comparison, figure out a better way to do this
                if a == b {
                    Self::Scalar(a)
                } else {
                    Self::Interval {
                        min: f64::min(a, b),
                        max: f64::max(a, b),
                    }
                }
            }
            (
                Self::Scalar(a),
                Self::Interval {
                    min: b_min,
                    max: b_max,
                },
            ) => Self::Interval {
                min: f64::min(a, b_min),
                max: f64::max(a, b_max),
            },
            (
                Self::Interval {
                    min: a_min,
                    max: a_max,
                },
                Self::Scalar(b),
            ) => Self::Interval {
                min: f64::min(a_min, b),
                max: f64::max(a_max, b),
            },
            (
                Self::Interval {
                    min: a_min,
                    max: a_max,
                },
                Self::Interval {
                    min: b_min,
                    max: b_max,
                },
            ) => Self::Interval {
                min: f64::min(a_min, b_min),
                max: f64::max(a_max, b_max),
            },
        }
    }
}

impl PartialEq for NumberValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scalar(left), Self::Scalar(right)) => left == right,
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => left_min == right_min && left_max == right_max,
            (
                Self::Scalar(left),
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => left == right_min && left == right_max,
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Scalar(right),
            ) => left_min == right && left_max == right,
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
            (Self::Scalar(left), Self::Scalar(right)) => left.partial_cmp(right),
            (
                Self::Scalar(left),
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => match (left.partial_cmp(right_min), left.partial_cmp(right_max)) {
                (Some(Ordering::Less), Some(Ordering::Less)) => Some(Ordering::Less),
                (Some(Ordering::Greater), Some(Ordering::Greater)) => Some(Ordering::Greater),
                (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
                _ => None,
            },
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Scalar(right),
            ) => match (left_min.partial_cmp(right), left_max.partial_cmp(right)) {
                (Some(Ordering::Less), Some(Ordering::Less)) => Some(Ordering::Less),
                (Some(Ordering::Greater), Some(Ordering::Greater)) => Some(Ordering::Greater),
                (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
                _ => None,
            },
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => match (
                left_min.partial_cmp(right_min),
                left_max.partial_cmp(right_max),
            ) {
                (Some(Ordering::Less), Some(Ordering::Less)) => Some(Ordering::Less),
                (Some(Ordering::Greater), Some(Ordering::Greater)) => Some(Ordering::Greater),
                (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
                _ => None,
            },
        }
    }
}

impl ops::Neg for NumberValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(-value),
            Self::Interval { min, max } => Self::Interval {
                min: -max,
                max: -min,
            },
        }
    }
}

impl ops::Add for NumberValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(left), Self::Scalar(right)) => Self::Scalar(left + right),
            (
                Self::Scalar(left),
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => Self::Interval {
                min: left + right_min,
                max: left + right_max,
            },
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Scalar(right),
            ) => Self::Interval {
                min: left_min + right,
                max: left_max + right,
            },
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => Self::Interval {
                min: left_min + right_min,
                max: left_max + right_max,
            },
        }
    }
}

impl ops::Sub for NumberValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(left), Self::Scalar(right)) => Self::Scalar(left - right),
            (
                Self::Scalar(left),
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => Self::Interval {
                min: left - right_max,
                max: left - right_min,
            },
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Scalar(right),
            ) => Self::Interval {
                min: left_min - right,
                max: left_max - right,
            },
            (
                Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => Self::Interval {
                min: left_min - right_max,
                max: left_max - right_min,
            },
        }
    }
}

impl ops::Mul for NumberValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(left_value), Self::Scalar(right_value)) => {
                Self::Scalar(left_value * right_value)
            }
            (Self::Scalar(left_value), right_interval @ Self::Interval { .. }) => {
                right_interval * left_value
            }
            (left_interval @ Self::Interval { .. }, Self::Scalar(right_value)) => {
                left_interval * right_value
            }
            (
                left_interval @ Self::Interval { .. },
                Self::Interval {
                    min: right_min_value,
                    max: right_max_value,
                },
            ) => Self::range_merge(
                left_interval * right_min_value,
                left_interval * right_max_value,
            ),
        }
    }
}

impl ops::Mul<f64> for NumberValue {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            Self::Scalar(value) => Self::Scalar(value * rhs),
            Self::Interval { min, max } => {
                if rhs > 0.0 {
                    Self::Interval {
                        min: min * rhs,
                        max: max * rhs,
                    }
                } else {
                    Self::Interval {
                        min: max * rhs,
                        max: min * rhs,
                    }
                }
            }
        }
    }
}

impl ops::Div for NumberValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Scalar(left_value), Self::Scalar(right_value)) => {
                Self::Scalar(left_value / right_value)
            }
            (
                Self::Scalar(left_value),
                right_interval @ Self::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => {
                let left_sign = left_value.signum();
                let right_min_sign = right_min.signum();
                let right_max_sign = right_max.signum();

                let swap_min_max = (left_sign * right_min_sign * right_max_sign) > 0.0;

                if swap_min_max {
                    Self::Interval {
                        min: left_value / right_max,
                        max: left_value / right_min,
                    }
                } else {
                    Self::Interval {
                        min: left_value / right_min,
                        max: left_value / right_max,
                    }
                }
            }
            (
                left_interval @ Self::Interval {
                    min: left_min,
                    max: left_max,
                },
                Self::Scalar(right_value),
            ) => {
                if right_value > 0.0 {
                    Self::Interval {
                        min: left_min / right_value,
                        max: left_max / right_value,
                    }
                } else {
                    Self::Interval {
                        min: left_max / right_value,
                        max: left_min / right_value,
                    }
                }
            }
            (left_interval @ Self::Interval { .. }, right_interval @ Self::Interval { .. }) => {
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn scalar(value: f64) -> NumberValue {
        NumberValue::Scalar(value)
    }

    const fn interval(min: f64, max: f64) -> NumberValue {
        NumberValue::Interval { min, max }
    }

    macro_rules! assert_valid_interval_min_max {
        ($interval:expr, $expr_str:expr) => {
            let interval = $interval;
            let expr_str = $expr_str;
            let NumberValue::Interval { min, max } = interval else {
                panic!("Expected interval, got {interval:?} ({expr_str})");
            };

            assert!(
                min <= max,
                "(min: {min}, max: {max}) is not a valid interval ({expr_str})"
            );
        };
    }

    const CASES: [(NumberValue, NumberValue); 12] = [
        (scalar(1.0), interval(2.0, 3.0)),
        (scalar(1.0), interval(-2.0, 3.0)),
        (scalar(1.0), interval(-3.0, -2.0)),
        (scalar(-1.0), interval(2.0, 3.0)),
        (scalar(-1.0), interval(-2.0, 3.0)),
        (scalar(-1.0), interval(-3.0, -2.0)),
        (interval(1.0, 2.0), interval(3.0, 4.0)),
        (interval(1.0, 2.0), interval(-3.0, 4.0)),
        (interval(1.0, 2.0), interval(-4.0, -3.0)),
        (interval(-1.0, 2.0), interval(-3.0, 4.0)),
        (interval(-1.0, 2.0), interval(-4.0, -3.0)),
        (interval(-2.0, -1.0), interval(-4.0, 3.0)),
    ];

    #[test]
    fn interval_neg() {
        for (left, _) in CASES {
            if let NumberValue::Scalar(_) = left {
                continue;
            }

            let result = -left;
            assert_valid_interval_min_max!(result, format!("-{left:?}"));
        }
    }

    #[test]
    fn interval_add() {
        for (left, right) in CASES {
            let result = left + right;
            assert_valid_interval_min_max!(result, format!("{left:?} + {right:?}"));

            let result = right + left;
            assert_valid_interval_min_max!(result, format!("{right:?} + {left:?}"));
        }
    }

    #[test]
    fn interval_sub() {
        for (left, right) in CASES {
            let result = left - right;
            assert_valid_interval_min_max!(result, format!("{left:?} - {right:?}"));

            let result = right - left;
            assert_valid_interval_min_max!(result, format!("{right:?} - {left:?}"));
        }
    }

    #[test]
    fn interval_mul() {
        for (left, right) in CASES {
            let result = left * right;
            assert_valid_interval_min_max!(result, format!("{left:?} * {right:?}"));

            let result = right * left;
            assert_valid_interval_min_max!(result, format!("{right:?} * {left:?}"));
        }
    }

    #[test]
    fn interval_div() {
        for (left, right) in CASES {
            let result = left / right;
            assert_valid_interval_min_max!(result, format!("{left:?} / {right:?}"));

            let result = right / left;
            assert_valid_interval_min_max!(result, format!("{right:?} / {left:?}"));
        }
    }
}
