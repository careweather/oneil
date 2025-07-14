use std::collections::HashMap;

use oneil_ir::{reference::Identifier, test::SubmodelTest};
use oneil_ir_traverse::ProcessSubmodelTest;
use oneil_unit::{SubmodelTestUnits, Unit};

use crate::error::UnitError;

pub struct SubmodelTestChecker;

impl SubmodelTestChecker {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessSubmodelTest for SubmodelTestChecker {
    type Output = SubmodelTestUnits;

    type Error = Vec<UnitError>;

    fn process(&self, submodel_test: &SubmodelTest) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}
