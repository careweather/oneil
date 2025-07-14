use oneil_ir::parameter::Parameter;
use oneil_ir_traverse::ProcessParameter;
use oneil_unit::Unit;

use crate::error::UnitError;

pub struct ParameterChecker;

impl ParameterChecker {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessParameter for ParameterChecker {
    type Output = Unit;

    type Error = Vec<UnitError>;

    fn process(&self, parameter: &Parameter) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}
