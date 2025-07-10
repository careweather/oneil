use oneil_ir::model::ModelCollection;

pub type UnitMap = oneil_shared::ModelMap<oneil_unit::Unit>;

pub fn check_units(_model_collection: &ModelCollection) -> Result<UnitMap, ()> {
    // TODO: assign units to parameters
    // TODO: check that units are consistent
    todo!()
}
