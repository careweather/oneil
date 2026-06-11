//! Resolving model paths and import definitions through the instance graph.

use oneil_runtime::Runtime;
use oneil_shared::{InstancePath, paths::ModelPath, symbols::ReferenceName};

/// Resolves a reference name to the on-disk model path it imports from `model_path`.
pub fn resolve_import_reference_model_path(
    runtime: &mut Runtime,
    model_path: &ModelPath,
    reference_name: &ReferenceName,
) -> Option<ModelPath> {
    let (model, _, _) = runtime.load_and_lower(model_path);
    let model = model?;

    model
        .reference_imports()
        .get(reference_name)
        .map(|reference| reference.path.clone())
        .or_else(|| {
            model
                .submodel_imports()
                .get(reference_name)
                .map(|submodel| submodel.instance.path().clone())
        })
}

/// Resolves the model path reached by following `instance_path` from `start_path`.
pub fn resolve_instance_path_model_path(
    runtime: &Runtime,
    start_path: &ModelPath,
    instance_path: Option<&InstancePath>,
) -> Result<ModelPath, String> {
    let Some(instance_path) = instance_path.filter(|path| !path.is_empty()) else {
        return Ok(start_path.clone());
    };

    let mut current_path = start_path.clone();

    for segment in instance_path.segments() {
        current_path = resolve_segment_model_path(runtime, &current_path, segment)?;
    }

    Ok(current_path)
}

/// Returns the model path reached by following one reference-name segment from `current_path`.
fn resolve_segment_model_path(
    runtime: &Runtime,
    current_path: &ModelPath,
    segment: &ReferenceName,
) -> Result<ModelPath, String> {
    let (Some(model), _) = runtime.get_loaded_model(current_path) else {
        return Err("could not load model along instance path".to_string());
    };

    if let Some(submodel) = model.submodel_imports().get(segment) {
        return Ok(submodel.instance.path().clone());
    }

    if let Some(reference) = model.reference_imports().get(segment) {
        return Ok(reference.path.clone());
    }

    if let Some(alias) = model.alias_imports().get(segment) {
        let alias_path = alias.alias_path.clone();
        return resolve_alias_path(runtime, current_path, &alias_path);
    }

    Err(format!(
        "no submodel, reference, or alias named '{}'",
        segment.as_str()
    ))
}

/// Returns the model path at the end of an extracted alias path.
fn resolve_alias_path(
    runtime: &Runtime,
    host_path: &ModelPath,
    alias_path: &InstancePath,
) -> Result<ModelPath, String> {
    let mut current_path = host_path.clone();

    for segment in alias_path.segments() {
        let (Some(model), _) = runtime.get_loaded_model(&current_path) else {
            return Err("could not load model along alias path".to_string());
        };

        let submodel_imports = model.submodel_imports();

        let Some(submodel) = submodel_imports.get(segment) else {
            return Err(format!(
                "no submodel named '{}' along alias path",
                segment.as_str()
            ));
        };

        current_path = submodel.instance.path().clone();
    }

    Ok(current_path)
}
