//! Go-to-definition resolution for [`crate::symbol_lookup::SymbolAtPosition`].

use std::path::Path;

use oneil_runtime::Runtime;
use oneil_shared::{InstancePath, paths::ModelPath, symbols::ParameterName};
use tower_lsp_server::ls_types::{Location, Position, Range, Uri};

use crate::{
    location::{python_function_line_to_location, span_to_location},
    model_navigation::resolve_instance_path_model_path,
    symbol_lookup::SymbolAtPosition,
};

/// Resolves a symbol to its definition location.
pub fn resolve_definition(
    symbol: &SymbolAtPosition,
    runtime: &mut Runtime,
    current_model_path: &ModelPath,
) -> Option<Location> {
    match symbol {
        SymbolAtPosition::ParameterDefinition { span, .. }
        | SymbolAtPosition::DesignParameterAddition { span, .. } => {
            Some(span_to_location(current_model_path, span))
        }
        SymbolAtPosition::ParameterReference { name, .. } => {
            // TODO: handle design info if it exists
            let (model, _design_info_opt, _errors) = runtime.load_and_lower(current_model_path);
            let model = model?;

            let param = model.get_parameter(name)?;

            Some(span_to_location(current_model_path, param.name_span()))
        }
        SymbolAtPosition::ExternalParameterReference {
            reference_name,
            parameter_name,
            ..
        } => {
            // Resolve the reference name to a model path via the current model's imports.
            // TODO: handle design info if it exists
            let (current_model, _design_info_opt, _errors) =
                runtime.load_and_lower(current_model_path);
            let current_model = current_model?;
            let external_model_path = current_model
                .reference_imports()
                .get(reference_name)
                .map(|r| r.path.clone())
                .or_else(|| {
                    current_model
                        .submodel_imports()
                        .get(reference_name)
                        .map(|s| s.instance.path().clone())
                })?;

            // TODO: handle design info if it exists
            let (external_model, _design_info_opt, _errors) =
                runtime.load_and_lower(&external_model_path);
            let external_model = external_model?;

            let param = external_model.get_parameter(parameter_name)?;
            Some(span_to_location(&external_model_path, param.name_span()))
        }
        SymbolAtPosition::ModelImportDefinition { path, .. }
        | SymbolAtPosition::DesignTarget { path, .. }
        | SymbolAtPosition::ApplyDesignPath { path, .. } => top_of_file(path.as_path()),
        SymbolAtPosition::ModelImportAlias {
            alias: reference_name,
            ..
        }
        | SymbolAtPosition::ModelImportReference { reference_name, .. }
        | SymbolAtPosition::ApplyTargetReference { reference_name, .. } => {
            let (model, _design_info_opt, _errors) = runtime.load_and_lower(current_model_path);
            let model = model?;

            if let Some(reference) = model.reference_imports().get(reference_name) {
                return Some(span_to_location(current_model_path, &reference.name_span));
            }

            let submodel_imports = model.submodel_imports();
            let submodel = submodel_imports.get(reference_name)?;
            Some(span_to_location(current_model_path, &submodel.name_span))
        }
        SymbolAtPosition::PythonImport { path, .. } => top_of_file(path.as_path()),
        SymbolAtPosition::PythonFunctionReference {
            python_path, name, ..
        } => {
            let function = runtime.lookup_python_function(python_path, name)?;
            let function_line_no = function.get_line_no()?;

            Some(python_function_line_to_location(
                python_path,
                function_line_no,
            ))
        }
        SymbolAtPosition::BuiltinValueReference { .. }
        | SymbolAtPosition::BuiltinFunctionReference { .. } => None,
        SymbolAtPosition::DesignParameterOverride {
            name,
            instance_path,
            ..
        } => resolve_design_parameter_override_definition(
            runtime,
            current_model_path,
            name,
            instance_path.as_ref(),
        ),
        SymbolAtPosition::DesignParameterOverrideInstancePath { instance_path, .. } => {
            resolve_design_instance_path_definition(runtime, current_model_path, instance_path)
        }
    }
}

/// Resolves a design override parameter name to its definition on the design target.
fn resolve_design_parameter_override_definition(
    runtime: &mut Runtime,
    design_file_path: &ModelPath,
    name: &ParameterName,
    instance_path: Option<&InstancePath>,
) -> Option<Location> {
    let (_, design_info_opt, _) = runtime.load_and_lower(design_file_path);
    let design_info = design_info_opt?;
    let design = design_info.design_export.as_ref()?;
    let (target_model_path, _) = design.target_model()?;
    let effective_target_path =
        resolve_instance_path_model_path(runtime, target_model_path, instance_path).ok()?;

    let (target_model, _, _) = runtime.load_and_lower(&effective_target_path);
    let target_model = target_model?;
    let param = target_model.get_parameter(name)?;

    Some(span_to_location(&effective_target_path, param.name_span()))
}

/// Resolves a scoped override instance-path segment to its import declaration.
fn resolve_design_instance_path_definition(
    runtime: &mut Runtime,
    design_file_path: &ModelPath,
    instance_path: &InstancePath,
) -> Option<Location> {
    let (_, design_info_opt, _) = runtime.load_and_lower(design_file_path);
    let design_info = design_info_opt?;
    let design = design_info.design_export.as_ref()?;
    let (target_model_path, _) = design.target_model()?;
    let reference_name = instance_path.segments().first()?;

    let (model, _, _) = runtime.load_and_lower(target_model_path);
    let model = model?;

    if let Some(reference) = model.reference_imports().get(reference_name) {
        return Some(span_to_location(target_model_path, &reference.name_span));
    }

    let submodel_imports = model.submodel_imports();
    let submodel = submodel_imports.get(reference_name)?;
    Some(span_to_location(target_model_path, &submodel.name_span))
}

fn top_of_file(path: &Path) -> Option<Location> {
    let uri = Uri::from_file_path(path)?;
    Some(Location {
        uri,
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        },
    })
}
