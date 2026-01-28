//! Submodel resolution for the Oneil model loader

use oneil_ast as ast;
use oneil_ir as ir;
use oneil_shared::span::Span;

use crate::{
    ExternalResolutionContext, ResolutionContext, error::ModelImportResolutionError,
    util::ModelResult,
};

/// Resolves submodels and their associated tests from use model declarations.
pub fn resolve_model_imports<E>(
    model_path: &ir::ModelPath,
    model_imports: Vec<&ast::UseModelNode>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    for model_import in model_imports {
        let import_path = calc_import_path(model_path, model_import);

        let (reference_name, reference_name_span) =
            get_reference_name_and_span(model_import.model_info());
        let (submodel_name, submodel_name_span) =
            get_submodel_name_and_span(model_import.model_info());

        let is_submodel = model_import.model_kind() == ast::ModelKind::Submodel;

        // check for duplicates
        let maybe_reference_duplicate_error = resolution_context
            .get_reference_from_active_model(&reference_name)
            .map(|original_reference| {
                ModelImportResolutionError::duplicate_reference(
                    reference_name.clone(),
                    *original_reference.name_span(),
                    reference_name_span,
                )
            });

        let maybe_submodel_duplicate_error = resolution_context
            .get_submodel_from_active_model(&submodel_name)
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
            let submodel_name = (is_submodel).then(|| submodel_name.clone());
            resolution_context.add_model_import_resolution_error_to_active_model(
                reference_name.clone(),
                submodel_name,
                reference_duplicate_error,
            );
        }

        // handle duplicate submodels if the use model is a submodel
        if let Some(submodel_duplicate_error) = maybe_submodel_duplicate_error {
            resolution_context.add_model_import_resolution_error_to_active_model(
                reference_name.clone(),
                Some(submodel_name.clone()),
                submodel_duplicate_error,
            );
        }

        // if there were any duplicates, stop processing this use model
        if had_duplicate {
            continue;
        }

        // resolve the path for the use model
        let subcomponents = model_import.model_info().subcomponents();
        let model_name_span = submodel_name_span;
        let resolved_path = resolve_model_path(
            import_path,
            model_name_span,
            subcomponents,
            resolution_context,
        );

        // handle the error if there was one
        let resolved_path = match resolved_path {
            Ok(resolved_path) => resolved_path,
            Err(error) => {
                handle_resolution_error(
                    *error,
                    model_import,
                    reference_name,
                    submodel_name,
                    submodel_name_span,
                    is_submodel,
                    resolution_context,
                );

                continue;
            }
        };

        // add the submodel to the active model if it's a submodel
        if is_submodel {
            resolution_context.add_submodel_to_active_model(
                submodel_name,
                submodel_name_span,
                reference_name.clone(),
            );
        }

        // add the reference to the active model
        resolution_context.add_reference_to_active_model(
            reference_name,
            reference_name_span,
            resolved_path.clone(),
        );

        let Some(submodel_list) = model_import.imported_submodels() else {
            // if we don't have any imported submodels, we're done
            continue;
        };

        resolve_sumbodels(&resolved_path, submodel_list, resolution_context);
    }
}

