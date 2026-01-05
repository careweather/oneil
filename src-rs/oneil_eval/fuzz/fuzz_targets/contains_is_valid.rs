#![no_main]

use libfuzzer_sys::{arbitrary, fuzz_target};
use oneil_eval::value::Interval;

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
pub struct FuzzData {
    interval1: Interval,
    interval2: Interval,
}

fuzz_target!(|data: FuzzData| {
    let interval1_contains_interval2 = data.interval1.contains(&data.interval2);
    let interval_intersection = data.interval1.intersection(data.interval2);
    let interval_intersection_is_interval2 = interval_intersection == data.interval2;
    assert!(
        interval1_contains_interval2 == interval_intersection_is_interval2,
        "interval1 contains interval2 ({:?}) but interval intersection ({:?}) is not equal to interval2 ({:?})",
        interval1_contains_interval2,
        interval_intersection,
        data.interval2,
    );
});
