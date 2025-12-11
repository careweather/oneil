use libfuzzer_sys::arbitrary::{self, Result, Unstructured};
use oneil_eval::value::Interval;

#[derive(Debug, Clone, PartialEq)]
pub struct IntervalWithValue {
    pub interval: Interval,
    pub value: f64,
}

fn pick_value(interval: &Interval, seed: f64) -> f64 {
    let value = if interval.is_empty() {
        f64::NAN
    } else if interval.min() == interval.max() {
        interval.min()
    } else if seed.is_infinite() && seed.is_sign_positive() {
        // positive infinity => max
        interval.max()
    } else if seed.is_infinite() && seed.is_sign_negative() {
        // negative infinity => min
        interval.min()
    } else if seed.is_nan() {
        // nan => midpoint
        (interval.min() / 2.0) + (interval.max() / 2.0)
    } else {
        let difference = interval.max() - interval.min();
        interval.min() + (seed.abs() % difference)
    };

    if !value.is_nan() {
        assert!(
            value >= interval.min(),
            "value ({:?}) is less than the interval min ({:?}), interval: ({:?}, {:?})",
            value,
            interval.min(),
            interval.min(),
            interval.max()
        );
        assert!(
            value <= interval.max(),
            "value ({:?}) is greater than the interval max ({:?}), interval: ({:?}, {:?})",
            value,
            interval.max(),
            interval.min(),
            interval.max()
        );
    }

    value
}

impl<'a> arbitrary::Arbitrary<'a> for IntervalWithValue {
    fn arbitrary(u: &mut Unstructured) -> Result<Self> {
        let interval = u.arbitrary::<Interval>()?;

        let value_seed = u.arbitrary::<f64>()?;
        let value = pick_value(&interval, value_seed);
        Ok(IntervalWithValue { interval, value })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonNanF64(f64);

impl<'a> arbitrary::Arbitrary<'a> for NonNanF64 {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let value = u.arbitrary::<f64>()?;

        if value.is_nan() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(NonNanF64(value))
    }
}
