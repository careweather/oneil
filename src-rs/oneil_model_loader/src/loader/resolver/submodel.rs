//! Submodel resolution for the Oneil model loader
//!
//! This module provides functionality for resolving submodels in Oneil models.
//! Submodel resolution involves processing `use` and `from` declarations to
//! establish relationships between models and their submodels.
//!
//! # Overview
//!
//! Submodels in Oneil allow models to reference and use other models as
//! components. This is achieved through two main declaration types:
//!
//! ## Use Declarations
//! ```oneil
//! use model_name.subcomponent as alias
//! use model_name(x=1, y=2) as alias
//! ```
//!
//! ## From Declarations
//! ```oneil
//! from parent_model use submodel as alias
//! from parent_model use submodel(x=1, y=2) as alias
//! ```
//!
//! # Resolution Process
//!
//! The resolution process involves:
//! 1. **Path Construction**: Building the model path from the declaration
//! 2. **Model Loading**: Ensuring the target model is loaded and available
//! 3. **Subcomponent Navigation**: Traversing nested subcomponents if specified
//! 4. **Alias Assignment**: Mapping the resolved path to the specified alias
//! 5. **Test Input Collection**: Gathering any test inputs for later resolution
//!
//! # Error Handling
//!
//! The model provides comprehensive error handling for various failure scenarios:
//! - **Model Loading Errors**: When the target model fails to load
//! - **Undefined Submodels**: When a submodel doesn't exist in the target model
//! - **Nested Resolution Errors**: When subcomponents in the path don't exist
//!
//! # Examples
//!
//! ## Simple Submodel Usage
//! ```oneil
//! use temperature_model as temp
//! ```
//! This creates a submodel named `temp` that references the `temperature_model.on` file.
//!
//! ## Nested Submodel Access
//! ```oneil
//! use weather_model.atmosphere.temperature as temp
//! ```
//! This navigates through `weather_model.on` → `atmosphere` submodel → `temperature` submodel.
//!
//! ## Submodel with Test Inputs
//! ```oneil
//! use sensor_model(location="north", height=100) as sensor
//! ```
//! This creates a submodel with predefined test inputs for later evaluation.

use std::collections::HashMap;

use oneil_ast as ast;
use oneil_ir::reference::{Identifier, ModelPath};

use crate::{error::SubmodelResolutionError, loader::resolver::ModelInfo, util::info::InfoResult};

/// Resolves submodels and their associated tests from use model declarations.
///
/// This function processes a collection of `UseModel` declarations and resolves
/// them into submodel mappings, test inputs, and any resolution errors that occur.
///
/// # Arguments
///
/// * `use_models` - A vector of use model declarations to resolve
/// * `model_path` - The path of the current model being processed
/// * `model_info` - Information about all available models and their loading status
///
/// # Returns
///
/// A tuple containing:
/// * `HashMap<Identifier, ModelPath>` - Successfully resolved submodels mapped to their paths
/// * `Vec<(Identifier, Vec<ModelInput>)>` - Submodel test inputs for later resolution
/// * `HashMap<Identifier, SubmodelResolutionError>` - Any resolution errors that occurred
///
/// # Error Handling
///
/// The function handles various error conditions gracefully:
/// - **Model loading failures**: If a referenced model failed to load
/// - **Undefined submodels**: If a submodel doesn't exist in the target model
/// - **Invalid paths**: If the model path cannot be resolved
///
/// All errors are collected and returned rather than causing the function to fail.
pub fn resolve_submodels_and_tests(
    use_models: Vec<ast::declaration::UseModel>,
    model_path: &ModelPath,
    model_info: &ModelInfo,
) -> (
    HashMap<Identifier, ModelPath>,
    Vec<(Identifier, Vec<ast::declaration::ModelInput>)>,
    HashMap<Identifier, SubmodelResolutionError>,
) {
    use_models.into_iter().fold(
        (HashMap::new(), Vec::new(), HashMap::new()),
        |(mut submodels, mut submodel_tests, mut resolution_errors), use_model| {
            // get the use model path
            let use_model_path = model_path.get_sibling_path(&use_model.model_name);
            let use_model_path = ModelPath::new(use_model_path);

            // get the submodel name
            let submodel_name = use_model.as_name.as_ref().unwrap_or(
                use_model
                    .subcomponents
                    .last()
                    .unwrap_or(&use_model.model_name),
            );
            let submodel_name = Identifier::new(submodel_name);

            // resolve the use model path
            let resolved_use_model_path =
                resolve_model_path(use_model_path.clone(), &use_model.subcomponents, model_info);

            // insert the use model path into the submodels map if it was resolved successfully
            // otherwise, add the error to the builder
            match resolved_use_model_path {
                Ok(resolved_use_model_path) => {
                    submodels.insert(submodel_name.clone(), resolved_use_model_path.clone());

                    // store the inputs for the submodel tests
                    // (the inputs are stored in their AST form for now and converted to
                    // the model input type once all the submodels have been resolved)
                    let inputs = use_model.inputs.unwrap_or_default();
                    submodel_tests.push((submodel_name, inputs));
                }
                Err(error) => {
                    resolution_errors.insert(submodel_name, error);
                }
            }

            (submodels, submodel_tests, resolution_errors)
        },
    )
}

