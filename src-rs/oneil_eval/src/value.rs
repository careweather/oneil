use std::cmp::Ordering;

use crate::unit::Unit;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number { value: NumberValue, unit: Unit },
}

#[derive(Debug, Clone)]
pub enum NumberValue {
    Scalar(f64),
    Interval { min: f64, max: f64 },
}

impl PartialEq for NumberValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NumberValue::Scalar(left), NumberValue::Scalar(right)) => left == right,
            (
                NumberValue::Interval {
                    min: left_min,
                    max: left_max,
                },
                NumberValue::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => left_min == right_min && left_max == right_max,
            (
                NumberValue::Scalar(left),
                NumberValue::Interval {
                    min: right_min,
                    max: right_max,
                },
            ) => left == right_min && left == right_max,
            (
                NumberValue::Interval {
                    min: left_min,
                    max: left_max,
                },
                NumberValue::Scalar(right),
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
            (NumberValue::Scalar(left), NumberValue::Scalar(right)) => left.partial_cmp(right),
            (
                NumberValue::Interval {
                    min: left_min,
                    max: left_max,
                },
                NumberValue::Interval {
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
            (
                NumberValue::Scalar(left),
                NumberValue::Interval {
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
                NumberValue::Interval {
                    min: left_min,
                    max: left_max,
                },
                NumberValue::Scalar(right),
            ) => match (left_min.partial_cmp(right), left_max.partial_cmp(right)) {
                (Some(Ordering::Less), Some(Ordering::Less)) => Some(Ordering::Less),
                (Some(Ordering::Greater), Some(Ordering::Greater)) => Some(Ordering::Greater),
                (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
                _ => None,
            },
        }
    }
}
