use oneil_ir::model::ModelCollection;
use oneil_unit::Unit;

use crate::util::{UnitErrorMap, UnitMap};

mod convert;
mod error;
mod infer;
mod process;
mod util;

use crate::process::{ParameterChecker, SubmodelTestChecker, TestChecker};

pub fn check_units(model_collection: &ModelCollection) -> Result<UnitMap, (UnitMap, UnitErrorMap)> {
    let parameter_checker = ParameterChecker::new();
    let test_checker = TestChecker::new();
    let submodel_test_checker = SubmodelTestChecker::new();

    oneil_ir_traverse::traverse(
        model_collection,
        (),
        (),
        parameter_checker,
        test_checker,
        submodel_test_checker,
    )
    .map(|map| map.into())
    .map_err(|(map, errors)| (map.into(), errors.into()))
}
