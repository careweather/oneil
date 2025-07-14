use oneil_ir::test::{Test, TestIndex};
use oneil_ir_traverse::ProcessTest;
use oneil_unit::TestUnits;

use crate::error::UnitError;

pub struct TestChecker;

impl TestChecker {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessTest for TestChecker {
    type Output = TestUnits;

    type Error = Vec<UnitError>;

    fn process(&self, test_index: &TestIndex, test: &Test) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}
