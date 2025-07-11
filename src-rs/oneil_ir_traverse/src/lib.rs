use oneil_ir::model::ModelCollection;

use crate::{
    builder::ModelMapBuilder,
    model_map::ModelMap,
    traits::{
        ModelTestProcess, ParameterProcess, PythonImportProcess, SubmodelProcess,
        SubmodelTestProcess,
    },
};

mod builder;
mod model_map;
mod traits;

pub fn traverse<PythonImportP, SubmodelP, ParameterP, ModelTestP, SubmodelTestP>(
    model_collection: &ModelCollection,
    python_import_process: PythonImportP,
    submodel_process: SubmodelP,
    parameter_process: ParameterP,
    model_test_process: ModelTestP,
    submodel_test_process: SubmodelTestP,
) -> Result<
    ModelMap<
        PythonImportP::Output,
        SubmodelP::Output,
        ParameterP::Output,
        ModelTestP::Output,
        SubmodelTestP::Output,
    >,
    (
        ModelMap<
            PythonImportP::Output,
            SubmodelP::Output,
            ParameterP::Output,
            ModelTestP::Output,
            SubmodelTestP::Output,
        >,
        ModelMap<
            PythonImportP::Error,
            SubmodelP::Error,
            ParameterP::Error,
            ModelTestP::Error,
            SubmodelTestP::Error,
        >,
    ),
>
where
    PythonImportP: PythonImportProcess,
    SubmodelP: SubmodelProcess,
    ParameterP: ParameterProcess,
    ModelTestP: ModelTestProcess,
    SubmodelTestP: SubmodelTestProcess,
{
    let mut model_map_builder = ModelMapBuilder::new();

    todo!();

    model_map_builder.try_into()
}
