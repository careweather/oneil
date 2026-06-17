//! Collecting source spans for symbol occurrences across loaded models.

use indexmap::IndexSet;
use oneil_frontend::ModelDesignInfo;
use oneil_runtime::{
    Runtime,
    output::{ir, reference::ModelTemplateReference},
};
use oneil_shared::{
    paths::ModelPath,
    span::Span,
    symbols::{ParameterName, ReferenceName},
};

use crate::{model_navigation::resolve_instance_path_model_path, symbol_lookup::SymbolAtPosition};

/// What symbol occurrences to collect across the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTarget {
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

/// A single source occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub model_path: ModelPath,
    pub span: Span,
}

/// Resolves the symbol under the cursor to a search target, if occurrence lookup is supported.
pub fn resolve_search_target(
    symbol: &SymbolAtPosition,
    runtime: &mut Runtime,
    current_model_path: &ModelPath,
) -> Option<SearchTarget> {
    match symbol {
        SymbolAtPosition::ExternalParameterReference {
            reference_name,
            parameter_name,
            ..
        } => {
            let model = runtime.load_and_lower(current_model_path).0;
            let model = model?;
            let model_path = model.resolve_reference_model_path(reference_name)?;
            Some(SearchTarget::Parameter {
                model_path,
                name: parameter_name.clone(),
            })
        }
        SymbolAtPosition::ModelImportAlias {
            alias: reference_name,
            ..
        }
        | SymbolAtPosition::ModelImportReference { reference_name, .. } => {
            Some(SearchTarget::ImportAlias {
                model_path: current_model_path.clone(),
                name: reference_name.clone(),
            })
        }
        SymbolAtPosition::ParameterDefinition { name, .. }
        | SymbolAtPosition::ParameterReference { name, .. }
        | SymbolAtPosition::DesignParameterAddition { name, .. } => Some(SearchTarget::Parameter {
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

            Some(SearchTarget::Parameter {
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

            Some(SearchTarget::ImportAlias {
                model_path: effective_parent_path,
                name: self_ref_name,
            })
        }
        SymbolAtPosition::ApplyTargetReference { reference_name, .. } => {
            let (_, design_info_opt, _) = runtime.load_and_lower(current_model_path);
            let design_info = design_info_opt?;
            let design_export = design_info.design_export.as_ref()?;
            let (target_model_path, _) = design_export.target_model()?;

            Some(SearchTarget::ImportAlias {
                model_path: target_model_path.clone(),
                name: reference_name.clone(),
            })
        }
        SymbolAtPosition::ModelImportDefinition { .. }
        | SymbolAtPosition::BuiltinValueReference { .. }
        | SymbolAtPosition::BuiltinFunctionReference { .. }
        | SymbolAtPosition::PythonImport { .. }
        | SymbolAtPosition::PythonFunctionReference { .. }
        | SymbolAtPosition::DesignTarget { .. }
        | SymbolAtPosition::ApplyDesignPath { .. } => None,
    }
}

/// Collects all source spans that match `target` across loaded models.
pub fn collect_occurrences(target: &SearchTarget, runtime: &Runtime) -> Vec<Occurrence> {
    let mut occurrences = Vec::new();
    match target {
        SearchTarget::Parameter { model_path, name } => {
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
        SearchTarget::ImportAlias { model_path, name } => {
            let (model, _) = runtime.get_loaded_model(model_path);
            let model = model.expect("model must be loaded");

            collect_import_alias_definition_occurrences(model, name, &mut occurrences);

            let mode = VariableRenameMode::ImportAlias {
                import_alias_name: name,
            };

            collect_parameter_occurrences(model, mode, &mut occurrences);

            let designs_referencing_model = get_designs_referencing_model(model_path, runtime);

            for loaded_path in designs_referencing_model {
                let (model, design_info) = runtime.get_loaded_model(&loaded_path);
                let model = model.expect("model must be loaded");

                if let Some(design_info) = design_info.as_ref() {
                    collect_design_parameter_occurrences(
                        model,
                        design_info,
                        mode,
                        runtime,
                        &mut occurrences,
                    );
                }
            }
        }
    }
    occurrences
}

fn collect_parameter_occurrences(
    model: ModelTemplateReference<'_>,
    mode: VariableRenameMode<'_>,
    occurrences: &mut Vec<Occurrence>,
) {
    for param in model.parameters().values() {
        if let VariableRenameMode::LocalParameter {
            parameter_name: name,
        } = mode
            && param.name() == name
        {
            let model_path = model.path().clone();
            let span = param.name_span().clone();
            occurrences.push(Occurrence { model_path, span });
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
    occurrences: &mut Vec<Occurrence>,
) {
    if let Some(design_export) = design_info.design_export.as_ref() {
        if !matches!(mode, VariableRenameMode::ImportAlias { .. }) {
            for param in design_export.parameter_additions() {
                // if renaming a local parameter and the parameter is defined in the design,
                // add the parameter name span to the occurrences
                if let VariableRenameMode::LocalParameter {
                    parameter_name: name,
                } = mode
                    && param.name() == name
                {
                    let model_path = model.path().clone();
                    let span = param.name_span().clone();
                    occurrences.push(Occurrence { model_path, span });
                }

                collect_parameter_value(model, None, param.value(), mode, occurrences);

                collect_limits(model, None, param.limits(), mode, occurrences);
            }

            for test in design_export.test_additions() {
                collect_expr(model, None, test.expr(), mode, occurrences);
            }
        }

        if let VariableRenameMode::ImportAlias { import_alias_name } = mode {
            for applied_design in &design_info.applied_designs {
                let (first_segment, first_segment_span) = applied_design
                    .target_segments
                    .first()
                    .expect("target must have at least one segment");

                if first_segment == import_alias_name {
                    let model_path = model.path().clone();
                    let span = first_segment_span.clone();
                    occurrences.push(Occurrence { model_path, span });
                }
            }
        }

        for (param_name, overlay) in design_export.parameter_overrides() {
            // if renaming a local parameter and the parameter is overridden in the design,
            // add the design span to the occurrences
            if let VariableRenameMode::LocalParameter {
                parameter_name: name,
            } = mode
                && param_name == name
            {
                let model_path = model.path().clone();
                let span = overlay.design_span.clone();
                occurrences.push(Occurrence { model_path, span });
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
                    let model_path = model.path().clone();
                    let span = overlay.design_span.clone();
                    occurrences.push(Occurrence { model_path, span });
                }
            }

            if let VariableRenameMode::ImportAlias { import_alias_name } = mode {
                let first_segment = param_instance_path
                    .segments()
                    .first()
                    .expect("instance path must have at least one segment");

                if first_segment == import_alias_name {
                    let model_path = model.path().clone();
                    let span = overlay
                        .instance_path_span
                        .clone()
                        .expect("instance path span must be present");
                    occurrences.push(Occurrence { model_path, span });
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
    occurrences: &mut Vec<Occurrence>,
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
    occurrences: &mut Vec<Occurrence>,
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
    occurrences: &mut Vec<Occurrence>,
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
    occurrences: &mut Vec<Occurrence>,
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
    occurrences: &mut Vec<Occurrence>,
) {
    match mode {
        VariableRenameMode::LocalParameter { parameter_name } => {
            if let ir::Variable::Parameter {
                parameter_name: current_parameter_name,
                parameter_span,
            } = variable
                && current_parameter_name == parameter_name
            {
                let model_path = model.path().clone();
                let span = parameter_span.clone();
                occurrences.push(Occurrence { model_path, span });
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
                && model
                    .resolve_reference_model_path(reference_name)
                    .as_ref()
                    .is_some_and(|path| external_model_paths.contains(path))
            {
                let model_path = model.path().clone();
                let span = parameter_span.clone();
                occurrences.push(Occurrence { model_path, span });
            } else if current_parameter_name == parameter_name
                && let Some(target_model) = target_model
                && target_model
                    .resolve_reference_model_path(reference_name)
                    .as_ref()
                    .is_some_and(|path| external_model_paths.contains(path))
            {
                let model_path = target_model.path().clone();
                let span = parameter_span.clone();
                occurrences.push(Occurrence { model_path, span });
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
                let model_path = model.path().clone();
                let span = reference_span.clone();
                occurrences.push(Occurrence { model_path, span });
            }
        }
    }
}

/// Collects alias-definition spans for an explicit `as` import alias.
fn collect_import_alias_definition_occurrences(
    model: ModelTemplateReference<'_>,
    name: &ReferenceName,
    occurrences: &mut Vec<Occurrence>,
) {
    for reference_import in model.reference_imports().values() {
        if reference_import.alias.as_ref() == Some(name)
            && let Some(span) = reference_import.alias_span.as_ref()
        {
            let model_path = model.path().clone();
            let span = span.clone();
            occurrences.push(Occurrence { model_path, span });
        }
    }

    for submodel_import in model.submodel_imports().values() {
        if submodel_import.alias.as_ref() == Some(name)
            && let Some(span) = submodel_import.alias_span.as_ref()
        {
            let model_path = model.path().clone();
            let span = span.clone();
            occurrences.push(Occurrence { model_path, span });
        }
    }

    for alias_import in model.alias_imports().values() {
        if alias_import.alias.as_ref() == Some(name)
            && let Some(span) = alias_import.alias_span.as_ref()
        {
            let model_path = model.path().clone();
            let span = span.clone();
            occurrences.push(Occurrence { model_path, span });
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
