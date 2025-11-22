use crate::interval::Interval;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalClass {
    /// An empty interval (represented by both fields as `NaN`)
    Empty,
    /// min > 0
    Positive1,
    /// min = 0 and max > 0
    Positive0,
    /// min = max = 0
    Zero,
    /// min < 0 < max
    Mixed,
    /// min < 0 and max = 0
    Negative0,
    /// max < 0
    Negative1,
}

pub fn classify(interval: &Interval) -> IntervalClass {
    if interval.is_empty() {
        IntervalClass::Empty
    } else if interval.min > 0.0 {
        IntervalClass::Positive1
    } else if interval.min == 0.0 && interval.max > 0.0 {
        IntervalClass::Positive0
    } else if interval.min == 0.0 && interval.max == 0.0 {
        IntervalClass::Zero
    } else if interval.min < 0.0 && interval.max > 0.0 {
        IntervalClass::Mixed
    } else if interval.min < 0.0 && interval.max == 0.0 {
        IntervalClass::Negative0
    } else if interval.max < 0.0 {
        IntervalClass::Negative1
    } else {
        panic!("invalid interval: ({}, {})", interval.min, interval.max)
    }
}
