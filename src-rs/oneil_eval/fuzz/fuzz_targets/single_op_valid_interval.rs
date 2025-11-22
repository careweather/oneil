#![no_main]

// TODO: make multi-op version of this fuzz target

use libfuzzer_sys::{arbitrary, fuzz_target};
use oneil_eval::interval::Interval;

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Add { lhs: Interval, rhs: Interval },
    Div { lhs: Interval, rhs: Interval },
    Intersection { lhs: Interval, rhs: Interval },
    Mod { lhs: Interval, rhs: Interval },
    Mul { lhs: Interval, rhs: Interval },
    Neg { val: Interval },
    Pow { base: Interval, exponent: Interval },
    Sub { lhs: Interval, rhs: Interval },
}

fuzz_target!(|data: FuzzData| {
    let interval_result = match data {
        FuzzData::Add { lhs, rhs } => lhs + rhs,
        FuzzData::Div { lhs, rhs } => lhs / rhs,
        FuzzData::Intersection { lhs, rhs } => lhs.intersection(&rhs),
        FuzzData::Mod { lhs, rhs } => lhs % rhs,
        FuzzData::Mul { lhs, rhs } => lhs * rhs,
        FuzzData::Neg { val } => -val,
        FuzzData::Pow { base, exponent } => base.pow(&exponent),
        FuzzData::Sub { lhs, rhs } => lhs - rhs,
    };

    assert!(
        interval_result.is_valid(),
        "interval result is not valid: ({:?}, {:?})",
        interval_result.min(),
        interval_result.max()
    );
});
