#![no_main]

// TODO: make multi-op version of this fuzz target

use libfuzzer_sys::{arbitrary, fuzz_target};
use oneil_eval::value::Interval;

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Add { lhs: Interval, rhs: Interval },
    Div { lhs: Interval, rhs: Interval },
    EscapedSub { lhs: Interval, rhs: Interval },
    EscapedDiv { lhs: Interval, rhs: Interval },
    Intersection { lhs: Interval, rhs: Interval },
    Ln { val: Interval },
    Log10 { val: Interval },
    Log2 { val: Interval },
    Mod { lhs: Interval, rhs: Interval },
    Mul { lhs: Interval, rhs: Interval },
    Neg { val: Interval },
    Pow { base: Interval, exponent: Interval },
    Sqrt { val: Interval },
    Sub { lhs: Interval, rhs: Interval },
}

fuzz_target!(|data: FuzzData| {
    let interval_result = match data {
        FuzzData::Add { lhs, rhs } => lhs + rhs,
        FuzzData::Div { lhs, rhs } => lhs / rhs,
        FuzzData::EscapedSub { lhs, rhs } => lhs.escaped_sub(rhs),
        FuzzData::EscapedDiv { lhs, rhs } => lhs.escaped_div(rhs),
        FuzzData::Intersection { lhs, rhs } => lhs.intersection(rhs),
        FuzzData::Ln { val } => val.ln(),
        FuzzData::Log10 { val } => val.log10(),
        FuzzData::Log2 { val } => val.log2(),
        FuzzData::Mod { lhs, rhs } => lhs % rhs,
        FuzzData::Mul { lhs, rhs } => lhs * rhs,
        FuzzData::Neg { val } => -val,
        FuzzData::Pow { base, exponent } => base.pow(exponent),
        FuzzData::Sqrt { val } => val.sqrt(),
        FuzzData::Sub { lhs, rhs } => lhs - rhs,
    };

    assert!(
        interval_result.is_valid(),
        "interval result is not valid: ({:?}, {:?})",
        interval_result.min(),
        interval_result.max()
    );
});