/// Recursively resolves a model path by traversing subcomponents.
///
/// This internal function handles the recursive resolution of model paths
/// when dealing with nested submodels (e.g., `parent.submodel1.submodel2`).
/// It traverses the subcomponent chain and validates that each level exists.
///
/// # Arguments
///
/// * `parent_model_path` - The path of the parent model (None for root resolution)
/// * `model_path` - The current model path being resolved
/// * `subcomponents` - The remaining subcomponents to traverse
/// * `model_info` - Information about all available models
///
/// # Returns
///
/// * `Ok(ModelPath)` - The fully resolved model path
/// * `Err(SubmodelResolutionError)` - An error if resolution fails
///
/// # Examples
///
/// For a path like `weather.atmosphere.temperature`:
/// 1. First call: `resolve_model_path(None, "weather", ["atmosphere", "temperature"], ...)`
/// 2. Second call: `resolve_model_path(Some("weather"), "atmosphere", ["temperature"], ...)`
/// 3. Third call: `resolve_model_path(Some("atmosphere"), "temperature", [], ...)`
/// 4. Returns: `Ok("temperature")`
///
/// # Error Conditions
///
/// * **Model has error**: If the target model failed to load
/// * **Undefined submodel**: If a subcomponent doesn't exist in the parent model
/// * **Invalid model**: If the model doesn't exist in model_info
///
/// # Safety
///
/// This function assumes that models referenced in `model_info` have been
/// properly loaded and validated. If this assumption is violated, the function
/// will panic, indicating a bug in the model loading process.
fn resolve_model_path(
    model_path: ModelPath,
    subcomponents: &[String],
    model_info: &ModelInfo,
) -> Result<ModelPath, SubmodelResolutionError> {
    // if the model that we are trying to resolve has had an error, this
    // operation should fail
    let model = match model_info.get(&model_path) {
        InfoResult::Found(model) => model,
        InfoResult::HasError => return Err(SubmodelResolutionError::model_has_error(model_path)),
        InfoResult::NotFound => panic!("model should have been visited already"),
    };

    // if there are no more subcomponents, we have resolved the model path
    if subcomponents.is_empty() {
        return Ok(model_path);
    }

    let submodel_name = Identifier::new(subcomponents[0].clone());
    let submodel_path = model
        .get_submodel(&submodel_name)
        .ok_or(SubmodelResolutionError::undefined_submodel_in_submodel(
            model_path.clone(),
            submodel_name,
        ))?
        .clone();

    resolve_model_path(submodel_path, &subcomponents[1..], model_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ast::declaration::{ModelInput, UseModel};
    use oneil_ast::expression::{Expr, Literal};
    use oneil_ir::model::Model;
    use std::collections::HashSet;

    /// Creates a test model with specified submodels
    fn create_test_model(submodels: Vec<(&str, ModelPath)>) -> Model {
        let mut submodel_map = HashMap::new();
        for (name, path) in submodels {
            submodel_map.insert(Identifier::new(name), path);
        }

        Model::new(
            HashSet::new(),                                                // python_imports
            submodel_map,                                                  // submodels
            oneil_ir::parameter::ParameterCollection::new(HashMap::new()), // parameters
            HashMap::new(),                                                // tests
            Vec::new(),                                                    // submodel_tests
        )
    }

    #[test]
    fn test_resolve_simple_submodel() {
        // build the use model list
        let use_models = vec![
            // use temperature as temp
            UseModel {
                model_name: "temperature".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: Some("temp".to_string()),
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_submodel = create_test_model(vec![]);
        let model_map = HashMap::from([(&temperature_path, &temperature_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert!(errors.is_empty());

        // submodel tests
        assert_eq!(submodels.len(), 1);

        let submodel_path = submodels.get(&temperature_id);
        assert_eq!(submodel_path, Some(&temperature_path));

        // test inputs tests
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &temperature_id);
        assert!(test_inputs.is_empty()); // no inputs
    }

    #[test]
    fn test_resolve_submodel_with_inputs() {
        // build the use model list
        let use_models = vec![
            // use sensor(location="north", height=100) as sensor
            UseModel {
                model_name: "sensor".to_string(),
                subcomponents: vec![],
                inputs: Some(vec![
                    ModelInput {
                        name: "location".to_string(),
                        value: Expr::Literal(Literal::String("north".to_string())),
                    },
                    ModelInput {
                        name: "height".to_string(),
                        value: Expr::Literal(Literal::Number(100.0)),
                    },
                ]),
                as_name: Some("sensor".to_string()),
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let sensor_id = Identifier::new("sensor");
        let sensor_path = ModelPath::new("/sensor");
        let sensor_submodel = create_test_model(vec![]);
        let model_map = HashMap::from([(&sensor_path, &sensor_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert!(errors.is_empty());

        // submodel tests
        assert_eq!(submodels.len(), 1);
        let submodel_path = submodels.get(&sensor_id);
        assert_eq!(submodel_path, Some(&sensor_path));

        // test inputs tests
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &sensor_id);
        assert_eq!(test_inputs.len(), 2);

        assert_eq!(test_inputs[0].name, "location");
        assert_eq!(
            test_inputs[0].value,
            Expr::Literal(Literal::String("north".to_string()))
        );

        assert_eq!(test_inputs[1].name, "height");
        assert_eq!(test_inputs[1].value, Expr::Literal(Literal::Number(100.0)));
    }

    #[test]
    fn test_resolve_nested_submodel() {
        let use_models = vec![UseModel {
            // use weather.atmosphere.temperature as temp
            model_name: "weather".to_string(),
            subcomponents: vec!["atmosphere".to_string(), "temperature".to_string()],
            inputs: None,
            as_name: Some("temp".to_string()),
        }];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_submodel = create_test_model(vec![]);
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_model =
            create_test_model(vec![("temperature", ModelPath::new("/temperature"))]);
        let weather_path = ModelPath::new("/weather");
        let weather_model = create_test_model(vec![("atmosphere", ModelPath::new("/atmosphere"))]);
        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
            (&temperature_path, &temperature_submodel),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert!(errors.is_empty());

        // submodel tests
        assert_eq!(submodels.len(), 1);

        let submodel_path = submodels.get(&temperature_id);
        assert_eq!(submodel_path, Some(&temperature_path));

        // test inputs tests
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &temperature_id);
        assert!(test_inputs.is_empty()); // no inputs
    }

    #[test]
    fn test_resolve_submodel_without_alias() {
        // build the use model list
        let use_models = vec![
            // use temperature
            UseModel {
                model_name: "temperature".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: None,
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temperature");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_submodel = create_test_model(vec![]);
        let model_map = HashMap::from([(&temperature_path, &temperature_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert!(errors.is_empty());

        // submodel tests
        assert_eq!(submodels.len(), 1);

        let submodel_path = submodels.get(&temperature_id);
        assert_eq!(submodel_path, Some(&temperature_path));

        // test inputs tests
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &temperature_id);
        assert!(test_inputs.is_empty()); // no inputs
    }

    #[test]
    fn test_resolve_submodel_with_subcomponent_alias() {
        // build the use model list
        let use_models = vec![
            // use weather.atmosphere
            UseModel {
                model_name: "weather".to_string(),
                subcomponents: vec!["atmosphere".to_string()],
                inputs: None,
                as_name: None, // Should use "atmosphere" as the alias
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let atmosphere_id = Identifier::new("atmosphere");
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_submodel = create_test_model(vec![]);
        let weather_path = ModelPath::new("/weather");
        let weather_submodel =
            create_test_model(vec![("atmosphere", ModelPath::new("/atmosphere"))]);

        // create the model map
        let model_map = HashMap::from([
            (&weather_path, &weather_submodel),
            (&atmosphere_path, &atmosphere_submodel),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert!(errors.is_empty());

        // submodel tests
        assert_eq!(submodels.len(), 1);

        let submodel_path = submodels.get(&atmosphere_id);
        assert_eq!(submodel_path, Some(&atmosphere_path));

        // test inputs tests
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &atmosphere_id);
        assert!(test_inputs.is_empty()); // no inputs
    }

    #[test]
    fn test_resolve_model_with_error() {
        // build the use model list
        let use_models = vec![
            // use error_model as error
            UseModel {
                model_name: "error_model".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: Some("error".to_string()),
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let error_id = Identifier::new("error");
        let error_path = ModelPath::new("/error_model");
        let error_submodel = create_test_model(vec![]);
        let model_map = HashMap::from([(&error_path, &error_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::from([&error_path]));

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert_eq!(errors.len(), 1);
        let error = errors.get(&error_id).unwrap();

        match error {
            SubmodelResolutionError::ModelHasError(path) => {
                assert_eq!(path, &error_path);
            }
            _ => panic!("Expected ModelHasError, got {:?}", error),
        }

        // submodel tests
        assert!(submodels.is_empty());

        // test inputs tests
        assert!(tests.is_empty());
    }

    #[test]
    fn test_resolve_undefined_submodel() {
        // build the use model list
        let use_models = vec![UseModel {
            model_name: "weather".to_string(),
            subcomponents: vec!["undefined_submodel".to_string()],
            inputs: None,
            as_name: Some("weather".to_string()),
        }];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let weather_id = Identifier::new("weather");
        let weather_path = ModelPath::new("/weather");
        let weather_model = create_test_model(vec![]); // No submodels
        let model_map = HashMap::from([(&weather_path, &weather_model)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert_eq!(errors.len(), 1);

        let error = errors.get(&weather_id).unwrap();
        match error {
            SubmodelResolutionError::UndefinedSubmodel(parent_path, submodel_name) => {
                assert_eq!(parent_path, &Some(weather_path));
                assert_eq!(submodel_name, &Identifier::new("undefined_submodel"));
            }
            _ => panic!("Expected UndefinedSubmodel, got {:?}", error),
        }

        // submodel tests
        assert!(submodels.is_empty());

        // test inputs tests
        assert!(tests.is_empty());
    }

    #[test]
    fn test_resolve_undefined_submodel_in_submodel() {
        // build the use model list
        let use_models = vec![UseModel {
            // use weather.atmosphere.undefined as undefined
            model_name: "weather".to_string(),
            subcomponents: vec!["atmosphere".to_string(), "undefined".to_string()],
            inputs: None,
            as_name: Some("undefined".to_string()),
        }];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let undefined_id = Identifier::new("undefined");
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_model = create_test_model(vec![]); // No submodels
        let weather_path = ModelPath::new("/weather");
        let weather_model = create_test_model(vec![("atmosphere", ModelPath::new("/atmosphere"))]);
        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert_eq!(errors.len(), 1);

        let error = errors.get(&undefined_id).unwrap();
        match error {
            SubmodelResolutionError::UndefinedSubmodel(parent_path, submodel_name) => {
                assert_eq!(parent_path, &Some(atmosphere_path));
                assert_eq!(submodel_name, &Identifier::new("undefined"));
            }
            _ => panic!("Expected UndefinedSubmodel, got {:?}", error),
        }

        // submodel tests
        assert!(submodels.is_empty());

        // test inputs tests
        assert!(tests.is_empty());
    }

    #[test]
    fn test_resolve_multiple_submodels() {
        // build the use model list
        let use_models = vec![
            // use temperature as temp
            UseModel {
                model_name: "temperature".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: Some("temp".to_string()),
            },
            // use pressure(altitude=1000) as press
            UseModel {
                model_name: "pressure".to_string(),
                subcomponents: vec![],
                inputs: Some(vec![ModelInput {
                    name: "altitude".to_string(),
                    value: Expr::Literal(Literal::Number(1000.0)),
                }]),
                as_name: Some("press".to_string()),
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = create_test_model(vec![]);
        let pressure_id = Identifier::new("press");
        let pressure_path = ModelPath::new("/pressure");
        let pressure_model = create_test_model(vec![]);
        let model_map = HashMap::from([
            (&temperature_path, &temperature_model),
            (&pressure_path, &pressure_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert!(errors.is_empty());

        // submodel tests
        assert_eq!(submodels.len(), 2);

        let temp_submodel_path = submodels.get(&temperature_id);
        assert_eq!(temp_submodel_path, Some(&temperature_path));

        let press_submodel_path = submodels.get(&pressure_id);
        assert_eq!(press_submodel_path, Some(&pressure_path));

        // test inputs tests
        assert_eq!(tests.len(), 2);

        // Check temperature test (no inputs)
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &temperature_id);
        assert!(test_inputs.is_empty());

        // Check pressure test (with altitude input)
        let (test_id, test_inputs) = &tests[1];
        assert_eq!(test_id, &pressure_id);
        assert_eq!(test_inputs.len(), 1);
        assert_eq!(test_inputs[0].name, "altitude");
        assert_eq!(test_inputs[0].value, Expr::Literal(Literal::Number(1000.0)));
    }

    #[test]
    fn test_resolve_mixed_success_and_error() {
        // build the use model list
        let use_models = vec![
            // use temperature as temp
            UseModel {
                model_name: "temperature".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: Some("temp".to_string()),
            },
            // use error_model as error
            UseModel {
                model_name: "error_model".to_string(),
                subcomponents: vec![],
                inputs: None,
                as_name: Some("error".to_string()),
            },
        ];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = create_test_model(vec![]);
        let error_id = Identifier::new("error");
        let error_path = ModelPath::new("/error_model");
        let error_model = create_test_model(vec![]);

        // create model info with one valid model and one error model
        let model_map = HashMap::from([
            (&temperature_path, &temperature_model),
            (&error_path, &error_model),
        ]);
        let mut model_with_errors = HashSet::new();
        model_with_errors.insert(&error_path);
        let model_info = ModelInfo::new(model_map, model_with_errors);

        // resolve the submodels and tests
        let (submodels, tests, errors) =
            resolve_submodels_and_tests(use_models, &model_path, &model_info);

        // error tests
        assert_eq!(errors.len(), 1);

        let error = errors.get(&error_id).unwrap();
        match error {
            SubmodelResolutionError::ModelHasError(path) => {
                assert_eq!(path, &error_path);
            }
            _ => panic!("Expected ModelHasError, got {:?}", error),
        }

        // submodel tests
        assert_eq!(submodels.len(), 1);

        let temp_submodel_path = submodels.get(&temperature_id);
        assert_eq!(temp_submodel_path, Some(&temperature_path));

        // test inputs tests
        assert_eq!(tests.len(), 1);

        // check temperature test (no inputs)
        let (test_id, test_inputs) = &tests[0];
        assert_eq!(test_id, &temperature_id);
        assert!(test_inputs.is_empty());
    }
}
