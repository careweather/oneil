use oneil_ir::model::ModelCollection;
use oneil_shared::ModelMap;
use oneil_unit::Unit;

use crate::util::{UnitMap, UnitMapBuilder};

mod convert;
mod infer;
mod util;

pub fn check_units(model_collection: &ModelCollection) -> Result<UnitMap, (UnitMap, ModelMap<()>)> {
}
