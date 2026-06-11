//! Rename (refactoring) support for Oneil models.

use std::collections::HashMap;

use indexmap::IndexSet;
use oneil_frontend::{ModelDesignInfo, instance::design::Design};
use oneil_runtime::{
    Runtime,
    output::{ir, reference::ModelTemplateReference},
};
use oneil_shared::{
    paths::ModelPath,
    span::Span,
    symbols::{ParameterName, ReferenceName},
};
use tower_lsp_server::ls_types::{PrepareRenameResponse, TextEdit, Uri, WorkspaceEdit};

use crate::{
    location::span_to_range, model_navigation::resolve_instance_path_model_path,
    symbol_lookup::SymbolAtPosition,
};

/// What the user is renaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameTarget {
    /// A parameter in `model_path` (definition and references).
    Parameter {
        model_path: ModelPath,
        name: ParameterName,
    },
    /// An import alias (`foo` in `submodel bar as foo`) in `model_path`.
    ImportAlias {
        model_path: ModelPath,
        name: ReferenceName,
    },
}

/// A single source occurrence to replace.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RenameOccurrence {
    model_path: ModelPath,
    span: Span,
}

/// Resolves the symbol under the cursor to a rename target, if rename is supported.
pub fn resolve_rename_target(
    symbol: &SymbolAtPosition,
    runtime: &mut Runtime,
    current_model_path: &ModelPath,
) -> Option<RenameTarget> {
    match symbol {
        SymbolAtPosition::ExternalParameterReference {
            reference_name,
            parameter_name,
            ..
        } => {
            let model = runtime.load_and_lower(current_model_path).0;
            let model = model?;
            let model_path = resolve_reference_model_path(model, reference_name)?;
            Some(RenameTarget::Parameter {
                model_path,
                name: parameter_name.clone(),
            })
        }
        SymbolAtPosition::ModelImportAlias {
            alias: reference_name,
            ..
        }
        | SymbolAtPosition::ModelImportReference { reference_name, .. } => {
            Some(RenameTarget::ImportAlias {
                model_path: current_model_path.clone(),
                name: reference_name.clone(),
            })
        }
        SymbolAtPosition::ParameterDefinition { name, .. }
        | SymbolAtPosition::ParameterReference { name, .. }
        | SymbolAtPosition::DesignParameterAddition { name, .. } => Some(RenameTarget::Parameter {
            model_path: current_model_path.clone(),
            name: name.clone(),
        }),
        SymbolAtPosition::DesignParameterOverride {
            name,
            instance_path,
            ..
        } => {
            let (_, design_info_opt, _) = runtime.load_and_lower(current_model_path);
            let design_info = design_info_opt?;
            let design_export = design_info.design_export.as_ref()?;
            let (target_model_path, _) = design_export.target_model()?;
            let effective_target_path = resolve_instance_path_model_path(
                runtime,
                target_model_path,
                instance_path.as_ref(),
            )
            .ok()?;

            Some(RenameTarget::Parameter {
                model_path: effective_target_path,
                name: name.clone(),
            })
        }
        SymbolAtPosition::DesignParameterOverrideInstancePath { instance_path, .. } => {
            let (parent_path, self_ref_name) = instance_path
                .split_parent_and_self()
                .expect("instance path must have at least one segment");

            let (_, design_info_opt, _) = runtime.load_and_lower(current_model_path);
            let design_info = design_info_opt?;
            let design_export = design_info.design_export.as_ref()?;
            let (target_model_path, _) = design_export.target_model()?;
            let effective_parent_path =
                resolve_instance_path_model_path(runtime, target_model_path, Some(&parent_path))
                    .ok()?;

            Some(RenameTarget::ImportAlias {
                model_path: effective_parent_path,
                name: self_ref_name,
            })
        }
        SymbolAtPosition::ModelImportDefinition { .. }
        | SymbolAtPosition::BuiltinValueReference { .. }
        | SymbolAtPosition::BuiltinFunctionReference { .. }
        | SymbolAtPosition::PythonImport { .. }
        | SymbolAtPosition::PythonFunctionReference { .. }
        | SymbolAtPosition::DesignTarget { .. }
        | SymbolAtPosition::ApplyDesignPath { .. }
        | SymbolAtPosition::ApplyTargetReference { .. } => None,
    }
}

fn resolve_reference_model_path(
    model: ModelTemplateReference<'_>,
    reference_name: &ReferenceName,
) -> Option<ModelPath> {
    model
        .reference_imports()
        .get(reference_name)
        .map(|r| r.path.clone())
        .or_else(|| {
            model
                .submodel_imports()
                .get(reference_name)
                .map(|s| s.instance.path().clone())
        })
}

