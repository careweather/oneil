#![no_main]

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(arbitrary::Arbitrary)]
enum FuzzData {
    CompareSameString(String),
    CompareDifferentString(String, String),
}

fuzz_target!(|data: &[u8]| { todo!() });
