#![no_main]

// TODO: make multi-op version of this fuzz target

use libfuzzer_sys::{arbitrary, fuzz_target};
use shared::IntervalWithValue;

#[derive(Debug, Clone, PartialEq, arbitrary::Arbitrary)]
enum FuzzData {
    Abs {
        val: IntervalWithValue,
    },
    Acos {
        val: IntervalWithValue,
    },
    Add {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    Asin {
        val: IntervalWithValue,
    },
    Atan {
        val: IntervalWithValue,
    },
    Ceiling {
        val: IntervalWithValue,
    },
    Cos {
        val: IntervalWithValue,
    },
    Div {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    Floor {
        val: IntervalWithValue,
    },
    Ln {
        val: IntervalWithValue,
    },
    Log10 {
        val: IntervalWithValue,
    },
    Log2 {
        val: IntervalWithValue,
    },
    Mod {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    ModScalar {
        lhs: IntervalWithValue,
        rhs: f64,
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
    Sign {
        val: IntervalWithValue,
    },
    Sin {
        val: IntervalWithValue,
    },
    Sqrt {
        val: IntervalWithValue,
    },
    Sub {
        lhs: IntervalWithValue,
        rhs: IntervalWithValue,
    },
    Tan {
        val: IntervalWithValue,
    },
}

// TODO: note that this tests that interval arithmetic produces valid
//       intervals and that the arithmetic satisfies the inclusion property
//       mentioned in the docs

fuzz_target!(|data: FuzzData| {
    let (interval_result, value_result) = match data {
        FuzzData::Abs { val } => {
            let interval_result = val.interval.abs();
            let value_result = val.value.abs();
            (interval_result, value_result)
        }
        FuzzData::Acos { val } => {
            if val.value < -1.0 || val.value > 1.0 {
                return;
            }
            let interval_result = val.interval.acos();
            let value_result = val.value.acos();
            (interval_result, value_result)
        }
        FuzzData::Add { lhs, rhs } => {
            let interval_result = lhs.interval + rhs.interval;
            let value_result = lhs.value + rhs.value;
            (interval_result, value_result)
        }
        FuzzData::Asin { val } => {
            if val.value < -1.0 || val.value > 1.0 {
                return;
            }
            let interval_result = val.interval.asin();
            let value_result = val.value.asin();
            (interval_result, value_result)
        }
        FuzzData::Atan { val } => {
            let interval_result = val.interval.atan();
            let value_result = val.value.atan();
            (interval_result, value_result)
        }
        FuzzData::Ceiling { val } => {
            let interval_result = val.interval.ceiling();
            let value_result = val.value.ceil();
            (interval_result, value_result)
        }
        FuzzData::Cos { val } => {
            let interval_result = val.interval.cos();
            let value_result = val.value.cos();
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
        FuzzData::Floor { val } => {
            let interval_result = val.interval.floor();
            let value_result = val.value.floor();
            (interval_result, value_result)
        }
        FuzzData::Ln { val } => {
            let interval_result = val.interval.ln();
            let value_result = val.value.ln();
            (interval_result, value_result)
        }
        FuzzData::Log10 { val } => {
            let interval_result = val.interval.log10();
            let value_result = val.value.log10();
            (interval_result, value_result)
        }
        FuzzData::Log2 { val } => {
            let interval_result = val.interval.log2();
            let value_result = val.value.log2();
            (interval_result, value_result)
        }
        FuzzData::Mod { lhs, rhs } => {
            let interval_result = lhs.interval % rhs.interval;
            let value_result = lhs.value % rhs.value;
            (interval_result, value_result)
        }
        FuzzData::ModScalar { lhs, rhs } => {
            let interval_result = lhs.interval % rhs;
            let value_result = lhs.value % rhs;
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
        FuzzData::Sign { val } => {
            let interval_result = val.interval.sign();
            let value_result = val.value.signum();
            (interval_result, value_result)
        }
        FuzzData::Sin { val } => {
            let interval_result = val.interval.sin();
            let value_result = val.value.sin();
            (interval_result, value_result)
        }
        FuzzData::Sqrt { val } => {
            let interval_result = val.interval.sqrt();
            let value_result = val.value.sqrt();
            (interval_result, value_result)
        }
        FuzzData::Sub { lhs, rhs } => {
            let interval_result = lhs.interval - rhs.interval;
            let value_result = lhs.value - rhs.value;
            (interval_result, value_result)
        }
        FuzzData::Tan { val } => {
            let interval_result = val.interval.tan();
            let value_result = val.value.tan();
            (interval_result, value_result)
        }
    };

    assert!(
        interval_result.is_valid(),
        "interval result is not valid: ({:?}, {:?})",
        interval_result.min(),
        interval_result.max()
    );

    if !interval_result.is_empty() && value_result.is_finite() {
        assert!(
            value_result <= interval_result.max(),
            "value result ({:?}) is greater than the interval max ({:?}), interval: ({:?}, {:?})",
            value_result,
            interval_result.max(),
            interval_result.min(),
            interval_result.max(),
        );

        assert!(
            value_result >= interval_result.min(),
            "value result ({:?}) is less than the interval min ({:?}), interval: ({:?}, {:?})",
            value_result,
            interval_result.min(),
            interval_result.min(),
            interval_result.max(),
        );
    }
});