/// Maps a cursor symbol to a prepare-rename response.
pub fn prepare_rename_response(symbol: &SymbolAtPosition) -> Option<PrepareRenameResponse> {
    let range = span_to_range(&symbol.span());
    let placeholder = match symbol {
        SymbolAtPosition::ParameterDefinition { name, .. }
        | SymbolAtPosition::ParameterReference { name, .. }
        | SymbolAtPosition::ExternalParameterReference {
            parameter_name: name,
            ..
        }
        | SymbolAtPosition::DesignParameterOverride { name, .. }
        | SymbolAtPosition::DesignParameterAddition { name, .. } => name.as_str().to_string(),
        SymbolAtPosition::ModelImportReference { reference_name, .. }
        | SymbolAtPosition::ModelImportAlias {
            alias: reference_name,
            ..
        } => reference_name.as_str().to_string(),
        SymbolAtPosition::ModelImportDefinition { .. }
        | SymbolAtPosition::BuiltinValueReference { .. }
        | SymbolAtPosition::BuiltinFunctionReference { .. }
        | SymbolAtPosition::PythonImport { .. }
        | SymbolAtPosition::PythonFunctionReference { .. }
        | SymbolAtPosition::DesignTarget { .. }
        | SymbolAtPosition::ApplyDesignPath { .. }
        | SymbolAtPosition::ApplyTargetReference { .. }
        | SymbolAtPosition::DesignParameterOverrideInstancePath { .. } => return None,
    };

    Some(PrepareRenameResponse::RangeWithPlaceholder { range, placeholder })
}

