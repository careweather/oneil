//! Submodel resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    error::ModelImportResolutionError,
    util::{
        ReferenceMap, ReferenceResolutionErrors, SubmodelMap, SubmodelResolutionErrors,
        builder::ModelImportsBuilder,
        context::{ModelContext, ModelContextResult},
    },
};

/// Resolves submodels and their associated tests from use model declarations.
pub fn resolve_model_imports(
    use_models: Vec<&ast::UseModelNode>,
    model_path: &ir::ModelPath,
    context: &ModelContext<'_>,
) -> (
    SubmodelMap,
    ReferenceMap,
    SubmodelResolutionErrors,
    ReferenceResolutionErrors,
) {
    let mut builder = ModelImportsBuilder::new();

    for use_model in use_models {
        let import_path = calc_import_path(model_path, use_model);

        let (reference_name, reference_name_span) =
            get_reference_name_and_span(use_model.model_info());
        let (submodel_name, submodel_name_span) =
            get_submodel_name_and_span(use_model.model_info());

        let is_submodel = use_model.model_kind() == ast::ModelKind::Submodel;

        // check for duplicates
        let maybe_reference_duplicate_error =
            builder
                .get_reference(&reference_name)
                .map(|original_reference| {
                    ModelImportResolutionError::duplicate_reference(
                        reference_name.clone(),
                        *original_reference.name_span(),
                        reference_name_span,
                    )
                });

        let maybe_submodel_duplicate_error =
            builder
                .get_submodel(&submodel_name)
                .map(|original_submodel| {
                    ModelImportResolutionError::duplicate_submodel(
                        submodel_name.clone(),
                        *original_submodel.name_span(),
                        submodel_name_span,
                    )
                });

        let had_duplicate = maybe_reference_duplicate_error.is_some()
            || (is_submodel && maybe_submodel_duplicate_error.is_some());

        // handle duplicate references
        if let Some(reference_duplicate_error) = maybe_reference_duplicate_error {
            builder
                .add_reference_resolution_error(reference_name.clone(), reference_duplicate_error);
        }

        // handle duplicate submodels if the use model is a submodel
        if is_submodel && let Some(submodel_duplicate_error) = maybe_submodel_duplicate_error {
            builder.add_submodel_resolution_error(submodel_name.clone(), submodel_duplicate_error);
        }

        // if there were any duplicates, stop processing this use model
        if had_duplicate {
            continue;
        }

        // resolve the path for the use model
        let subcomponents = use_model.model_info().subcomponents();
        let model_name_span = submodel_name_span;
        let resolved_path =
            resolve_model_path(import_path, model_name_span, subcomponents, context);

        // handle the error if there was one
        let resolved_path = match resolved_path {
            Ok(resolved_path) => resolved_path,
            Err(error) if is_submodel => {
                builder.add_submodel_resolution_error(submodel_name, error.clone());

                // It's currently necessary to add it to the reference
                // resolution errors because otherwise, variable resolution will
                // assume that the reference is not defined in the current model,
                // not that the reference resolution failed
                // TODO: figure out how to remove this duplication - maybe
                // submodels point to references, since a reference will always
                // exist for every submodel?
                builder.add_reference_resolution_error(reference_name, error);

                continue;
            }
            Err(error) => {
                builder.add_reference_resolution_error(reference_name, error);
                continue;
            }
        };

        // add the reference to the builder
        builder.add_reference(reference_name, reference_name_span, resolved_path.clone());

        // add the submodel to the builder if it's a submodel
        if is_submodel {
            builder.add_submodel(submodel_name, submodel_name_span, resolved_path.clone());
        }

        let Some(submodel_list) = use_model.submodels() else {
            // if we don't have any submodels, we're done
            continue;
        };

        for submodel_info in submodel_list.iter() {
            // get the subcomponents relative to the main model being imported
            let mut submodel_subcomponents = submodel_info.subcomponents().to_vec();
            submodel_subcomponents.insert(0, submodel_info.top_component().clone());

            // get the reference name for the submodel
            let (reference_name, reference_name_span) = get_reference_name_and_span(submodel_info);

            // check for duplicate references
            let maybe_original_reference = builder.get_reference(&reference_name);
            if let Some(original_reference) = maybe_original_reference {
                // if there is a duplicate, add the error and continue
                let error = ModelImportResolutionError::duplicate_reference(
                    reference_name.clone(),
                    *original_reference.name_span(),
                    reference_name_span,
                );

                builder.add_reference_resolution_error(reference_name.clone(), error);

                continue;
            }

            // resolve the reference path
            let resolved_reference_path = resolve_model_path(
                resolved_path.clone(),
                reference_name_span,
                &submodel_subcomponents,
                context,
            );

            match resolved_reference_path {
                Ok(resolved_reference_path) => {
                    builder.add_reference(
                        reference_name,
                        reference_name_span,
                        resolved_reference_path,
                    );
                }
                Err(error) => {
                    builder.add_reference_resolution_error(reference_name, error);
                }
            }
        }
    }

    builder.into_submodels_and_references_and_resolution_errors()
}

