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
    util::{
        context::{LookupResult, ModelContext},
        get_span_from_ast_span,
    },
};

type SubmodelMap = HashMap<Identifier, (ModelPath, Span)>;
type ResolutionErrorMap = HashMap<Identifier, SubmodelResolutionError>;

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
    context: &impl ModelContext,
) -> (SubmodelMap, ResolutionErrorMap) {
    use_models.into_iter().fold(
        (HashMap::new(), HashMap::new()),
        |(submodels, resolution_errors), use_model| {
            let submodel_path = get_use_model_path(model_path, use_model);
            let (submodel_name, submodel_name_span) =
                get_submodel_name_and_span(use_model.model_info());

            let (submodels, resolution_errors, result) = resolve_single_submodel(
                submodel_path,
                submodel_name_span,
                use_model.model_info().subcomponents(),
                submodel_name,
                submodel_name_span,
                context,
                submodels,
                resolution_errors,
            );

            // if the use model was resolved successfully and has submodels,
            // resolve each submodel
            let (Ok(resolved_model_path), Some(submodel_list)) = (result, use_model.submodels())
            else {
                // otherwise, return early
                return (submodels, resolution_errors);
            };

            submodel_list.iter().fold(
                (submodels, resolution_errors),
                |(submodels, resolution_errors), submodel| {
                    let resolved_model_name_span = submodel_name_span;

                    let mut submodel_subcomponents = submodel.subcomponents().to_vec();
                    submodel_subcomponents.insert(0, submodel.top_component().clone());

                    let (resolved_submodel_name, resolved_submodel_name_span) =
                        get_submodel_name_and_span(submodel);

                    let (submodels, resolution_errors, _result) = resolve_single_submodel(
                        resolved_model_path.clone(),
                        resolved_model_name_span,
                        &submodel_subcomponents,
                        resolved_submodel_name,
                        resolved_submodel_name_span,
                        context,
                        submodels,
                        resolution_errors,
                    );

                    (submodels, resolution_errors)
                },
            )
        },
    )
}

fn get_use_model_path(
    model_path: &ModelPath,
    use_model: &oneil_ast::node::Node<oneil_ast::declaration::UseModel>,
) -> ModelPath {
    let use_model_relative_path = use_model.get_model_relative_path();
    let use_model_path = model_path.get_sibling_path(&use_model_relative_path);

    ModelPath::new(use_model_path)
}

fn get_submodel_name_and_span(model_info: &ast::declaration::ModelInfo) -> (Identifier, Span) {
    let submodel_name = model_info
        .alias()
        .or_else(|| model_info.subcomponents().last())
        .unwrap_or(model_info.top_component());
    let ident = Identifier::new(submodel_name.as_str());
    let span = get_span_from_ast_span(submodel_name.node_span());
    (ident, span)
}