/// Builds a workspace edit that renames `target` to `new_name`.
pub fn workspace_edit_for_rename(
    target: &RenameTarget,
    new_name: &str,
    runtime: &mut Runtime,
) -> Result<WorkspaceEdit, String> {
    validate_new_name(target, new_name, runtime)?;

    let occurrences = collect_rename_occurrences(target, runtime);
    if occurrences.is_empty() {
        return Err("no occurrences to rename".to_string());
    }

    let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
    for occurrence in occurrences {
        let uri = Uri::from_file_path(occurrence.model_path.as_path()).ok_or_else(|| {
            format!(
                "could not convert path to URI: {}",
                occurrence.model_path.as_path().display()
            )
        })?;
        changes.entry(uri).or_default().push(TextEdit {
            range: span_to_range(&occurrence.span),
            new_text: new_name.to_string(),
        });
    }

    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn validate_new_name(
    target: &RenameTarget,
    new_name: &str,
    runtime: &mut Runtime,
) -> Result<(), String> {
    if !is_valid_identifier(new_name) {
        return Err(format!("'{new_name}' is not a valid identifier"));
    }

    match target {
        RenameTarget::Parameter { model_path, name } => {
            if new_name == name.as_str() {
                return Err("new name is the same as the old name".to_string());
            }

            let Some(model) = runtime.load_and_lower(model_path).0 else {
                return Err("could not load model".to_string());
            };

            if model
                .parameters()
                .contains_key(&ParameterName::from(new_name))
            {
                return Err(format!("parameter '{new_name}' already exists"));
            }
        }
        RenameTarget::ImportAlias { model_path, name } => {
            if new_name == name.as_str() {
                return Err("new name is the same as the old name".to_string());
            }

            let Some(model) = runtime.load_and_lower(model_path).0 else {
                return Err("could not load model".to_string());
            };

            // check that the given reference name is an alias, not a reference
            // or submodel name, since renaming those is a more complex operation.

            let is_reference_alias = model
                .reference_imports()
                .get(name)
                .and_then(|r| r.alias.as_ref())
                .is_some_and(|alias| alias == name);

            let is_submodel_alias = model
                .submodel_imports()
                .get(name)
                .and_then(|s| s.alias.as_ref())
                .is_some_and(|alias| alias == name);

            let is_alias_alias = model
                .alias_imports()
                .get(name)
                .and_then(|a| a.alias.as_ref())
                .is_some_and(|alias| alias == name);

            if !is_reference_alias && !is_submodel_alias && !is_alias_alias {
                let name = name.as_str();
                return Err(format!("reference name '{name}' is not an alias"));
            }

            // check that the new reference name is not already in use

            let new_reference = ReferenceName::from(new_name);
            if model.reference_imports().contains_key(&new_reference)
                || model.submodel_imports().contains_key(&new_reference)
                || model.alias_imports().contains_key(&new_reference)
            {
                return Err(format!("import alias '{new_name}' already exists"));
            }
        }
    }

    Ok(())
}

/// Returns whether `name` is a valid Oneil identifier (not checked against keywords).
fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_alphabetic() || first == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Collects all source spans that should be updated for `target`.
fn collect_rename_occurrences(target: &RenameTarget, runtime: &Runtime) -> Vec<RenameOccurrence> {
    let mut occurrences = Vec::new();
    match target {
        RenameTarget::Parameter { model_path, name } => {
            let (model, _design_info) = runtime.get_loaded_model(model_path);
            let model = model.expect("model must be loaded");

            // rename the parameter in the local model
            collect_parameter_occurrences(
                model,
                VariableRenameMode::LocalParameter {
                    parameter_name: name,
                },
                &mut occurrences,
            );

            // collect all the paths that reference the parameter model,
            // including the parameter model itself
            let mut paths_referencing_model = get_designs_referencing_model(model_path, runtime);
            paths_referencing_model.insert(model_path.clone());

            for model in runtime.get_loaded_models() {
                let (model, design_info) = runtime.get_loaded_model(&model);
                let model = model.expect("model must be loaded");

                if let Some(design_info) = design_info.as_ref()
                    && paths_referencing_model.contains(model.path())
                {
                    // in each design that references the model, rename the
                    // parameter as a local parameter
                    collect_design_parameter_occurrences(
                        model,
                        design_info,
                        VariableRenameMode::LocalParameter {
                            parameter_name: name,
                        },
                        runtime,
                        &mut occurrences,
                    );
                }

                // wherever the parameter model, or a design that references the
                // parameter model, is referenced as an external parameter,
                // rename the parameter as an external parameter
                collect_external_parameter_occurrences(
                    model,
                    design_info.as_ref(),
                    name,
                    &paths_referencing_model,
                    runtime,
                    &mut occurrences,
                );
            }
        }
        RenameTarget::ImportAlias { model_path, name } => {
            todo!()
        }
    }
    occurrences
}

fn collect_parameter_occurrences(
    model: ModelTemplateReference<'_>,
    mode: VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    for param in model.parameters().values() {
        if let VariableRenameMode::LocalParameter {
            parameter_name: name,
        } = mode
            && param.name() == name
        {
            push_occurrence(occurrences, model.path().clone(), param.name_span().clone());
        }

        collect_parameter_value(model, None, param.value(), mode, occurrences);
        collect_limits(model, None, param.limits(), mode, occurrences);
    }

    for test in model.tests().values() {
        collect_expr(model, None, test.expr(), mode, occurrences);
    }
}

fn collect_design_parameter_occurrences(
    model: ModelTemplateReference<'_>,
    design_info: &ModelDesignInfo,
    mode: VariableRenameMode<'_>,
    runtime: &Runtime,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    if let Some(design_export) = design_info.design_export.as_ref() {
        for param in design_export.parameter_additions() {
            // if renaming a local parameter and the parameter is defined in the design,
            // add the parameter name span to the occurrences
            if let VariableRenameMode::LocalParameter {
                parameter_name: name,
            } = mode
                && param.name() == name
            {
                push_occurrence(occurrences, model.path().clone(), param.name_span().clone());
            }

            collect_parameter_value(model, None, param.value(), mode, occurrences);

            collect_limits(model, None, param.limits(), mode, occurrences);
        }

        for test in design_export.test_additions() {
            collect_expr(model, None, test.expr(), mode, occurrences);
        }

        for (param_name, overlay) in design_export.parameter_overrides() {
            // if renaming a local parameter and the parameter is overridden in the design,
            // add the design span to the occurrences
            if let VariableRenameMode::LocalParameter {
                parameter_name: name,
            } = mode
                && param_name == name
            {
                push_occurrence(
                    occurrences,
                    model.path().clone(),
                    overlay.design_span.clone(),
                );
            }

            collect_parameter_value(model, None, &overlay.value, mode, occurrences);

            if let Some(limits) = overlay.limits_override.as_ref() {
                collect_limits(model, None, limits, mode, occurrences);
            }
        }

        for (param_instance_path, param_name, overlay) in design_export.scoped_parameter_overrides()
        {
            // if renaming an external parameter and the parameter is overridden in the design,
            // add the design span to the occurrences
            if let VariableRenameMode::ExternalParameter {
                external_model_paths,
                parameter_name,
            } = mode
                && param_name == parameter_name
            {
                let param_model_path = design_export
                    .target_model()
                    .and_then(|(path, _)| runtime.get_loaded_model(path).0)
                    .and_then(|model| {
                        resolve_instance_path_model_path(
                            runtime,
                            model.path(),
                            Some(param_instance_path),
                        )
                        .ok()
                    });

                if let Some(param_model_path) = param_model_path
                    && external_model_paths.contains(&param_model_path)
                {
                    push_occurrence(
                        occurrences,
                        model.path().clone(),
                        overlay.design_span.clone(),
                    );
                }
            }

            collect_parameter_value(model, None, &overlay.value, mode, occurrences);

            if let Some(limits) = overlay.limits_override.as_ref() {
                collect_limits(model, None, limits, mode, occurrences);
            }
        }
    }
}

fn collect_external_parameter_occurrences(
    model: ModelTemplateReference<'_>,
    design_info: Option<&ModelDesignInfo>,
    name: &ParameterName,
    paths_referencing_model: &IndexSet<ModelPath>,
    runtime: &Runtime,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    let mode = VariableRenameMode::ExternalParameter {
        external_model_paths: paths_referencing_model,
        parameter_name: name,
    };

    collect_parameter_occurrences(model, mode, occurrences);

    if let Some(design_info) = design_info.as_ref() {
        collect_design_parameter_occurrences(model, design_info, mode, runtime, occurrences);
    }
}

fn collect_parameter_value(
    model: ModelTemplateReference<'_>,
    target_model: Option<ModelTemplateReference<'_>>,
    value: &ir::ParameterValue,
    mode: VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match value {
        ir::ParameterValue::Simple(expr, _) => {
            collect_expr(model, target_model, expr, mode, occurrences);
        }
        ir::ParameterValue::Piecewise(exprs, _) => {
            for piecewise in exprs {
                collect_expr(model, target_model, piecewise.expr(), mode, occurrences);
                collect_expr(model, target_model, piecewise.if_expr(), mode, occurrences);
            }
        }
    }
}

fn collect_limits(
    model: ModelTemplateReference<'_>,
    target_model: Option<ModelTemplateReference<'_>>,
    limits: &ir::Limits,
    mode: VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match limits {
        ir::Limits::Default => {}
        ir::Limits::Continuous { min, max, .. } => {
            collect_expr(model, target_model, min, mode, occurrences);
            collect_expr(model, target_model, max, mode, occurrences);
        }
        ir::Limits::Discrete { values, .. } => {
            for value in values {
                collect_expr(model, target_model, value, mode, occurrences);
            }
        }
    }
}

fn collect_expr(
    model: ModelTemplateReference<'_>,
    target_model: Option<ModelTemplateReference<'_>>,
    expr: &ir::Expr,
    mode: VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    expr.walk_variables(&mut |variable| {
        visit_variable(model, target_model, variable, mode, occurrences);
    });
}

/// How to match variable occurrences while walking expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariableRenameMode<'a> {
    LocalParameter {
        parameter_name: &'a ParameterName,
    },
    ExternalParameter {
        external_model_paths: &'a IndexSet<ModelPath>,
        parameter_name: &'a ParameterName,
    },
    ImportAlias {
        import_alias_name: &'a ReferenceName,
    },
}

