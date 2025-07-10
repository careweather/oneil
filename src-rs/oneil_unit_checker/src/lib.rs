use std::ops::Deref;

use oneil_ir::model::ModelCollection;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitMap(oneil_shared::ModelMap<oneil_unit::Unit>);

impl Deref for UnitMap {
    type Target = oneil_shared::ModelMap<oneil_unit::Unit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitMapBuilder(oneil_shared::ModelMapBuilder<oneil_unit::Unit>);

impl Deref for UnitMapBuilder {
    type Target = oneil_shared::ModelMapBuilder<oneil_unit::Unit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub fn check_units(model_collection: &ModelCollection) -> Result<UnitMap, ()> {
    for _model in model_collection.models() {
        todo!()
    }

    todo!()
}
