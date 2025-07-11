//! # Oneil IR Traverse
//!
//! This crate provides functionality for traversing Oneil IR model collections and processing
//! their components (python imports, submodels, parameters, model tests, and submodel tests)
//! using customizable processors.
//!
//! The main entry point is the `traverse` function, which takes a model collection and
//! processor implementations for each component type. It returns a `ModelMap` containing
//! the processed data or errors for each component.

#![warn(missing_docs)]

use oneil_ir::model::ModelCollection;

use crate::builder::ModelMapBuilder;

mod builder;
mod model_map;
pub mod traits;

pub use model_map::{ModelMap, ModelMapEntry};
pub use traits::{
    ModelTestProcess, ParameterProcess, PythonImportProcess, SubmodelProcess, SubmodelTestProcess,
};

/// Traverses a model collection and processes its components using the provided processors.
///
/// This function iterates through all models in evaluation order and processes each in the following order:
/// 1. Python imports
/// 2. Submodels
/// 3. Parameters
/// 4. Model tests
/// 5. Submodel tests
///
/// The processors are called in the order of the above list.
///
/// Parameters are processed in evaluation order. Other items do not have a
/// defined order.
///
/// # Arguments
///
/// * `model_collection` - The collection of models to traverse
/// * `python_import_process` - Processor for python imports
/// * `submodel_process` - Processor for submodels
/// * `parameter_process` - Processor for parameters
/// * `model_test_process` - Processor for model tests
/// * `submodel_test_process` - Processor for submodel tests
///
/// # Returns
///
/// Returns a `Result` containing either:
/// * `Ok(ModelMap)` - All components processed successfully
/// * `Err((ModelMap, ModelMap))` - Some components failed, with the first map containing
///   successful results and the second containing errors
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
    // create the model map builder
    let model_map_builder = ModelMapBuilder::new();

    // traverse the models in evaluation order
    let model_map_builder = model_collection
        .get_model_evaluation_order()
        .into_iter()
        .fold(model_map_builder, |builder, model_path| {
            // get the model
            let model = model_collection
                .models()
                .get(model_path)
                .expect("model should exist");

            // traverse the python imports
            let builder =
                model
                    .get_python_imports()
                    .iter()
                    .fold(builder, |mut builder, python_path| {
                        // process the python import
                        let result = python_import_process.process(python_path);

                        // add the python import to the model map
                        match result {
                            Ok(output) => builder.add_python_import_data(
                                model_path.clone(),
                                python_path.clone(),
                                output,
                            ),
                            Err(error) => builder.add_python_import_error(
                                model_path.clone(),
                                python_path.clone(),
                                error,
                            ),
                        };
                        builder
                    });

            // traverse the submodels
            let builder = model.get_submodels().into_iter().fold(
                builder,
                |mut builder, (submodel_id, submodel_path)| {
                    // process the submodel
                    let result = submodel_process.process(submodel_id, submodel_path);

                    // add the submodel to the model map
                    match result {
                        Ok(output) => builder.add_submodel_data(
                            model_path.clone(),
                            submodel_id.clone(),
                            output,
                        ),
                        Err(error) => builder.add_submodel_error(
                            model_path.clone(),
                            submodel_id.clone(),
                            error,
                        ),
                    };
                    builder
                },
            );

            // traverse the parameters
            let builder = model.get_parameter_evaluation_order().into_iter().fold(
                builder,
                |mut builder, parameter| {
                    // process the parameter
                    let result = parameter_process.process(parameter);

                    // add the parameter to the model map
                    match result {
                        Ok(output) => builder.add_parameter_data(
                            model_path.clone(),
                            parameter.identifier().clone(),
                            output,
                        ),
                        Err(error) => builder.add_parameter_error(
                            model_path.clone(),
                            parameter.identifier().clone(),
                            error,
                        ),
                    };
                    builder
                },
            );

            // traverse the model tests
            let builder = model.get_model_tests().into_iter().fold(
                builder,
                |mut builder, (test_index, model_test)| {
                    // process the model test
                    let result = model_test_process.process(test_index, model_test);

                    // add the model test to the model map
                    match result {
                        Ok(output) => builder.add_model_test_data(
                            model_path.clone(),
                            test_index.clone(),
                            output,
                        ),
                        Err(error) => builder.add_model_test_error(
                            model_path.clone(),
                            test_index.clone(),
                            error,
                        ),
                    };
                    builder
                },
            );

            // traverse the submodel tests
            let builder = model.get_submodel_tests().into_iter().fold(
                builder,
                |mut builder, submodel_test| {
                    // process the submodel test
                    let result = submodel_test_process.process(submodel_test);

                    // add the submodel test to the model map
                    match result {
                        Ok(output) => builder.add_submodel_test_data(
                            model_path.clone(),
                            submodel_test.submodel_identifier().clone(),
                            output,
                        ),
                        Err(error) => builder.add_submodel_test_error(
                            model_path.clone(),
                            submodel_test.submodel_identifier().clone(),
                            error,
                        ),
                    };
                    builder
                },
            );

            builder
        });

    model_map_builder.try_into()
}
