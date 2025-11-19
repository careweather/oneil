#![no_main]

use oneil_eval::interval::Interval;

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Add(Interval, Interval),
}

fuzz_target!(|data: FuzzData| {
    match data {
        FuzzData::Add(lhs, rhs) => {
            let result = lhs + rhs;
            assert!(result.is_valid());
        }
    }
});