fn resolve_single_submodel(
    top_model_path: ModelPath,
    top_model_ident_span: Span,
    subcomponents: &[ast::naming::IdentifierNode],
    resolved_model_name: Identifier,
    resolved_model_reference_span: Span,
    context: &impl ModelContext,
    mut submodels: SubmodelMap,
    mut resolution_errors: ResolutionErrorMap,
) -> (SubmodelMap, ResolutionErrorMap, Result<ModelPath, ()>) {
    // verify that the submodel name is not a duplicate
    let maybe_original_submodel = submodels.get(&resolved_model_name);
    if let Some((_path, original_submodel_span)) = maybe_original_submodel {
        resolution_errors.insert(
            resolved_model_name.clone(),
            SubmodelResolutionError::duplicate_submodel(
                resolved_model_name,
                *original_submodel_span,
                resolved_model_reference_span,
            ),
        );

        return (submodels, resolution_errors, Err(()));
    }

    // resolve the use model path
    let resolved_use_model_path =
        resolve_model_path(top_model_path, top_model_ident_span, subcomponents, context);

    // insert the use model path into the submodels map if it was resolved successfully
    // otherwise, add the error to the builder
    let result = match resolved_use_model_path {
        Ok(resolved_use_model_path) => {
            submodels.insert(
                resolved_model_name,
                (
                    resolved_use_model_path.clone(),
                    resolved_model_reference_span,
                ),
            );

            Ok(resolved_use_model_path)
        }
        Err(error) => {
            resolution_errors.insert(resolved_model_name, error);
            Err(())
        }
    };

    (submodels, resolution_errors, result)
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
/// * **Invalid model**: If the model doesn't exist in `model_info`
///
/// # Safety
///
/// This function assumes that models referenced in `model_info` have been
/// properly loaded and validated. If this assumption is violated, the function
/// will panic, indicating a bug in the model loading process.
#[allow(
    clippy::panic_in_result_fn,
    reason = "panic enforces a function invariant"
)]
fn resolve_model_path(
    model_path: ModelPath,
    model_name_span: Span,
    model_subcomponents: &[ast::naming::IdentifierNode],
    context: &impl ModelContext,
) -> Result<ModelPath, SubmodelResolutionError> {
    // if the model that we are trying to resolve has had an error, this
    // operation should fail
    let model = match context.lookup_model(&model_path) {
        LookupResult::Found(model) => model,
        LookupResult::HasError => {
            return Err(SubmodelResolutionError::model_has_error(
                model_path,
                model_name_span,
            ));
        }
        LookupResult::NotFound => panic!("model should have been visited already"),
    };

    // if there are no more subcomponents, we have resolved the model path
    if model_subcomponents.is_empty() {
        return Ok(model_path);
    }

    let submodel_name = Identifier::new(model_subcomponents[0].as_str());
    let submodel_name_span = get_span_from_ast_span(model_subcomponents[0].node_span());
    let submodel_path = model
        .get_submodel(&submodel_name)
        .map(|(path, _)| path)
        .ok_or_else(|| {
            SubmodelResolutionError::undefined_submodel_in_submodel(
                model_path,
                submodel_name,
                submodel_name_span,
            )
        })?
        .clone();

    let submodel_subcomponents = &model_subcomponents[1..];

    resolve_model_path(
        submodel_path,
        submodel_name_span,
        submodel_subcomponents,
        context,
    )
}

