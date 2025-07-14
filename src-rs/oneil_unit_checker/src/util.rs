use std::ops::Deref;

use oneil_ir_traverse::ModelMap;
use oneil_unit::{SubmodelTestUnits, TestUnits, Unit};

use crate::error::UnitError;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitMap(ModelMap<(), (), Unit, TestUnits, SubmodelTestUnits>);

impl Deref for UnitMap {
    type Target = ModelMap<(), (), Unit, TestUnits, SubmodelTestUnits>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ModelMap<(), (), Unit, TestUnits, SubmodelTestUnits>> for UnitMap {
    fn from(map: ModelMap<(), (), Unit, TestUnits, SubmodelTestUnits>) -> Self {
        Self(map)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitErrorMap(ModelMap<(), (), Vec<UnitError>, Vec<UnitError>, Vec<UnitError>>);

impl Deref for UnitErrorMap {
    type Target = ModelMap<(), (), Vec<UnitError>, Vec<UnitError>, Vec<UnitError>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ModelMap<(), (), Vec<UnitError>, Vec<UnitError>, Vec<UnitError>>> for UnitErrorMap {
    fn from(map: ModelMap<(), (), Vec<UnitError>, Vec<UnitError>, Vec<UnitError>>) -> Self {
        Self(map)
    }
}