fn visit_variable(
    model: ModelTemplateReference<'_>,
    target_model: Option<ModelTemplateReference<'_>>,
    variable: &ir::Variable,
    mode: VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match mode {
        VariableRenameMode::LocalParameter { parameter_name } => {
            if let ir::Variable::Parameter {
                parameter_name: current_parameter_name,
                parameter_span,
            } = variable
                && current_parameter_name == parameter_name
            {
                push_occurrence(occurrences, model.path().clone(), parameter_span.clone());
            }
        }
        VariableRenameMode::ExternalParameter {
            external_model_paths,
            parameter_name,
        } => {
            let ir::Variable::External {
                reference_name,
                parameter_name: current_parameter_name,
                parameter_span,
                ..
            } = variable
            else {
                return;
            };

            if current_parameter_name == parameter_name
                && resolve_reference_model_path(model, reference_name)
                    .as_ref()
                    .is_some_and(|path| external_model_paths.contains(path))
            {
                push_occurrence(occurrences, model.path().clone(), parameter_span.clone());
            } else if current_parameter_name == parameter_name
                && let Some(target_model) = target_model
                && resolve_reference_model_path(target_model, reference_name)
                    .as_ref()
                    .is_some_and(|path| external_model_paths.contains(path))
            {
                push_occurrence(
                    occurrences,
                    target_model.path().clone(),
                    parameter_span.clone(),
                );
            }
        }
        VariableRenameMode::ImportAlias { import_alias_name } => {
            let ir::Variable::External {
                reference_name,
                reference_span,
                ..
            } = variable
            else {
                return;
            };

            if reference_name == import_alias_name {
                push_occurrence(occurrences, model.path().clone(), reference_span.clone());
            }
        }
    }
}

fn get_designs_referencing_model(
    param_model_path: &ModelPath,
    runtime: &Runtime,
) -> IndexSet<ModelPath> {
    runtime
        .get_loaded_models()
        .iter()
        .filter_map(|model| {
            let (model, design_info) = runtime.get_loaded_model(model);
            let model = model.expect("model must be loaded");

            let (path, _) = design_info
                .as_ref()?
                .design_export
                .as_ref()?
                .target_model()?;

            (path == param_model_path).then(|| model.path().clone())
        })
        .collect()
}

fn push_occurrence(occurrences: &mut Vec<RenameOccurrence>, model_path: ModelPath, span: Span) {
    occurrences.push(RenameOccurrence { model_path, span });
}