#[cfg(test)]
#[cfg(any())]
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
                ast::node::Node::new(&test_ast_span(start, start + model_name.len()), identifier);

            let alias_node = alias.map(|name| {
                let identifier = ast::naming::Identifier::new(name.to_string());
                ast::node::Node::new(&test_ast_span(end - name.len(), end), identifier)
            });

            // Create the model info
            let model_info =
                ast::declaration::ModelInfo::new(model_name_node, subcomponents, alias_node);
            let model_info_node = ast::node::Node::new(&test_ast_span(start, end), model_info);

            // Create an empty directory path for tests
            let directory_path = vec![];

            let use_model = ast::declaration::UseModel::new(
                directory_path,
                model_info_node,
                None, // No submodel list
            );
            ast::node::Node::new(&test_ast_span(start, end), use_model)
        }

        /// Helper function to create a use model node with a specific directory path
        pub fn create_use_model_node_with_directory_path(
            model_name: &str,
            subcomponents: Vec<ast::naming::IdentifierNode>,
            alias: Option<&str>,
            directory_path: &[&str],
            start: usize,
            end: usize,
        ) -> ast::declaration::UseModelNode {
            let identifier = ast::naming::Identifier::new(model_name.to_string());
            let model_name_node =
                ast::node::Node::new(&test_ast_span(start, start + model_name.len()), identifier);

            let alias_node = alias.map(|name| {
                let identifier = ast::naming::Identifier::new(name.to_string());
                ast::node::Node::new(&test_ast_span(end - name.len(), end), identifier)
            });

            // Create the model info
            let model_info =
                ast::declaration::ModelInfo::new(model_name_node, subcomponents, alias_node);
            let model_info_node = ast::node::Node::new(&test_ast_span(start, end), model_info);

            // Convert string directory paths to DirectoryNode objects
            let directory_nodes: Vec<ast::naming::DirectoryNode> = directory_path
                .iter()
                .map(|dir_str| {
                    let directory = match *dir_str {
                        ".." => ast::naming::Directory::parent(),
                        "." => ast::naming::Directory::current(),
                        _ => ast::naming::Directory::name((*dir_str).to_string()),
                    };
                    ast::node::Node::new(&test_ast_span(start, start + dir_str.len()), directory)
                })
                .collect();

            let use_model = ast::declaration::UseModel::new(
                directory_nodes,
                model_info_node,
                None, // No submodel list
            );
            ast::node::Node::new(&test_ast_span(start, end), use_model)
        }

        /// Helper function to create a submodel node for the "with" clause
        pub fn create_submodel_node(
            name: &str,
            subcomponents: Vec<ast::naming::IdentifierNode>,
            alias: Option<&str>,
            start: usize,
            end: usize,
        ) -> ast::declaration::ModelInfoNode {
            let identifier = ast::naming::Identifier::new(name.to_string());
            let name_node =
                ast::node::Node::new(&test_ast_span(start, start + name.len()), identifier);

            let alias_node = alias.map(|alias_name| {
                let alias_identifier = ast::naming::Identifier::new(alias_name.to_string());
                ast::node::Node::new(
                    &test_ast_span(end - alias_name.len(), end),
                    alias_identifier,
                )
            });

            let submodel = ast::declaration::ModelInfo::new(name_node, subcomponents, alias_node);
            ast::node::Node::new(&test_ast_span(start, end), submodel)
        }

        /// Helper function to create a use model node with submodels in the "with" clause
        pub fn create_use_model_node_with_submodels(
            model_name: &str,
            subcomponents: Vec<ast::naming::IdentifierNode>,
            alias: Option<&str>,
            submodels: Vec<ast::declaration::ModelInfoNode>,
            start: usize,
            end: usize,
        ) -> ast::declaration::UseModelNode {
            let identifier = ast::naming::Identifier::new(model_name.to_string());
            let model_name_node =
                ast::node::Node::new(&test_ast_span(start, start + model_name.len()), identifier);

            let alias_node = alias.map(|name| {
                let identifier = ast::naming::Identifier::new(name.to_string());
                ast::node::Node::new(&test_ast_span(end - name.len(), end), identifier)
            });

            // Create the model info
            let model_info =
                ast::declaration::ModelInfo::new(model_name_node, subcomponents, alias_node);
            let model_info_node = ast::node::Node::new(&test_ast_span(start, end), model_info);

            // Create an empty directory path for tests
            let directory_path = vec![];

            // Create the submodel list
            let submodel_list = ast::declaration::SubmodelList::new(submodels);
            let submodel_list_node =
                ast::node::Node::new(&test_ast_span(start, end), submodel_list);

            let use_model = ast::declaration::UseModel::new(
                directory_path,
                model_info_node,
                Some(submodel_list_node),
            );
            ast::node::Node::new(&test_ast_span(start, end), use_model)
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
            ast::node::Node::new(&helper::test_ast_span(0, 10), atmosphere_identifier);
        let temperature_identifier = ast::naming::Identifier::new("temperature".to_string());
        let temperature_node =
            ast::node::Node::new(&helper::test_ast_span(0, 11), temperature_identifier);
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
            ast::node::Node::new(&helper::test_ast_span(0, 10), atmosphere_identifier);
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
        let use_model_alias = use_model.model_info().alias().expect("alias should exist");
        let use_model_name_span = get_span_from_ast_span(use_model_alias.node_span());
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
        let error = errors.get(&error_id).expect("error should exist");

        match error {
            SubmodelResolutionError::ModelHasError {
                model_path,
                reference_span,
            } => {
                assert_eq!(model_path, &error_path);
                assert_eq!(reference_span, &use_model_name_span);
            }
            _ => panic!("Expected ModelHasError, got {error:?}"),
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
            ast::node::Node::new(&undefined_identifier_span.clone(), undefined_identifier);
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

        let error = errors.get(&weather_id).expect("error should exist");
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
                    &get_span_from_ast_span(undefined_identifier_span)
                );
            }
            _ => panic!("Expected UndefinedSubmodel, got {error:?}"),
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
            ast::node::Node::new(&helper::test_ast_span(0, 10), atmosphere_identifier);
        let undefined_identifier = ast::naming::Identifier::new("undefined".to_string());
        let undefined_identifier_span = helper::test_ast_span(0, 9);
        let undefined_node =
            ast::node::Node::new(&undefined_identifier_span.clone(), undefined_identifier);
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

        let error = errors.get(&undefined_id).expect("error should exist");
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
                    &get_span_from_ast_span(undefined_identifier_span)
                );
            }
            _ => panic!("Expected UndefinedSubmodel, got {error:?}"),
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
        let error_model_alias = error_model
            .model_info()
            .alias()
            .expect("alias should exist");
        let error_model_name_span = get_span_from_ast_span(error_model_alias.node_span());

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

        let error = errors.get(&error_id).expect("error should exist");
        match error {
            SubmodelResolutionError::ModelHasError {
                model_path,
                reference_span,
            } => {
                assert_eq!(model_path, &error_path);
                assert_eq!(reference_span, &error_model_name_span);
            }
            _ => panic!("Expected ModelHasError, got {error:?}"),
        }

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temperature_id);
        let (temp_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);
    }

    #[test]
    fn test_resolve_submodel_with_directory_path_error() {
        // create the use model list with directory path that doesn't exist
        // use nonexistent/utils/math as math
        let use_model = helper::create_use_model_node_with_directory_path(
            "math",
            vec![],
            Some("math"),
            &["nonexistent", "utils"],
            0,
            30,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with the target model marked as having an error
        let math_id = Identifier::new("math");
        let math_path = ModelPath::new("/nonexistent/utils/math");
        let math_submodel = helper::create_test_model(vec![]);
        let model_map = HashMap::from([(&math_path, &math_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::from([&math_path]));

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors - should have a model loading error
        assert_eq!(errors.len(), 1);

        let error = errors.get(&math_id).expect("error should exist");
        match error {
            SubmodelResolutionError::ModelHasError { .. } => {
                // This is expected when the model has an error
            }
            _ => panic!("Expected ModelHasError, got {error:?}"),
        }

        // check the submodels
        assert!(submodels.is_empty());
    }

    #[test]
    fn test_resolve_submodel_with_directory_path_success() {
        // create the use model list with directory path that exists
        // use utils/math as math
        let use_model = helper::create_use_model_node_with_directory_path(
            "math",
            vec![],
            Some("math"),
            &["utils"],
            0,
            20,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with the target model
        let math_id = Identifier::new("math");
        let math_path = ModelPath::new("/utils/math");
        let math_submodel = helper::create_test_model(vec![]);
        let model_map = HashMap::from([(&math_path, &math_submodel)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&math_id);
        let (submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(submodel_path, &math_path);
    }

    #[test]
    fn test_resolve_duplicate_submodel_names() {
        // create the use model list with duplicate submodel names
        // use temperature as temp
        let temp_model1 = helper::create_use_model_node("temperature", vec![], Some("temp"), 0, 20);
        // use pressure as temp (duplicate alias)
        let temp_model2 = helper::create_use_model_node("pressure", vec![], Some("temp"), 0, 25);
        let temp_model2_alias = temp_model2
            .model_info()
            .alias()
            .expect("alias should exist");
        let temp_model2_name_span = get_span_from_ast_span(temp_model2_alias.node_span());

        let use_models = vec![&temp_model1, &temp_model2];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);
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
        assert_eq!(errors.len(), 1);

        let temp_id = Identifier::new("temp");
        let error = errors.get(&temp_id).expect("error should exist");
        match error {
            SubmodelResolutionError::DuplicateSubmodel {
                submodel,
                original_span: _,
                duplicate_span,
            } => {
                assert_eq!(submodel.as_str(), "temp");
                assert_eq!(duplicate_span, &temp_model2_name_span);
            }
            _ => panic!("Expected DuplicateSubmodel, got {error:?}"),
        }

        // check the submodels - should only contain the first one
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temp_id);
        let (submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(submodel_path, &temperature_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_failing_submodel_resolution() {
        // create the use model list where the original use declaration resolution fails
        // use nonexistent_model.subcomponent as sub
        let subcomponent_identifier = ast::naming::Identifier::new("subcomponent".to_string());
        let subcomponent_span = helper::test_ast_span(0, 12);
        let subcomponent_node =
            ast::node::Node::new(&subcomponent_span.clone(), subcomponent_identifier);
        let subcomponents = vec![subcomponent_node];

        let use_model =
            helper::create_use_model_node("nonexistent_model", subcomponents, Some("sub"), 0, 35);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with the target model marked as having an error
        let sub_id = Identifier::new("sub");
        let nonexistent_path = ModelPath::new("/nonexistent_model");
        let nonexistent_model = helper::create_test_model(vec![]);
        let model_map = HashMap::from([(&nonexistent_path, &nonexistent_model)]);
        let model_info = ModelInfo::new(model_map, HashSet::from([&nonexistent_path]));

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors - should have a model loading error
        assert_eq!(errors.len(), 1);

        let error = errors.get(&sub_id).expect("error should exist");
        match error {
            SubmodelResolutionError::ModelHasError { .. } => {
                // This is expected when the model has an error
            }
            _ => panic!("Expected ModelHasError, got {error:?}"),
        }

        // check the submodels
        assert!(submodels.is_empty());
    }

    #[test]
    fn test_resolve_use_declaration_with_multiple_submodels_complex() {
        // create the use model list with multiple submodels with different structures
        // use weather.atmosphere.temperature as temp
        let atmosphere_identifier = ast::naming::Identifier::new("atmosphere".to_string());
        let atmosphere_node =
            ast::node::Node::new(&helper::test_ast_span(0, 10), atmosphere_identifier);
        let temperature_identifier = ast::naming::Identifier::new("temperature".to_string());
        let temperature_node =
            ast::node::Node::new(&helper::test_ast_span(0, 11), temperature_identifier);
        let temp_subcomponents = vec![atmosphere_node, temperature_node];
        let temp_model =
            helper::create_use_model_node("weather", temp_subcomponents, Some("temp"), 0, 35);

        // use sensor.location as loc
        let location_identifier = ast::naming::Identifier::new("location".to_string());
        let location_node = ast::node::Node::new(&helper::test_ast_span(0, 8), location_identifier);
        let loc_subcomponents = vec![location_node];
        let loc_model =
            helper::create_use_model_node("sensor", loc_subcomponents, Some("loc"), 0, 25);

        // use pressure as press
        let press_model = helper::create_use_model_node("pressure", vec![], Some("press"), 0, 20);

        let use_models = vec![&temp_model, &loc_model, &press_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with complex nested structure
        let temp_id = Identifier::new("temp");
        let loc_id = Identifier::new("loc");
        let press_id = Identifier::new("press");

        let temperature_path = ModelPath::new("/temperature");
        let location_path = ModelPath::new("/location");
        let pressure_path = ModelPath::new("/pressure");

        let temperature_submodel = helper::create_test_model(vec![]);
        let location_submodel = helper::create_test_model(vec![]);
        let pressure_submodel = helper::create_test_model(vec![]);

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

        let sensor_path = ModelPath::new("/sensor");
        let sensor_model = helper::create_test_model(vec![(
            "location",
            (ModelPath::new("/location"), helper::test_ir_span(0, 8)),
        )]);

        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
            (&temperature_path, &temperature_submodel),
            (&sensor_path, &sensor_model),
            (&location_path, &location_submodel),
            (&pressure_path, &pressure_submodel),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 3);

        let result = submodels.get(&temp_id);
        let (temp_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let result = submodels.get(&loc_id);
        let (loc_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(loc_submodel_path, &location_path);

        let result = submodels.get(&press_id);
        let (press_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(press_submodel_path, &pressure_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_failing_submodel() {
        // create the use model list with a submodel that fails to resolve
        // use weather.atmosphere.temperature as temp
        let atmosphere_identifier = ast::naming::Identifier::new("atmosphere".to_string());
        let atmosphere_node =
            ast::node::Node::new(&helper::test_ast_span(0, 10), atmosphere_identifier);
        let temperature_identifier = ast::naming::Identifier::new("temperature".to_string());
        let temperature_span = helper::test_ast_span(0, 11);
        let temperature_node =
            ast::node::Node::new(&temperature_span.clone(), temperature_identifier);
        let subcomponents = vec![atmosphere_node, temperature_node];

        let use_model =
            helper::create_use_model_node("weather", subcomponents, Some("temp"), 0, 35);
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map where the nested submodel doesn't exist
        let temp_id = Identifier::new("temp");
        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_model = helper::create_test_model(vec![]); // No temperature submodel
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

        let error = errors.get(&temp_id).expect("error should exist");
        match error {
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span,
            } => {
                assert_eq!(parent_model_path, &atmosphere_path);
                assert_eq!(submodel.as_str(), "temperature");
                assert_eq!(reference_span, &get_span_from_ast_span(temperature_span));
            }
            _ => panic!("Expected UndefinedSubmodel, got {error:?}"),
        }

        // check the submodels
        assert!(submodels.is_empty());
    }

    #[test]
    fn test_resolve_use_declaration_with_successful_and_failing_submodels() {
        // create the use model list with both successful and failing submodels
        // use temperature as temp (successful)
        let temp_model = helper::create_use_model_node("temperature", vec![], Some("temp"), 0, 20);

        // use weather.atmosphere.undefined as undefined (failing)
        let atmosphere_identifier = ast::naming::Identifier::new("atmosphere".to_string());
        let atmosphere_node =
            ast::node::Node::new(&helper::test_ast_span(0, 10), atmosphere_identifier);
        let undefined_identifier = ast::naming::Identifier::new("undefined".to_string());
        let undefined_span = helper::test_ast_span(0, 9);
        let undefined_node = ast::node::Node::new(&undefined_span.clone(), undefined_identifier);
        let undefined_subcomponents = vec![atmosphere_node, undefined_node];
        let undefined_model = helper::create_use_model_node(
            "weather",
            undefined_subcomponents,
            Some("undefined"),
            0,
            35,
        );

        let use_models = vec![&temp_model, &undefined_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with mixed success and failure scenarios
        let temp_id = Identifier::new("temp");
        let undefined_id = Identifier::new("undefined");

        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);

        let atmosphere_path = ModelPath::new("/atmosphere");
        let atmosphere_model = helper::create_test_model(vec![]); // No undefined submodel

        let weather_path = ModelPath::new("/weather");
        let weather_model = helper::create_test_model(vec![(
            "atmosphere",
            (ModelPath::new("/atmosphere"), helper::test_ir_span(0, 11)),
        )]);

        let model_map = HashMap::from([
            (&temperature_path, &temperature_model),
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);

        let error = errors.get(&undefined_id).expect("error should exist");
        match error {
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span,
            } => {
                assert_eq!(parent_model_path, &atmosphere_path);
                assert_eq!(submodel.as_str(), "undefined");
                assert_eq!(reference_span, &get_span_from_ast_span(undefined_span));
            }
            _ => panic!("Expected UndefinedSubmodel, got {error:?}"),
        }

        // check the submodels - should only contain the successful one
        assert_eq!(submodels.len(), 1);

        let result = submodels.get(&temp_id);
        let (temp_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);
    }

    // Tests for the "with" clause functionality
    // These tests assume that the resolve_submodels function will be updated to handle the with clause

    #[test]
    fn test_resolve_use_declaration_with_single_submodel() {
        // create the use model list with a single submodel in the with clause
        // use weather with temperature as temp
        let temperature_submodel =
            helper::create_submodel_node("temperature", vec![], Some("temp"), 0, 20);
        let use_model = helper::create_use_model_node_with_submodels(
            "weather",
            vec![],
            None,
            vec![temperature_submodel],
            0,
            25,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temp_id = Identifier::new("temp");
        let weather_path = ModelPath::new("/weather");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);
        let weather_model = helper::create_test_model(vec![(
            "temperature",
            (ModelPath::new("/temperature"), helper::test_ir_span(0, 11)),
        )]);

        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&temperature_path, &temperature_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 2);

        let (temp_submodel_path, _span) = submodels
            .get(&temp_id)
            .expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let weather_id = Identifier::new("weather");
        let (weather_submodel_path, _span) = submodels
            .get(&weather_id)
            .expect("submodel path should be present");
        assert_eq!(weather_submodel_path, &weather_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_multiple_submodels() {
        // create the use model list with multiple submodels in the with clause
        // use weather with [temperature as temp, pressure as press]
        let temperature_submodel =
            helper::create_submodel_node("temperature", vec![], Some("temp"), 0, 20);
        let pressure_submodel =
            helper::create_submodel_node("pressure", vec![], Some("press"), 0, 20);
        let use_model = helper::create_use_model_node_with_submodels(
            "weather",
            vec![],
            None,
            vec![temperature_submodel, pressure_submodel],
            0,
            45,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temp_id = Identifier::new("temp");
        let press_id = Identifier::new("press");
        let weather_path = ModelPath::new("/weather");
        let temperature_path = ModelPath::new("/temperature");
        let pressure_path = ModelPath::new("/pressure");
        let temperature_model = helper::create_test_model(vec![]);
        let pressure_model = helper::create_test_model(vec![]);
        let weather_model = helper::create_test_model(vec![
            (
                "temperature",
                (ModelPath::new("/temperature"), helper::test_ir_span(0, 11)),
            ),
            (
                "pressure",
                (ModelPath::new("/pressure"), helper::test_ir_span(0, 8)),
            ),
        ]);

        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&temperature_path, &temperature_model),
            (&pressure_path, &pressure_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 3);

        let (temp_submodel_path, _span) = submodels
            .get(&temp_id)
            .expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let (press_submodel_path, _span) = submodels
            .get(&press_id)
            .expect("submodel path should be present");
        assert_eq!(press_submodel_path, &pressure_path);

        let weather_id = Identifier::new("weather");
        let (weather_submodel_path, _span) = submodels
            .get(&weather_id)
            .expect("submodel path should be present");
        assert_eq!(weather_submodel_path, &weather_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_nested_submodel() {
        // create the use model list with a nested submodel in the with clause
        // use weather with atmosphere.temperature as temp
        let temperature_identifier = ast::naming::Identifier::new("temperature".to_string());
        let temperature_node =
            ast::node::Node::new(&helper::test_ast_span(0, 11), temperature_identifier);
        let temperature_submodel =
            helper::create_submodel_node("atmosphere", vec![temperature_node], Some("temp"), 0, 25);
        let use_model = helper::create_use_model_node_with_submodels(
            "weather",
            vec![],
            None,
            vec![temperature_submodel],
            0,
            30,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let temp_id = Identifier::new("temp");
        let weather_path = ModelPath::new("/weather");
        let atmosphere_path = ModelPath::new("/atmosphere");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);
        let atmosphere_model = helper::create_test_model(vec![(
            "temperature",
            (ModelPath::new("/temperature"), helper::test_ir_span(0, 11)),
        )]);
        let weather_model = helper::create_test_model(vec![(
            "atmosphere",
            (ModelPath::new("/atmosphere"), helper::test_ir_span(0, 10)),
        )]);

        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&atmosphere_path, &atmosphere_model),
            (&temperature_path, &temperature_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels
        assert_eq!(submodels.len(), 2);

        let (temp_submodel_path, _span) = submodels
            .get(&temp_id)
            .expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let weather_id = Identifier::new("weather");
        let (weather_submodel_path, _span) = submodels
            .get(&weather_id)
            .expect("submodel path should be present");
        assert_eq!(weather_submodel_path, &weather_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_failing_submodel_in_with_clause() {
        // create the use model list with a failing submodel in the with clause
        // use weather with undefined as undefined
        let undefined_submodel =
            helper::create_submodel_node("undefined", vec![], Some("undefined"), 0, 20);
        let use_model = helper::create_use_model_node_with_submodels(
            "weather",
            vec![],
            None,
            vec![undefined_submodel],
            0,
            30,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map where the submodel doesn't exist
        let undefined_id = Identifier::new("undefined");
        let weather_path = ModelPath::new("/weather");
        let weather_model = helper::create_test_model(vec![]); // No undefined submodel

        let model_map = HashMap::from([(&weather_path, &weather_model)]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);

        let error = errors.get(&undefined_id).expect("error should exist");
        match error {
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span: _,
            } => {
                assert_eq!(parent_model_path, &weather_path);
                assert_eq!(submodel.as_str(), "undefined");
            }
            _ => panic!("Expected UndefinedSubmodel, got {error:?}"),
        }

        // check the submodels
        assert_eq!(submodels.len(), 1);

        let weather_id = Identifier::new("weather");
        let (submodel_path, _span) = submodels
            .get(&weather_id)
            .expect("submodel path should be present");
        assert_eq!(submodel_path, &weather_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_successful_and_failing_submodels_in_with_clause() {
        // create the use model list with both successful and failing submodels in the with clause
        // use weather with [temperature as temp, undefined as undefined]
        let temperature_submodel =
            helper::create_submodel_node("temperature", vec![], Some("temp"), 0, 20);
        let undefined_submodel =
            helper::create_submodel_node("undefined", vec![], Some("undefined"), 0, 20);
        let use_model = helper::create_use_model_node_with_submodels(
            "weather",
            vec![],
            None,
            vec![temperature_submodel, undefined_submodel],
            0,
            50,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map with mixed success and failure scenarios
        let temp_id = Identifier::new("temp");
        let undefined_id = Identifier::new("undefined");
        let weather_path = ModelPath::new("/weather");
        let temperature_path = ModelPath::new("/temperature");
        let temperature_model = helper::create_test_model(vec![]);
        let weather_model = helper::create_test_model(vec![(
            "temperature",
            (ModelPath::new("/temperature"), helper::test_ir_span(0, 11)),
        )]); // No undefined submodel

        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&temperature_path, &temperature_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert_eq!(errors.len(), 1);

        let error = errors.get(&undefined_id).expect("error should exist");
        match error {
            SubmodelResolutionError::UndefinedSubmodel {
                parent_model_path,
                submodel,
                reference_span: _,
            } => {
                assert_eq!(parent_model_path, &weather_path);
                assert_eq!(submodel.as_str(), "undefined");
            }
            _ => panic!("Expected UndefinedSubmodel, got {error:?}"),
        }

        // check the submodels - should only contain the successful one
        assert_eq!(submodels.len(), 2);

        let (temp_submodel_path, _span) = submodels
            .get(&temp_id)
            .expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let weather_id = Identifier::new("weather");
        let (weather_submodel_path, _span) = submodels
            .get(&weather_id)
            .expect("submodel path should be present");
        assert_eq!(weather_submodel_path, &weather_path);
    }

    #[test]
    fn test_resolve_use_declaration_with_model_alias_and_submodels() {
        // create the use model list with model alias and submodels in the with clause
        // use weather as weather_model with [temperature as temp, pressure as press]
        let temperature_submodel =
            helper::create_submodel_node("temperature", vec![], Some("temp"), 0, 20);
        let pressure_submodel =
            helper::create_submodel_node("pressure", vec![], Some("press"), 0, 20);
        let use_model = helper::create_use_model_node_with_submodels(
            "weather",
            vec![],
            Some("weather_model"),
            vec![temperature_submodel, pressure_submodel],
            0,
            55,
        );
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ModelPath::new("/parent_model");

        // create the model map
        let weather_model_id = Identifier::new("weather_model");
        let temp_id = Identifier::new("temp");
        let press_id = Identifier::new("press");
        let weather_path = ModelPath::new("/weather");
        let temperature_path = ModelPath::new("/temperature");
        let pressure_path = ModelPath::new("/pressure");
        let temperature_model = helper::create_test_model(vec![]);
        let pressure_model = helper::create_test_model(vec![]);
        let weather_model = helper::create_test_model(vec![
            (
                "temperature",
                (ModelPath::new("/temperature"), helper::test_ir_span(0, 11)),
            ),
            (
                "pressure",
                (ModelPath::new("/pressure"), helper::test_ir_span(0, 8)),
            ),
        ]);

        let model_map = HashMap::from([
            (&weather_path, &weather_model),
            (&temperature_path, &temperature_model),
            (&pressure_path, &pressure_model),
        ]);
        let model_info = ModelInfo::new(model_map, HashSet::new());

        // resolve the submodels
        let (submodels, errors) = resolve_submodels(use_models, &model_path, &model_info);

        // check the errors
        assert!(errors.is_empty());

        // check the submodels - should include both the model alias and the with clause submodels
        assert_eq!(submodels.len(), 3);

        let result = submodels.get(&weather_model_id);
        let (weather_model_path, _span) = result.expect("weather model path should be present");
        assert_eq!(weather_model_path, &weather_path);

        let result = submodels.get(&temp_id);
        let (temp_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(temp_submodel_path, &temperature_path);

        let result = submodels.get(&press_id);
        let (press_submodel_path, _span) = result.expect("submodel path should be present");
        assert_eq!(press_submodel_path, &pressure_path);
    }
}
