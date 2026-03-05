#![no_main]

// TODO: make multi-op version of this fuzz target

use libfuzzer_sys::{arbitrary, fuzz_target};
use oneil_output::Interval;

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Abs { val: Interval },
    Acos { val: Interval },
    Add { lhs: Interval, rhs: Interval },
    Asin { val: Interval },
    Atan { val: Interval },
    Ceiling { val: Interval },
    Cos { val: Interval },
    Div { lhs: Interval, rhs: Interval },
    EscapedDiv { lhs: Interval, rhs: Interval },
    EscapedSub { lhs: Interval, rhs: Interval },
    Floor { val: Interval },
    Intersection { lhs: Interval, rhs: Interval },
    Ln { val: Interval },
    Log10 { val: Interval },
    Log2 { val: Interval },
    Mod { lhs: Interval, rhs: Interval },
    ModScalar { lhs: Interval, rhs: f64 },
    Mul { lhs: Interval, rhs: Interval },
    Neg { val: Interval },
    Pow { base: Interval, exponent: Interval },
    Sign { val: Interval },
    Sin { val: Interval },
    Sqrt { val: Interval },
    Sub { lhs: Interval, rhs: Interval },
    Tan { val: Interval },
    TightestEnclosingInterval { lhs: Interval, rhs: Interval },
}

fuzz_target!(|data: FuzzData| {
    let interval_result = match data {
        FuzzData::Abs { val } => val.abs(),
        FuzzData::Acos { val } => val.acos(),
        FuzzData::Add { lhs, rhs } => lhs + rhs,
        FuzzData::Asin { val } => val.asin(),
        FuzzData::Atan { val } => val.atan(),
        FuzzData::Ceiling { val } => val.ceiling(),
        FuzzData::Cos { val } => val.cos(),
        FuzzData::Div { lhs, rhs } => lhs / rhs,
        FuzzData::EscapedDiv { lhs, rhs } => lhs.escaped_div(rhs),
        FuzzData::EscapedSub { lhs, rhs } => lhs.escaped_sub(rhs),
        FuzzData::Floor { val } => val.floor(),
        FuzzData::Intersection { lhs, rhs } => lhs.intersection(rhs),
        FuzzData::Ln { val } => val.ln(),
        FuzzData::Log10 { val } => val.log10(),
        FuzzData::Log2 { val } => val.log2(),
        FuzzData::Mod { lhs, rhs } => lhs % rhs,
        FuzzData::ModScalar { lhs, rhs } => lhs % rhs,
        FuzzData::Mul { lhs, rhs } => lhs * rhs,
        FuzzData::Neg { val } => -val,
        FuzzData::Pow { base, exponent } => base.pow(exponent),
        FuzzData::Sign { val } => val.sign(),
        FuzzData::Sin { val } => val.sin(),
        FuzzData::Sqrt { val } => val.sqrt(),
        FuzzData::Sub { lhs, rhs } => lhs - rhs,
        FuzzData::Tan { val } => val.tan(),
        FuzzData::TightestEnclosingInterval { lhs, rhs } => lhs.tightest_enclosing_interval(rhs),
    };

    assert!(
        interval_result.is_valid(),
        "interval result is not valid: ({:?}, {:?})",
        interval_result.min(),
        interval_result.max()
    );
});
