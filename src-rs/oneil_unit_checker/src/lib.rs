use oneil_ir::model::ModelCollection;
use oneil_shared::ModelMap;
use oneil_unit::Unit;

use crate::util::{UnitMap, UnitMapBuilder};

mod convert;
mod infer;
mod util;

pub fn check_units(model_collection: &ModelCollection) -> Result<UnitMap, (UnitMap, ModelMap<()>)> {
    let unit_map_builder = UnitMapBuilder::new();

    let unit_map_builder = model_collection.get_model_evaluation_order().iter().fold(
        unit_map_builder,
        |builder, model_path| {
            let model = model_collection
                .models()
                .get(model_path)
                .expect("model should exist");

            model
                .get_parameter_evaluation_order()
                .iter()
                .fold(builder, |builder, parameter_id| {
                    let parameter = model
                        .get_parameter(parameter_id)
                        .expect("parameter should exist");

                    let parameter_unit = parameter.unit();
                    let parameter_unit = parameter_unit
                        .map(convert::convert_unit)
                        .unwrap_or(Unit::empty());

                    let inferred_units = infer::infer_units(parameter_id, &builder);

                    todo!()
                })
        },
    );

    unit_map_builder.try_into()
}