fn get_submodel_name_and_span(model_info: &ast::ModelInfo) -> (ir::SubmodelName, Span) {
    let model_name = model_info.get_model_name();
    let name = ir::SubmodelName::new(model_name.as_str().to_string());
    let span = model_name.span();
    (name, span)
}

fn get_reference_name_and_span(model_info: &ast::ModelInfo) -> (ir::ReferenceName, Span) {
    let model_name = model_info.get_alias();
    let name = ir::ReferenceName::new(model_name.as_str().to_string());
    let span = model_name.span();
    (name, span)
}

fn calc_import_path(model_path: &ir::ModelPath, use_model: &ast::UseModelNode) -> ir::ModelPath {
    let use_model_relative_path = use_model.get_model_relative_path();
    let use_model_path = model_path.get_sibling_path(&use_model_relative_path);

    ir::ModelPath::new(use_model_path)
}

/// Recursively resolves a model path by traversing subcomponents.
///
/// This internal function handles the recursive resolution of model paths
/// when dealing with nested submodels (e.g., `parent.submodel1.submodel2`).
/// It traverses the subcomponent chain and validates that each level exists.
///
/// # Examples
///
/// For a path like `weather.atmosphere.temperature`:
/// 1. First call: `resolve_model_path(None, "weather", ["atmosphere", "temperature"], ...)`
/// 2. Second call: `resolve_model_path(Some("weather"), "atmosphere", ["temperature"], ...)`
/// 3. Third call: `resolve_model_path(Some("atmosphere"), "temperature", [], ...)`
/// 4. Returns: `Ok("temperature")`
///
/// # Panics
///
/// This function assumes that models referenced in `model_info` have been
/// properly loaded and validated. If this assumption is violated, the function
/// will panic, indicating a bug in the model loading process.
fn resolve_model_path(
    model_path: ir::ModelPath,
    model_name_span: Span,
    model_subcomponents: &[ast::IdentifierNode],
    context: &ModelContext<'_>,
) -> Result<ir::ModelPath, ModelImportResolutionError> {
    // if the model that we are trying to resolve has had an error, this
    // operation should fail
    let model = match context.lookup_model(&model_path) {
        ModelContextResult::Found(model) => model,
        ModelContextResult::HasError => {
            return Err(ModelImportResolutionError::model_has_error(
                model_path,
                model_name_span,
            ));
        }
        ModelContextResult::NotFound => unreachable!("model should have been visited already"),
    };

    // if there are no more subcomponents, we have resolved the model path
    if model_subcomponents.is_empty() {
        return Ok(model_path);
    }

    let submodel_name = ir::SubmodelName::new(model_subcomponents[0].as_str().to_string());
    let submodel_name_span = model_subcomponents[0].span();
    let submodel_path = model
        .get_submodel(&submodel_name)
        .map(ir::SubmodelImport::path)
        .ok_or_else(|| {
            ModelImportResolutionError::undefined_submodel_in_submodel(
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
mod tests {
    use std::collections::HashMap;

    use crate::test::construct::{ModelContextBuilder, test_ast, test_ir};

    use super::*;
    use oneil_ast as ast;
    use oneil_ir as ir;

    // This is a macro, as opposed to a function, because we want the error
    // location to show the line in the test where the assertion failed, rather
    // than some line in an `assert_has_submodels` function
    macro_rules! assert_has_submodels {
        ($actual_submodel_map:expr, $expected_submodels:expr $(,)?) => {
            let actual_submodel_map: &HashMap<ir::SubmodelName, ir::SubmodelImport> =
                $actual_submodel_map;
            let expected_submodels: Vec<(&'static str, &ir::ModelPath)> =
                $expected_submodels.into_iter().collect();

            // check that the submodel map length is the same as the number of submodels
            assert_eq!(
                actual_submodel_map.len(),
                expected_submodels.len(),
                "length of *actual* submodel map differs from *expected* submodel map",
            );

            // check that the submodel map contains the expected submodels
            for (submodel_name, expected_submodel_path) in expected_submodels {
                let submodel_name = ir::SubmodelName::new(submodel_name.to_string());
                let submodel_import = actual_submodel_map.get(&submodel_name).expect(
                    format!(
                        "did not find submodel path for '{}'",
                        submodel_name.as_str()
                    )
                    .as_str(),
                );

                assert_eq!(
                    submodel_import.path(),
                    expected_submodel_path,
                    "actual submodel path for '{}' differs from expected submodel path",
                    submodel_name.as_str(),
                );
            }
        };
    }

    // This is a macro, as opposed to a function, because we want the error
    // location to show the line in the test where the assertion failed, rather
    // than some line in an `assert_has_references` function
    macro_rules! assert_has_references {
        ($reference_map:expr, $references:expr $(,)?) => {
            let reference_map: &HashMap<ir::ReferenceName, ir::ReferenceImport> = $reference_map;
            let references: Vec<(&'static str, &ir::ModelPath)> = $references.into_iter().collect();

            // check that the reference map length is the same as the number of references
            assert_eq!(
                reference_map.len(),
                references.len(),
                "length of *actual* reference map differs from *expected* reference map",
            );

            // check that the reference map contains the expected references
            for (reference_name, reference_path) in references {
                let reference_name = ir::ReferenceName::new(reference_name.to_string());
                let reference_import = reference_map.get(&reference_name).expect(
                    format!(
                        "did not find reference path for '{}'",
                        reference_name.as_str()
                    )
                    .as_str(),
                );

                assert_eq!(
                    reference_import.path(),
                    reference_path,
                    "actual reference path for '{}' differs from expected reference path",
                    reference_name.as_str(),
                );
            }
        };
    }

    #[test]
    fn resolve_simple_submodel() {
        // create the model import list
        // > use temperature as temp
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(temperature_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("temperature", &temperature_path)]);

        // check the references
        assert_has_references!(&reference_map, [("temp", &temperature_path)]);
    }

    #[test]
    fn resolve_nested_submodel() {
        // create the use model list with nested subcomponents
        // > use weather.atmosphere.temperature as temp
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "temperature"])
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let atmosphere_path = ir::ModelPath::new("/atmosphere");
        let atmosphere_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", "/temperature")
            .build();

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", "/atmosphere")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (atmosphere_path, atmosphere_model),
            (weather_path, weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("temperature", &temperature_path)]);

        // check the references
        assert_has_references!(&reference_map, [("temp", &temperature_path)]);
    }

    #[test]
    fn resolve_submodel_without_alias() {
        // create the use model list without alias
        // > use temperature
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(temperature_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("temperature", &temperature_path)]);

        // check the references
        assert_has_references!(&reference_map, [("temperature", &temperature_path)]);
    }

    #[test]
    fn resolve_submodel_with_subcomponent_alias() {
        // create the use model list with subcomponent as alias
        // > use weather.atmosphere
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let atmosphere_path = ir::ModelPath::new("/atmosphere");

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", "/atmosphere")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (atmosphere_path.clone(), test_ir::empty_model()),
            (weather_path, weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the errors
        assert!(submodel_errors.is_empty());
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("atmosphere", &atmosphere_path)]);

        // check the references
        assert_has_references!(&reference_map, [("atmosphere", &atmosphere_path)]);
    }

    #[test]
    fn resolve_model_with_error() {
        // create the use model list with error model
        // > use error_model as error
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("error_model")
            .with_alias("error")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let error_path = ir::ModelPath::new("/error_model");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(error_path.clone(), test_ir::empty_model())])
            .with_model_error_context([error_path.clone()]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 1);
        let error = submodel_errors
            .get(&ir::SubmodelName::new("error_model".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };

        assert_eq!(model_path, &error_path);

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);
        let error = reference_errors
            .get(&ir::ReferenceName::new("error".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };

        assert_eq!(model_path, &error_path);

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, []);
    }

    #[test]
    fn resolve_undefined_submodel() {
        // create the use model list with undefined submodel
        // > use weather.undefined_submodel
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["undefined_submodel"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let weather_path = ir::ModelPath::new("/weather");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(weather_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 1);

        let error = submodel_errors
            .get(&ir::SubmodelName::new("undefined_submodel".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &weather_path);
        assert_eq!(submodel.as_str(), "undefined_submodel");

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);
        let error = reference_errors
            .get(&ir::ReferenceName::new("undefined_submodel".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &weather_path);
        assert_eq!(submodel.as_str(), "undefined_submodel");

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, []);
    }

    #[test]
    fn resolve_undefined_submodel_in_submodel() {
        // create the use model list with nested undefined submodel
        // > use weather.atmosphere.undefined
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "undefined"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&model_import];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let atmosphere_path = ir::ModelPath::new("/atmosphere");

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", "/atmosphere")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (atmosphere_path.clone(), test_ir::empty_model()),
            (weather_path, weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the errors
        assert_eq!(submodel_errors.len(), 1);

        let error = submodel_errors
            .get(&ir::SubmodelName::new("undefined".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &atmosphere_path);
        assert_eq!(submodel.as_str(), "undefined");

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let error = reference_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &atmosphere_path);
        assert_eq!(submodel.as_str(), "undefined");

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, []);
    }

    #[test]
    fn resolve_multiple_submodels() {
        // create the use model list with multiple submodels
        // > use temperature as temp
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        // > use pressure as press
        let press_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("pressure")
            .with_alias("press")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&temp_model, &press_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let pressure_path = ir::ModelPath::new("/pressure");

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (pressure_path.clone(), test_ir::empty_model()),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(
            &submodel_map,
            [
                ("temperature", &temperature_path),
                ("pressure", &pressure_path),
            ],
        );

        // check the references
        assert_has_references!(
            &reference_map,
            [("temp", &temperature_path), ("press", &pressure_path)],
        );
    }

    #[test]
    fn resolve_mixed_success_and_error() {
        // create the use model list with mixed success and error cases
        // > use temperature as temp
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        // > use error_model as error
        let error_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("error_model")
            .with_alias("error")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&temp_model, &error_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let error_path = ir::ModelPath::new("/error_model");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([
                (temperature_path.clone(), test_ir::empty_model()),
                (error_path.clone(), test_ir::empty_model()),
            ])
            .with_model_error_context([error_path.clone()]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 1);

        let error = submodel_errors
            .get(&ir::SubmodelName::new("error_model".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };

        assert_eq!(model_path, &error_path);

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let error = reference_errors
            .get(&ir::ReferenceName::new("error".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };

        assert_eq!(model_path, &error_path);

        // check the submodels
        assert_has_submodels!(&submodel_map, [("temperature", &temperature_path)]);

        // check the references
        assert_has_references!(&reference_map, [("temp", &temperature_path)]);
    }

    #[test]
    fn resolve_submodel_with_directory_path_success() {
        // create the use model list with directory path that exists
        // > use utils/math as math
        let math_model = test_ast::ImportModelNodeBuilder::new()
            .with_directory_path(["utils"])
            .with_top_component("math")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&math_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let math_path = ir::ModelPath::new("/utils/math");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(math_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("math", &math_path)]);

        // check the references
        assert_has_references!(&reference_map, [("math", &math_path)]);
    }

    #[test]
    fn resolve_submodel_with_directory_path_error() {
        // create the use model list with directory path that doesn't exist
        // > use nonexistent/math as math
        let math_model = test_ast::ImportModelNodeBuilder::new()
            .with_directory_path(["nonexistent"])
            .with_top_component("math")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports = vec![&math_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let math_path = ir::ModelPath::new("/nonexistent/math");
        let context_builder =
            ModelContextBuilder::new().with_model_error_context([math_path.clone()]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 1);

        let error = submodel_errors
            .get(&ir::SubmodelName::new("math".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };

        assert_eq!(model_path, &math_path);

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let error = reference_errors
            .get(&ir::ReferenceName::new("math".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };

        assert_eq!(model_path, &math_path);

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, []);
    }

    #[test]
    fn resolve_duplicate_submodel_aliases() {
        // create the use model list with duplicate submodel names
        // > use temperature as temp
        let temp_model1 = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        // > use pressure as temp (duplicate alias)
        let temp_model2 = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("other_temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let use_models = vec![&temp_model1, &temp_model2];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let other_temperature_path = ir::ModelPath::new("/other_temperature");

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (other_temperature_path, test_ir::empty_model()),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(use_models, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let temp_id = ir::ReferenceName::new("temp".to_string());
        let error = reference_errors.get(&temp_id).expect("error should exist");
        let ModelImportResolutionError::DuplicateReference {
            reference,
            original_span: _,
            duplicate_span: _,
        } = error
        else {
            panic!("Expected DuplicateReference, got {error:?}");
        };

        assert_eq!(reference.as_str(), "temp");

        // check the submodels - should contain only the successful one
        assert_has_submodels!(&submodel_map, [("temperature", &temperature_path),],);

        // check the references - should only contain one reference
        assert_has_references!(&reference_map, [("temp", &temperature_path)]);
    }

    #[test]
    fn resolve_use_declaration_with_failing_submodel() {
        // create the use model list with a submodel that fails to resolve
        // > use weather.atmosphere.temperature
        let weather_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "temperature"])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&weather_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let atmosphere_path = ir::ModelPath::new("/atmosphere");

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", "/atmosphere")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (weather_path, weather_model),
            (atmosphere_path.clone(), test_ir::empty_model()),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 1);

        let error = submodel_errors
            .get(&ir::SubmodelName::new("temperature".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &atmosphere_path);
        assert_eq!(submodel.as_str(), "temperature");

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let error = reference_errors
            .get(&ir::ReferenceName::new("temperature".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &atmosphere_path);
        assert_eq!(submodel.as_str(), "temperature");

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, []);
    }

    #[test]
    fn resolve_use_declaration_with_successful_and_failing_submodels() {
        // create the use model list with both successful and failing submodels
        // > use temperature as temp  # successful
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();

        // > use weather.atmosphere.undefined  # failing
        let undefined_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "undefined"])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&temp_model, &undefined_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let atmosphere_path = ir::ModelPath::new("/atmosphere");

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", "/atmosphere")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (atmosphere_path.clone(), test_ir::empty_model()),
            (weather_path, weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 1);

        let submodel_name = ir::SubmodelName::new("undefined".to_string());
        let error = submodel_errors
            .get(&submodel_name)
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &atmosphere_path);
        assert_eq!(submodel.as_str(), "undefined");

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let reference_name = ir::ReferenceName::new("undefined".to_string());
        let error = reference_errors
            .get(&reference_name)
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &atmosphere_path);
        assert_eq!(submodel.as_str(), "undefined");

        // check the submodels - should only contain the successful one
        assert_has_submodels!(&submodel_map, [("temperature", &temperature_path)]);

        // check the references - should only contain the successful one
        assert_has_references!(&reference_map, [("temp", &temperature_path)]);
    }

    #[test]
    fn resolve_use_declaration_with_single_submodel() {
        // create the use model list with a single submodel in the with clause
        // > use weather with temperature as temp
        let temperature_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .build();

        let weather_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_submodels([temperature_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&weather_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", "/temperature")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (weather_path.clone(), weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("weather", &weather_path)]);

        // check the references
        assert_has_references!(
            &reference_map,
            [("temp", &temperature_path), ("weather", &weather_path)],
        );
    }

    #[test]
    fn resolve_use_declaration_with_multiple_submodels() {
        // create the use model list with multiple submodels in the with clause
        // > use weather with [temperature as temp, pressure as press]
        let temperature_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .build();
        let pressure_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("pressure")
            .with_alias("press")
            .build();
        let use_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_submodels([temperature_submodel, pressure_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let use_models = vec![&use_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");
        let pressure_path = ir::ModelPath::new("/pressure");
        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", "/temperature")
            .with_submodel("pressure", "/pressure")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (pressure_path.clone(), test_ir::empty_model()),
            (weather_path.clone(), weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(use_models, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("weather", &weather_path)]);

        // check the references
        assert_has_references!(
            &reference_map,
            [
                ("temp", &temperature_path),
                ("press", &pressure_path),
                ("weather", &weather_path),
            ],
        );
    }

    #[test]
    fn resolve_use_declaration_with_nested_submodel() {
        // create the use model list with a nested submodel in the with clause
        // > use weather with atmosphere.temperature as temp
        let temperature_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("atmosphere")
            .with_subcomponents(["temperature"])
            .with_alias("temp")
            .build();

        let weather_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_submodels([temperature_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&weather_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let atmosphere_path = ir::ModelPath::new("/atmosphere");
        let atmosphere_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", "/temperature")
            .build();

        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", "/atmosphere")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (atmosphere_path, atmosphere_model),
            (weather_path.clone(), weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("weather", &weather_path)]);

        // check the references
        assert_has_references!(
            &reference_map,
            [("temp", &temperature_path), ("weather", &weather_path)],
        );
    }

    #[test]
    fn resolve_use_declaration_with_failing_submodel_in_with_clause() {
        // create the use model list with a failing submodel in the with clause
        // use weather with undefined
        let undefined_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("undefined")
            .build();

        let weather_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_submodels([undefined_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&weather_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let weather_path = ir::ModelPath::new("/weather");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(weather_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 0);

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let error = reference_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &weather_path);
        assert_eq!(submodel.as_str(), "undefined");

        // check the submodels
        assert_has_submodels!(&submodel_map, [("weather", &weather_path)]);

        // check the references
        assert_has_references!(&reference_map, [("weather", &weather_path)]);
    }

    #[test]
    fn resolve_use_declaration_with_successful_and_failing_submodels_in_with_clause() {
        // create the use model list with both successful and failing submodels in the with clause
        // use weather with [temperature as temp, undefined as undefined]
        let temperature_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .build();
        let undefined_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("undefined")
            .build();
        let weather_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_submodels([temperature_submodel, undefined_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let model_imports = vec![&weather_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");
        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", "/temperature")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (weather_path.clone(), weather_model),
            (temperature_path.clone(), test_ir::empty_model()),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert_eq!(submodel_errors.len(), 0);

        // check the reference errors
        assert_eq!(reference_errors.len(), 1);

        let error = reference_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");

        let ModelImportResolutionError::UndefinedSubmodel {
            parent_model_path,
            submodel,
            reference_span: _,
        } = error
        else {
            panic!("Expected UndefinedSubmodel, got {error:?}");
        };

        assert_eq!(parent_model_path, &weather_path);
        assert_eq!(submodel.as_str(), "undefined");

        // check the submodels
        assert_has_submodels!(&submodel_map, [("weather", &weather_path)]);

        // check the references
        assert_has_references!(
            &reference_map,
            [("temp", &temperature_path), ("weather", &weather_path)],
        );
    }

    #[test]
    fn resolve_use_declaration_with_model_alias_and_submodels() {
        // create the use model list with model alias and submodels in the with clause
        // use weather as weather_model with [temperature as temp, pressure as press]
        let temperature_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .build();
        let pressure_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("pressure")
            .with_alias("press")
            .build();
        let use_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_alias("weather_model")
            .with_submodels([temperature_submodel, pressure_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();

        let import_models = vec![&use_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");
        let pressure_path = ir::ModelPath::new("/pressure");
        let weather_path = ir::ModelPath::new("/weather");
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", "/temperature")
            .with_submodel("pressure", "/pressure")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (temperature_path.clone(), test_ir::empty_model()),
            (pressure_path.clone(), test_ir::empty_model()),
            (weather_path.clone(), weather_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(import_models, &model_path, &context);

        // check the errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, [("weather", &weather_path)]);

        // check the references
        assert_has_references!(
            &reference_map,
            [
                ("temp", &temperature_path),
                ("press", &pressure_path),
                ("weather_model", &weather_path),
            ],
        );
    }

    #[test]
    fn resolve_reference() {
        // create the import model list
        // > ref temperature
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_kind(ast::ModelKind::Reference)
            .build();

        let model_imports = vec![&temp_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(temperature_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, [("temperature", &temperature_path)]);
    }

    #[test]
    fn resolve_reference_with_alias() {
        // create the import model list
        // > ref temperature as temp
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Reference)
            .build();

        let model_imports = vec![&temp_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let temperature_path = ir::ModelPath::new("/temperature");

        let context_builder = ModelContextBuilder::new()
            .with_model_context([(temperature_path.clone(), test_ir::empty_model())]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());

        // check the reference errors
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, []);

        // check the references
        assert_has_references!(&reference_map, [("temp", &temperature_path)]);
    }

    #[test]
    fn resolve_reference_with_alias_and_submodels() {
        // create the import model list
        // > ref temperature as temp with [pressure as press]
        let pressure_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("pressure")
            .with_alias("press")
            .build();

        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Reference)
            .with_submodels([pressure_submodel])
            .build();

        let model_imports = vec![&temp_model];

        // create the current model path
        let model_path = ir::ModelPath::new("/parent_model");

        // create the context
        let pressure_path = ir::ModelPath::new("/pressure");
        let temperature_path = ir::ModelPath::new("/temperature");
        let temperature_model = test_ir::ModelBuilder::new()
            .with_submodel("pressure", "/pressure")
            .build();

        let context_builder = ModelContextBuilder::new().with_model_context([
            (pressure_path.clone(), test_ir::empty_model()),
            (temperature_path.clone(), temperature_model),
        ]);
        let context = context_builder.build();

        // resolve the submodels
        let (submodel_map, reference_map, submodel_errors, reference_errors) =
            resolve_model_imports(model_imports, &model_path, &context);

        // check the submodel errors
        assert!(submodel_errors.is_empty());
        assert!(reference_errors.is_empty());

        // check the submodels
        assert_has_submodels!(&submodel_map, []);
        // check the references
        assert_has_references!(
            &reference_map,
            [("temp", &temperature_path), ("press", &pressure_path)]
        );
    }
}
