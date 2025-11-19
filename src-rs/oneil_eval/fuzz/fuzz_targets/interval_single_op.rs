#![no_main]

use oneil_eval::interval::Interval;

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Add { lhs: Interval, rhs: Interval },
    Div { lhs: Interval, rhs: Interval },
    Mul { lhs: Interval, rhs: Interval },
    Neg { val: Interval },
    Pow { base: Interval, exponent: Interval },
    Sub { lhs: Interval, rhs: Interval },
}

fuzz_target!(|data: FuzzData| {
    match data {
        FuzzData::Add { lhs, rhs } => {
            let result = lhs + rhs;
            assert!(result.is_valid());
        }
        FuzzData::Div { lhs, rhs } => {
            let result = lhs / rhs;
            assert!(result.is_valid());
        }
        FuzzData::Mul { lhs, rhs } => {
            let result = lhs * rhs;
            assert!(result.is_valid());
        }
        FuzzData::Neg { val } => {
            let result = -val;
            assert!(result.is_valid());
        }
        FuzzData::Pow { base, exponent } => {
            let result = base.pow(&exponent);
            assert!(result.is_valid());
        }
        FuzzData::Sub { lhs, rhs } => {
            let result = lhs - rhs;
            assert!(result.is_valid());
        }
    }
});
