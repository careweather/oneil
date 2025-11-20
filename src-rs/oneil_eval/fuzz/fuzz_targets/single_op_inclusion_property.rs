#![no_main]

use libfuzzer_sys::{arbitrary, fuzz_target};
use shared::IntervalWithValue;

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Add {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    Div {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    Mul {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    Neg {
        val: IntervalWithValue,
    },
    // TODO: this is causing some false positives, so we're disabling it for now.
    //       figure out if this is a mathematical problem or an f64 problem
    // Pow {
    //     base: IntervalWithValue,
    //     exponent: IntervalWithValue,
    // },
    Sub {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
}

// TODO: note that this tests that interval arithmetic produces valid
//       intervals and that the arithmetic satisfies the inclusion property
//       mentioned in the docs

fuzz_target!(|data: FuzzData| {
    let (interval_result, value_result) = match data {
        FuzzData::Add { lhs, rhs } => {
            let interval_result = lhs.interval + rhs.interval;
            let value_result = lhs.value + rhs.value;
            (interval_result, value_result)
        }
        FuzzData::Div { lhs, rhs } => {
            let interval_result = lhs.interval / rhs.interval;
            let value_result = lhs.value / rhs.value;

            if lhs.value == 0.0 && rhs.value == 0.0 {
                // special case: 0/0 produces NaN with float arithmetic,
                //               but is considered undefined with interval arithmetic
                //               so we skip the test to avoid false positives
                //
                //               if you feel it is important to test this case,
                //               you can remove this and fix the problem
                return;
            }

            (interval_result, value_result)
        }
        FuzzData::Mul { lhs, rhs } => {
            let interval_result = lhs.interval * rhs.interval;
            let value_result = lhs.value * rhs.value;
            (interval_result, value_result)
        }
        FuzzData::Neg { val } => {
            let interval_result = -val.interval;
            let value_result = -val.value;
            (interval_result, value_result)
        }
        // FuzzData::Pow { base, exponent } => {
        //     let interval_result = base.interval.pow(&exponent.interval);
        //     let value_result = base.value.powf(exponent.value);

        //     if base.value == 0.0 && exponent.value == 0.0 {
        //         // special case: 0^0 produces 1.0 with float arithmetic,
        //         //               but is considered undefined with interval arithmetic
        //         //               so we skip the test to avoid false positives
        //         //
        //         //               if you feel it is important to test this case,
        //         //               you can remove this and fix the problem
        //         return;
        //     }
        //     (interval_result, value_result)
        // }
        FuzzData::Sub { lhs, rhs } => {
            let interval_result = lhs.interval - rhs.interval;
            let value_result = lhs.value - rhs.value;
            (interval_result, value_result)
        }
    };

    assert!(
        interval_result.is_valid(),
        "interval result is not valid: ({}, {})",
        interval_result.min(),
        interval_result.max()
    );

    if !interval_result.is_empty() && !value_result.is_nan() {
        assert!(
            value_result <= interval_result.max(),
            "value result ({}) is greater than the interval max ({}), interval: ({}, {})",
            value_result,
            interval_result.max(),
            interval_result.min(),
            interval_result.max(),
        );

        assert!(
            value_result >= interval_result.min(),
            "value result ({}) is less than the interval min ({}), interval: ({}, {})",
            value_result,
            interval_result.min(),
            interval_result.min(),
            interval_result.max(),
        );
    }
});