fn resolve_sumbodels<E>(
    resolved_path: &ir::ModelPath,
    submodel_list: &oneil_ast::Node<oneil_ast::SubmodelList>,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    for submodel_info in submodel_list.iter() {
        // get the subcomponents relative to the main model being imported
        let mut submodel_subcomponents = submodel_info.subcomponents().to_vec();
        submodel_subcomponents.insert(0, submodel_info.top_component().clone());

        // get the reference name for the submodel
        let (reference_name, reference_name_span) = get_reference_name_and_span(submodel_info);

        // check for duplicate references
        let maybe_original_reference =
            resolution_context.get_reference_from_active_model(&reference_name);
        if let Some(original_reference) = maybe_original_reference {
            // if there is a duplicate, add the error and continue
            let error = ModelImportResolutionError::duplicate_reference(
                reference_name.clone(),
                *original_reference.name_span(),
                reference_name_span,
            );

            resolution_context.add_model_import_resolution_error_to_active_model(
                reference_name.clone(),
                None,
                error,
            );

            continue;
        }

        // resolve the reference path
        let resolved_reference_path = resolve_model_path(
            resolved_path.clone(),
            reference_name_span,
            &submodel_subcomponents,
            resolution_context,
        );

        match resolved_reference_path {
            Ok(resolved_reference_path) => {
                resolution_context.add_reference_to_active_model(
                    reference_name,
                    reference_name_span,
                    resolved_reference_path,
                );
            }
            Err(error) => {
                resolution_context.add_model_import_resolution_error_to_active_model(
                    reference_name,
                    None,
                    *error,
                );
            }
        }
    }
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

fn calc_import_path(model_path: &ir::ModelPath, model_import: &ast::UseModelNode) -> ir::ModelPath {
    let model_import_relative_path = model_import.get_model_relative_path();
    let model_import_path = model_path.get_sibling_path(&model_import_relative_path);

    ir::ModelPath::new(model_import_path)
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
fn resolve_model_path<E>(
    model_path: ir::ModelPath,
    model_name_span: Span,
    model_subcomponents: &[ast::IdentifierNode],
    resolution_context: &mut ResolutionContext<'_, E>,
) -> Result<ir::ModelPath, Box<ModelImportResolutionError>>
where
    E: ExternalResolutionContext,
{
    // if the model that we are trying to resolve has had an error, this
    // operation should fail
    let model = match resolution_context.lookup_model(&model_path) {
        ModelResult::Found(model) => model,
        ModelResult::HasError => {
            return Err(Box::new(ModelImportResolutionError::model_has_error(
                model_path,
                model_name_span,
            )));
        }
        ModelResult::NotFound => unreachable!("model should have been visited already"),
    };

    // if there are no more subcomponents, we have resolved the model path
    if model_subcomponents.is_empty() {
        return Ok(model_path);
    }

    let submodel_name = ir::SubmodelName::new(model_subcomponents[0].as_str().to_string());
    let submodel_name_span = model_subcomponents[0].span();
    let submodel_reference = model
        .get_submodel_reference(&submodel_name)
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
        submodel_reference.path().clone(),
        submodel_name_span,
        submodel_subcomponents,
        resolution_context,
    )
}

fn handle_resolution_error<E>(
    error: ModelImportResolutionError,
    model_import: &oneil_ast::Node<oneil_ast::UseModel>,
    reference_name: oneil_ir::ReferenceName,
    submodel_name: oneil_ir::SubmodelName,
    submodel_name_span: Span,
    is_submodel: bool,
    resolution_context: &mut ResolutionContext<'_, E>,
) where
    E: ExternalResolutionContext,
{
    if is_submodel {
        resolution_context.add_model_import_resolution_error_to_active_model(
            reference_name,
            Some(submodel_name.clone()),
            error,
        );
    } else {
        resolution_context.add_model_import_resolution_error_to_active_model(
            reference_name,
            None,
            error,
        );
    }

    let Some(submodel_list) = model_import.imported_submodels() else {
        // if we don't have any submodels, we're done
        return;
    };

    let parent_model_name = submodel_name;
    let parent_model_name_span = submodel_name_span;

    for submodel_info in submodel_list.iter() {
        // this is a bit hacky, but it's necessary to avoid getting confusing "undefined reference" errors
        let (reference_name, reference_name_span) = get_reference_name_and_span(submodel_info);

        let error = ModelImportResolutionError::parent_model_has_error(
            parent_model_name.clone(),
            parent_model_name_span,
            reference_name.clone(),
            reference_name_span,
        );

        resolution_context.add_model_import_resolution_error_to_active_model(
            reference_name.clone(),
            Some(parent_model_name.clone()),
            error,
        );
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::test::{
        external_context::TestExternalContext, resolution_context::ResolutionContextBuilder,
        test_ast, test_ir,
    };

    use super::*;
    use oneil_ast as ast;
    use oneil_ir as ir;

    /// Asserts that the submodel map contains exactly the expected submodels.
    /// Uses the reference map to resolve each submodel's path via its reference name.
    macro_rules! assert_has_submodels {
        ($submodel_map:expr, $reference_map:expr, $expected_submodels:expr $(,)?) => {
            let submodel_map: &IndexMap<ir::SubmodelName, ir::SubmodelImport> = $submodel_map;
            let reference_map: &IndexMap<ir::ReferenceName, ir::ReferenceImport> = $reference_map;
            let expected_submodels: Vec<(&'static str, &ir::ModelPath)> =
                $expected_submodels.into_iter().collect();

            // check that the submodel map length is the same as the number of submodels
            assert_eq!(
                submodel_map.len(),
                expected_submodels.len(),
                "length of *actual* submodel map differs from *expected* submodel map",
            );

            for (submodel_name, expected_path) in expected_submodels {
                let submodel_name = ir::SubmodelName::new(submodel_name.to_string());
                let submodel_import = submodel_map.get(&submodel_name).expect(
                    format!("did not find submodel for '{}'", submodel_name.as_str()).as_str(),
                );
                let reference_import = reference_map
                    .get(submodel_import.reference_name())
                    .expect("submodel's reference should exist");

                assert_eq!(
                    reference_import.path(),
                    expected_path,
                    "actual submodel path for '{}' differs from expected",
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
            let reference_map: &IndexMap<ir::ReferenceName, ir::ReferenceImport> = $reference_map;
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
        // build the model import list:
        // > use temperature as temp
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (temperature at sibling path so lookup finds it)
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(temperature_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the resolved references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_nested_submodel() {
        // build the model import list:
        // > use weather.atmosphere.temperature as temp
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "temperature"])
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (models in dependency order; paths as siblings so lookup finds them)
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let atmosphere_path = ir::ModelPath::new(weather_path.get_sibling_path("atmosphere"));
        let temperature_path = ir::ModelPath::new(atmosphere_path.get_sibling_path("temperature"));
        let atmosphere_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", &temperature_path)
            .build();
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", &atmosphere_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (atmosphere_path, atmosphere_model),
                (weather_path, weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the resolved references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_submodel_without_alias() {
        // build the model import list:
        // > use temperature  # (no alias, reference name is "temperature")
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (temperature at sibling path)
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(temperature_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the resolved references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_submodel_with_subcomponent_alias() {
        // build the model import list:
        // > use weather.atmosphere  # (subcomponent name as alias)
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (weather and atmosphere at sibling paths)
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let atmosphere_path = ir::ModelPath::new(weather_path.get_sibling_path("atmosphere"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", &atmosphere_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (atmosphere_path.clone(), test_ir::empty_model()),
                (weather_path, weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("atmosphere", &atmosphere_path)],
        );

        // check the resolved references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("atmosphere", &atmosphere_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_model_with_error() {
        // build the model import list:
        // > use error_model as error  # (model has error)
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("error_model")
            .with_alias("error")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (error_model at sibling path, marked as having an error)
        let model_path = ir::ModelPath::new("/parent_model");
        let error_path = ir::ModelPath::new(model_path.get_sibling_path("error_model"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(error_path.clone(), test_ir::empty_model())])
            .with_model_errors([error_path.clone()])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (none; import failed)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the resolved references (none; import failed)
        assert_has_references!(resolution_context.get_active_model_references(), []);

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);

        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("error".to_string()))
            .expect("error should exist");

        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("error_model".to_string()))
        );

        let ModelImportResolutionError::ModelHasError {
            model_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };
        assert_eq!(model_path, &error_path);
    }

    #[test]
    fn resolve_undefined_submodel() {
        // build the model import list:
        // > use weather.undefined_submodel  # (weather has no such submodel)
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["undefined_submodel"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (weather at sibling path, empty so no undefined_submodel)
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(weather_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (none; import failed)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the resolved references (none; import failed)
        assert_has_references!(resolution_context.get_active_model_references(), []);

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);

        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("undefined_submodel".to_string()))
            .expect("error should exist");
        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("undefined_submodel".to_string()))
        );

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
    }

    #[test]
    fn resolve_undefined_submodel_in_submodel() {
        // build the model import list:
        // > use weather.atmosphere.undefined  # (atmosphere has no "undefined")
        let model_import = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "undefined"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&model_import];

        // build the context (weather and atmosphere at sibling paths; atmosphere has no "undefined")
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let atmosphere_path = ir::ModelPath::new(weather_path.get_sibling_path("atmosphere"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", &atmosphere_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (atmosphere_path.clone(), test_ir::empty_model()),
                (weather_path, weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (none; import failed)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the resolved references (none; import failed)
        assert_has_references!(resolution_context.get_active_model_references(), []);

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);

        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");
        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("undefined".to_string()))
        );

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
    }

    #[test]
    fn resolve_multiple_submodels() {
        // build the model import list:
        // > use temperature as temp
        // > use pressure as press
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let press_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("pressure")
            .with_alias("press")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&temp_model, &press_model];

        // build the context (temperature and pressure at sibling paths)
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let pressure_path = ir::ModelPath::new(model_path.get_sibling_path("pressure"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (pressure_path.clone(), test_ir::empty_model()),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [
                ("temperature", &temperature_path),
                ("pressure", &pressure_path),
            ],
        );

        // check the resolved references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path), ("press", &pressure_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_mixed_success_and_error() {
        // build the model import list:
        // > use temperature as temp  # (success)
        // > use error_model as error  # (error)
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let error_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("error_model")
            .with_alias("error")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&temp_model, &error_model];

        // build the context (temperature and error_model at sibling paths; error_model marked as having error)
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let error_path = ir::ModelPath::new(model_path.get_sibling_path("error_model"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (error_path.clone(), test_ir::empty_model()),
            ])
            .with_model_errors([error_path.clone()])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (only temperature)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the resolved references (only temp)
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path)],
        );

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);

        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("error".to_string()))
            .expect("error should exist");
        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("error_model".to_string()))
        );

        let ModelImportResolutionError::ModelHasError {
            model_path: err_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };
        assert_eq!(err_path, &error_path);
    }

    #[test]
    fn resolve_submodel_with_directory_path_success() {
        // build the model import list:
        // > use utils/math as math  # (directory path)
        let math_model = test_ast::ImportModelNodeBuilder::new()
            .with_directory_path(["utils"])
            .with_top_component("math")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&math_model];

        // build the context (math at sibling path utils/math)
        let model_path = ir::ModelPath::new("/parent_model");
        let math_path = ir::ModelPath::new(model_path.get_sibling_path("utils/math"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(math_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("math", &math_path)],
        );

        // check the resolved references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("math", &math_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_submodel_with_directory_path_error() {
        // build the model import list:
        // > use nonexistent/math as math  # (model has error)
        let math_model = test_ast::ImportModelNodeBuilder::new()
            .with_directory_path(["nonexistent"])
            .with_top_component("math")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&math_model];

        // build the context (math at sibling path nonexistent/math, marked as having error)
        let model_path = ir::ModelPath::new("/parent_model");
        let math_path = ir::ModelPath::new(model_path.get_sibling_path("nonexistent/math"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(math_path.clone(), test_ir::empty_model())])
            .with_model_errors([math_path.clone()])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (none; import failed)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the resolved references (none; import failed)
        assert_has_references!(resolution_context.get_active_model_references(), []);

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);

        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("math".to_string()))
            .expect("error should exist");
        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("math".to_string()))
        );

        let ModelImportResolutionError::ModelHasError {
            model_path: err_path,
            reference_span: _,
        } = error
        else {
            panic!("Expected ModelHasError, got {error:?}");
        };
        assert_eq!(err_path, &math_path);
    }

    #[test]
    fn resolve_duplicate_submodel_aliases() {
        // build the model import list:
        // > use temperature as temp
        // > use other_temperature as temp  # (duplicate alias)
        let temp_model1 = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let temp_model2 = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("other_temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&temp_model1, &temp_model2];

        // build the context (temperature and other_temperature at sibling paths)
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let other_temperature_path =
            ir::ModelPath::new(model_path.get_sibling_path("other_temperature"));
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (other_temperature_path, test_ir::empty_model()),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (only first; second failed due to duplicate alias)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the resolved references (only temp -> temperature)
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path)],
        );

        // check the errors (duplicate reference "temp")
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);
        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("temp".to_string()))
            .expect("error should exist");

        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("other_temperature".to_string()))
        );

        let ModelImportResolutionError::DuplicateReference {
            reference,
            original_span: _,
            duplicate_span: _,
        } = error
        else {
            panic!("Expected DuplicateReference, got {error:?}");
        };
        assert_eq!(reference.as_str(), "temp");
    }

    #[test]
    fn resolve_use_declaration_with_failing_submodel() {
        // build the model import list:
        // > use weather.atmosphere.temperature  # (atmosphere has no temperature)
        let weather_model_ast = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "temperature"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&weather_model_ast];

        // build the context (weather and atmosphere at sibling paths; atmosphere has no temperature)
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let atmosphere_path = ir::ModelPath::new(weather_path.get_sibling_path("atmosphere"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", &atmosphere_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (atmosphere_path.clone(), test_ir::empty_model()),
                (weather_path, weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (none; import failed)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the resolved references (none; import failed)
        assert_has_references!(resolution_context.get_active_model_references(), []);

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);
        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("temperature".to_string()))
            .expect("error should exist");
        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("temperature".to_string()))
        );

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
    }

    #[test]
    fn resolve_use_declaration_with_successful_and_failing_submodels() {
        // build the model import list:
        // > use temperature as temp  # (success)
        // > use weather.atmosphere.undefined  # (fail)
        let temp_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let undefined_model = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_subcomponents(["atmosphere", "undefined"])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&temp_model, &undefined_model];

        // build the context (temperature, weather, atmosphere at sibling paths; atmosphere has no "undefined")
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let atmosphere_path = ir::ModelPath::new(weather_path.get_sibling_path("atmosphere"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", &atmosphere_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (atmosphere_path.clone(), test_ir::empty_model()),
                (weather_path, weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (only temperature)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the resolved references (only temp)
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path)],
        );

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);
        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");
        assert_eq!(
            submodel_name,
            &Some(ir::SubmodelName::new("undefined".to_string()))
        );

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
    }

    #[test]
    fn resolve_use_declaration_with_single_submodel() {
        // build the model import list:
        // > use weather with temperature as temp
        let temperature_submodel = test_ast::ModelInfoNodeBuilder::new()
            .with_top_component("temperature")
            .with_alias("temp")
            .build();
        let weather_model_ast = test_ast::ImportModelNodeBuilder::new()
            .with_top_component("weather")
            .with_submodels([temperature_submodel])
            .with_kind(ast::ModelKind::Submodel)
            .build();
        let model_imports: Vec<&ast::UseModelNode> = vec![&weather_model_ast];

        // build the context (weather and temperature at sibling paths)
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let temperature_path = ir::ModelPath::new(weather_path.get_sibling_path("temperature"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", &temperature_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (weather_path.clone(), weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (weather as submodel)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the resolved references (temp -> temperature, weather -> weather_path)
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path), ("weather", &weather_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }

    #[test]
    fn resolve_use_declaration_with_multiple_submodels() {
        // build the model import list:
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
        let model_imports: Vec<&ast::UseModelNode> = vec![&use_model];

        // build the context (weather, temperature, pressure at sibling paths)
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let temperature_path = ir::ModelPath::new(weather_path.get_sibling_path("temperature"));
        let pressure_path = ir::ModelPath::new(weather_path.get_sibling_path("pressure"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", &temperature_path)
            .with_submodel("pressure", &pressure_path)
            .build();
        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (pressure_path.clone(), test_ir::empty_model()),
                (weather_path.clone(), weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // run the resolution
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the resolved submodels (weather as submodel)
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the resolved references (temp, press, weather)
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [
                ("temp", &temperature_path),
                ("press", &pressure_path),
                ("weather", &weather_path),
            ],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
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

        // create the current model path and sibling paths used by the resolver
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let atmosphere_path = ir::ModelPath::new(weather_path.get_sibling_path("atmosphere"));
        let temperature_path = ir::ModelPath::new(atmosphere_path.get_sibling_path("temperature"));

        let atmosphere_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", &temperature_path)
            .build();
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("atmosphere", &atmosphere_path)
            .build();

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (atmosphere_path, atmosphere_model),
                (weather_path.clone(), weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path), ("weather", &weather_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
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

        // create the current model path and sibling path for weather
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(weather_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);

        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");
        assert_eq!(submodel_name, &None); // because it is in a with clause and therefore is a reference

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

        // create the current model path and sibling paths
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let temperature_path = ir::ModelPath::new(weather_path.get_sibling_path("temperature"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", &temperature_path)
            .build();

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (weather_path.clone(), weather_model),
                (temperature_path.clone(), test_ir::empty_model()),
            ])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path), ("weather", &weather_path)],
        );

        // check the errors
        let model_import_errors = resolution_context.get_active_model_model_import_errors();
        assert_eq!(model_import_errors.len(), 1);
        let (submodel_name, error) = model_import_errors
            .get(&ir::ReferenceName::new("undefined".to_string()))
            .expect("error should exist");
        assert_eq!(submodel_name, &None); // because it is in a with clause and therefore is a reference

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

        // create the current model path and sibling paths
        let model_path = ir::ModelPath::new("/parent_model");
        let weather_path = ir::ModelPath::new(model_path.get_sibling_path("weather"));
        let temperature_path = ir::ModelPath::new(weather_path.get_sibling_path("temperature"));
        let pressure_path = ir::ModelPath::new(weather_path.get_sibling_path("pressure"));
        let weather_model = test_ir::ModelBuilder::new()
            .with_submodel("temperature", &temperature_path)
            .with_submodel("pressure", &pressure_path)
            .build();

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (temperature_path.clone(), test_ir::empty_model()),
                (pressure_path.clone(), test_ir::empty_model()),
                (weather_path.clone(), weather_model),
            ])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, import_models, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [("weather", &weather_path)],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [
                ("temp", &temperature_path),
                ("press", &pressure_path),
                ("weather_model", &weather_path),
            ],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
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

        // create the current model path and sibling path for the ref target
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(temperature_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temperature", &temperature_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
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

        // create the current model path and sibling path for the ref target
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([(temperature_path.clone(), test_ir::empty_model())])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
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

        // create the current model path and sibling paths (ref target temperature, then pressure under it)
        let model_path = ir::ModelPath::new("/parent_model");
        let temperature_path = ir::ModelPath::new(model_path.get_sibling_path("temperature"));
        let pressure_path = ir::ModelPath::new(temperature_path.get_sibling_path("pressure"));
        let temperature_model = test_ir::ModelBuilder::new()
            .with_submodel("pressure", &pressure_path)
            .build();

        let mut external = TestExternalContext::new();
        let mut resolution_context = ResolutionContextBuilder::new()
            .with_active_model(model_path.clone())
            .with_models([
                (pressure_path.clone(), test_ir::empty_model()),
                (temperature_path.clone(), temperature_model),
            ])
            .with_external_context(&mut external)
            .build();

        // resolve the submodels
        resolve_model_imports(&model_path, model_imports, &mut resolution_context);

        // check the submodels
        assert_has_submodels!(
            resolution_context.get_active_model_submodels(),
            resolution_context.get_active_model_references(),
            [],
        );

        // check the references
        assert_has_references!(
            resolution_context.get_active_model_references(),
            [("temp", &temperature_path), ("press", &pressure_path)],
        );

        // check the errors
        assert!(
            resolution_context
                .get_active_model_model_import_errors()
                .is_empty()
        );
    }
}
