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
use oneil_ir::{
    reference::{Identifier, ModelPath},
    span::Span,
};

use crate::{
    error::SubmodelResolutionError,
    loader::resolver::ModelInfo,
    util::{get_span_from_ast_span, info::InfoResult},
};

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
pub fn resolve_submodels(
    use_models: Vec<&ast::declaration::UseModelNode>,
    model_path: &ModelPath,
    model_info: &ModelInfo<'_>,
) -> (
    HashMap<Identifier, (ModelPath, Span)>,
    HashMap<Identifier, SubmodelResolutionError>,
) {
    use_models.into_iter().fold(
        (HashMap::new(), HashMap::new()),
        |(mut submodels, mut resolution_errors), use_model| {
            // get the use model path
            let use_model_path = model_path.get_sibling_path(&use_model.model_name().as_str());
            let use_model_path = ModelPath::new(use_model_path);

            // get the submodel span
            let submodel_name_span = get_span_from_ast_span(use_model.node_span());

            // get the submodel name
            let submodel_name = use_model
                .alias()
                .or(use_model.subcomponents().last())
                .unwrap_or(use_model.model_name());
            let submodel_name = Identifier::new(submodel_name.as_str());

            // verify that the submodel name is not a duplicate
            let maybe_original_submodel = submodels.get(&submodel_name);
            if let Some((_path, original_submodel_span)) = maybe_original_submodel {
                resolution_errors.insert(
                    submodel_name.clone(),
                    SubmodelResolutionError::duplicate_submodel(
                        submodel_name,
                        original_submodel_span.clone(),
                        submodel_name_span,
                    ),
                );

                return (submodels, resolution_errors);
            }

            // resolve the use model path
            let resolved_use_model_path = resolve_model_path(
                use_model_path.clone(),
                submodel_name_span,
                use_model.subcomponents(),
                model_info,
            );

            // insert the use model path into the submodels map if it was resolved successfully
            // otherwise, add the error to the builder
            match resolved_use_model_path {
                Ok(resolved_use_model_path) => {
                    // create a span for the submodel
                    let span = get_span_from_ast_span(use_model.node_span());
                    submodels.insert(submodel_name.clone(), (resolved_use_model_path, span));
                }
                Err(error) => {
                    resolution_errors.insert(submodel_name, error);
                }
            }

            (submodels, resolution_errors)
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
    parent_component_span: Span,
    subcomponents: &[ast::naming::IdentifierNode],
    model_info: &ModelInfo<'_>,
) -> Result<ModelPath, SubmodelResolutionError> {
    // if the model that we are trying to resolve has had an error, this
    // operation should fail
    let model = match model_info.get(&model_path) {
        InfoResult::Found(model) => model,
        InfoResult::HasError => {
            return Err(SubmodelResolutionError::model_has_error(
                model_path,
                parent_component_span,
            ));
        }
        InfoResult::NotFound => panic!("model should have been visited already"),
    };

    // if there are no more subcomponents, we have resolved the model path
    if subcomponents.is_empty() {
        return Ok(model_path);
    }

    let submodel_name = Identifier::new(subcomponents[0].as_str());
    let submodel_name_span = get_span_from_ast_span(subcomponents[0].node_span());
    let submodel_path = model
        .get_submodel(&submodel_name)
        .map(|(path, _)| path)
        .ok_or(SubmodelResolutionError::undefined_submodel_in_submodel(
            model_path.clone(),
            submodel_name,
            submodel_name_span,
        ))?
        .clone();

    resolve_model_path(
        submodel_path,
        submodel_name_span,
        &subcomponents[1..],
        model_info,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oneil_ir::model::Model;
    use std::collections::HashSet;

    // TODO: write tests that test the span of the submodel path

    mod helper {
        use super::*;

        /// Helper function to create a test AST span
        pub fn test_ast_span(start: usize, end: usize) -> ast::Span {
            ast::Span::new(start, end - start, 0)
        }

        /// Helper function to create a test IR span
        pub fn test_ir_span(start: usize, end: usize) -> oneil_ir::span::Span {
            oneil_ir::span::Span::new(start, end - start)
        }

        /// Helper function to create a use model node
        pub fn create_use_model_node(
            model_name: &str,
            subcomponents: Vec<ast::naming::IdentifierNode>,
            alias: Option<&str>,
            start: usize,
            end: usize,
        ) -> ast::declaration::UseModelNode {
            let identifier = ast::naming::Identifier::new(model_name.to_string());
            let model_name_node =
                ast::node::Node::new(test_ast_span(start, start + model_name.len()), identifier);

            let alias_node = alias.map(|name| {
                let identifier = ast::naming::Identifier::new(name.to_string());
                ast::node::Node::new(test_ast_span(end - name.len(), end), identifier)
            });

            let use_model =
                ast::declaration::UseModel::new(model_name_node, subcomponents, alias_node);
            ast::node::Node::new(test_ast_span(start, end), use_model)
        }

        /// Helper function to create a test model with specified submodels
        pub fn create_test_model(submodels: Vec<(&str, (ModelPath, Span))>) -> Model {
            let mut submodel_map = HashMap::new();
            for (name, path) in submodels {
                let identifier = Identifier::new(name);
                submodel_map.insert(identifier, path);
            }

            Model::new(
                HashMap::new(),                                                // python_imports
                submodel_map,                                                  // submodels
                oneil_ir::parameter::ParameterCollection::new(HashMap::new()), // parameters
                HashMap::new(),                                                // tests
            )
        }
    }

    #[test]
    fn test_resolve_simple_submodel() {
        // create the use model list
        let use_model = helper::create_use_model_node("temperature", vec![], Some("temp"), 0, 20);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_submodel = helper::create_test_model(vec![]);
        let model_map = HashMap::from([(&temperature_path, &temperature_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temperature_id);
        let (submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(submodel_path, &temperature_path);
    }

    #[test]
    fn test_resolve_nested_submodel() {
        // create the use model list with nested subcomponents
        // use weather.atmosphere.temperature as temp
        let atmosphere_identifier = ast::naming::Identifier::new("atmosphere".to_string());
        let atmosphere_node =
            ast::node::Node::new(helper::test_ast_span(0, 10), atmosphere_identifier);
        let temperature_identifier = ast::naming::Identifier::new("temperature".to_string());
        let temperature_node =
            ast::node::Node::new(helper::test_ast_span(0, 11), temperature_identifier);
        let subcomponents = vec![atmosphere_node, temperature_node];

        let use_model =
            helper::create_use_model_node("weather", subcomponents, Some("temp"), 0, 35);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with nested structure
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_submodel = helper::create_test_model(vec![]);
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_model = helper::create_test_model(vec![(
            "temperature",
            (ModelPath::new("/temperature"), helper::test_ir_span(0, 11)),
        )]);
        let weather_path = ModelPath::new("/weather");
        let weather_model = helper::create_test_model(vec![(
            "atmosphere",
            (ModelPath::new("/atmosphere"), helper::test_ir_span(0, 11)),
        )]);
        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
            (&temperature_path, &temperature_submodel),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temperature_id);
        let (submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(submodel_path, &temperature_path);
    }

    #[test]
    fn test_resolve_submodel_without_alias() {
        // create the use model list without alias
        // use temperature
        let use_model = helper::create_use_model_node("temperature", vec![], None, 0, 12);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temperature");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_submodel = helper::create_test_model(vec![]);
        let model_map = HashMap::from([(&temperature_path, &temperature_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temperature_id);
        let (submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(submodel_path, &temperature_path);
    }

    #[test]
    fn test_resolve_submodel_with_subcomponent_alias() {
        // create the use model list with subcomponent as alias
        // use weather.atmosphere
        let atmosphere_identifier = ast::naming::Identifier::new("atmosphere".to_string());
        let atmosphere_node =
            ast::node::Node::new(helper::test_ast_span(0, 10), atmosphere_identifier);
        let subcomponents = vec![atmosphere_node];

        let use_model = helper::create_use_model_node("weather", subcomponents, None, 0, 20);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let atmosphere_id = Identifier::new("atmosphere");
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_submodel = helper::create_test_model(vec![]);
        let weather_path = ModelPath::new("/weather");
        let weather_submodel = helper::create_test_model(vec![(
            "atmosphere",
            (ModelPath::new("/atmosphere"), helper::test_ir_span(0, 11)),
        )]);

        // create the model map
        let model_map = HashMap::from([
            (&weather_path, &weather_submodel),
            (&atmosphere_path, &atmosphere_submodel),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&atmosphere_id);
        let (submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(submodel_path, &atmosphere_path);
    }

    #[test]
    fn test_resolve_model_with_error() {
        // create the use model list with error model
        // use error_model as error
        let use_model = helper::create_use_model_node("error_model", vec![], Some("error"), 0, 25);
        let use_model_span = get_span_from_ast_span(&use_model.node_span());
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with error model
        let error_id = Identifier::new("error");
        let error_path = ModelPath::new("/error_model");
        let error_submodel = helper::create_test_model(vec![]);
        let model_map = HashMap::from([(&error_path, &error_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::from([&error_path]));

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);
        let error = errors.get(&error_id).unwrap();

        match error {
            SubmodelResolutionError::ModelHasError {
                model_path,
                reference_span,
            } => {
                assert_eq!(model_path, &error_path);
                assert_eq!(reference_span, &use_model_span);
            }
            _ => panic!("Expected ModelHasError, got {:?}", error),
        }

        // check the submodels
        assert!(submodels.is_empty());
    }

    #[test]
    fn test_resolve_undefined_submodel() {
        // create the use model list with undefined submodel
        let undefined_identifier = ast::naming::Identifier::new("undefined_submodel".to_string());
        let undefined_identifier_span = helper::test_ast_span(0, 16);
        let undefined_node =
            ast::node::Node::new(undefined_identifier_span.clone(), undefined_identifier);
        let subcomponents = vec![undefined_node];

        let use_model =
            helper::create_use_model_node("weather", subcomponents, Some("weather"), 0, 30);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map without the undefined submodel
        let weather_id = Identifier::new("weather");
        let weather_path = ModelPath::new("/weather");
        let weather_model = helper::create_test_model(vec![]); // No submodels
        let model_map = HashMap::from([(&weather_path, &weather_model)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);

        let error = errors.get(&weather_id).unwrap();
        match error {
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span,
            } => {
                assert_eq!(parent_model_path, &weather_path);
                assert_eq!(submodel.as_str(), "undefined_submodel");
                assert_eq!(
                    reference_span,
                    &get_span_from_ast_span(&undefined_identifier_span)
                );
            }
            _ => panic!("Expected UndefinedSubmodel, got {:?}", error),
        }

        // check the submodels
        assert!(submodels.is_empty());
    }

    #[test]
    fn test_resolve_undefined_submodel_in_submodel() {
        // create the use model list with nested undefined submodel
        // use weather.atmosphere.undefined as undefined
        let atmosphere_identifier = ast::naming::Identifier::new("atmosphere".to_string());
        let atmosphere_node =
            ast::node::Node::new(helper::test_ast_span(0, 10), atmosphere_identifier);
        let undefined_identifier = ast::naming::Identifier::new("undefined".to_string());
        let undefined_identifier_span = helper::test_ast_span(0, 9);
        let undefined_node =
            ast::node::Node::new(undefined_identifier_span.clone(), undefined_identifier);
        let subcomponents = vec![atmosphere_node, undefined_node];

        let use_model =
            helper::create_use_model_node("weather", subcomponents, Some("undefined"), 0, 35);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with missing nested submodel
        let undefined_id = Identifier::new("undefined");
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_model = helper::create_test_model(vec![]); // No submodels
        let weather_path = ModelPath::new("/weather");
        let weather_model = helper::create_test_model(vec![(
            "atmosphere",
            (ModelPath::new("/atmosphere"), helper::test_ir_span(0, 11)),
        )]);
        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);

        let error = errors.get(&undefined_id).unwrap();
        match error {
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span,
            } => {
                assert_eq!(parent_model_path, &atmosphere_path);
                assert_eq!(submodel.as_str(), "undefined");
                assert_eq!(
                    reference_span,
                    &get_span_from_ast_span(&undefined_identifier_span)
                );
            }
            _ => panic!("Expected UndefinedSubmodel, got {:?}", error),
        }

        // check the submodels
        assert!(submodels.is_empty());
    }

    #[test]
    fn test_resolve_multiple_submodels() {
        // create the use model list with multiple submodels
        // use temperature as temp
        let temp_model = helper::create_use_model_node("temperature", vec![], Some("temp"), 0, 20);

        // use pressure as press
        let press_model = helper::create_use_model_node("pressure", vec![], Some("press"), 0, 25);

        let use_models = vec![&temp_model, &press_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);
        let pressure_id = Identifier::new("press");
        let pressure_path = ModelPath::new("/pressure");
        let pressure_model = helper::create_test_model(vec![]);
        let model_map = HashMap::from([
            (&temperature_path, &temperature_model),
            (&pressure_path, &pressure_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 2);

        let result = submodels.get(&temperature_id);
        let (temp_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let result = submodels.get(&pressure_id);
        let (press_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(press_submodel_path, &pressure_path);
    }

    #[test]
    fn test_resolve_mixed_success_and_error() {
        // create the use model list with mixed success and error cases
        // use temperature as temp
        let temp_model = helper::create_use_model_node("temperature", vec![], Some("temp"), 0, 20);

        // use error_model as error
        let error_model =
            helper::create_use_model_node("error_model", vec![], Some("error"), 0, 25);
        let error_model_ident_span = get_span_from_ast_span(&error_model.node_span());

        let use_models = vec![&temp_model, &error_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with one valid model and one error model
        let temperature_id = Identifier::new("temp");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);
        let error_id = Identifier::new("error");
        let error_path = ModelPath::new("/error_model");
        let error_model = helper::create_test_model(vec![]);

        // create model info with one valid model and one error model
        let model_map = HashMap::from([
            (&temperature_path, &temperature_model),
            (&error_path, &error_model),
        ]);
        let mut model_with_errors = HashSet::new();
        model_with_errors.insert(&error_path);
        let model_info = ModelInfo::new(model_map, model_with_errors);

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);

        let error = errors.get(&error_id).unwrap();
        match error {
            SubmodelResolutionError::ModelHasError {
                model_path,
                reference_span,
            } => {
                assert_eq!(model_path, &error_path);
                assert_eq!(reference_span, &error_model_ident_span);
            }
            _ => panic!("Expected ModelHasError, got {:?}", error),
        }

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temperature_id);
        let (temp_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);
    }
}
