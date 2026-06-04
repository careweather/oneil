//! Rename (refactoring) support for Oneil models.

use std::collections::HashMap;

use indexmap::IndexSet;
use oneil_runtime::{
    Runtime,
    output::{ir, reference::ModelTemplateReference},
};
use oneil_shared::{
    paths::ModelPath,
    span::Span,
    symbols::{ParameterName, ReferenceName},
};
use tower_lsp_server::ls_types::{PrepareRenameResponse, Range, TextEdit, Uri, WorkspaceEdit};

use crate::{
    location::span_to_range,
    symbol_lookup::{ModelImportName, SymbolAtPosition},
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
        SymbolAtPosition::ParameterDefinition { name, .. }
        | SymbolAtPosition::ParameterReference { name, .. } => Some(RenameTarget::Parameter {
            model_path: current_model_path.clone(),
            name: name.clone(),
        }),
        SymbolAtPosition::ExternalParameterReference {
            reference_name,
            parameter_name,
            ..
        } => {
            let model = load_model(runtime, current_model_path)?;
            let model_path = resolve_reference_model_path(model, reference_name)?;
            Some(RenameTarget::Parameter {
                model_path,
                name: parameter_name.clone(),
            })
        }
        SymbolAtPosition::ModelImportReference { reference_name, .. } => {
            Some(RenameTarget::ImportAlias {
                model_path: current_model_path.clone(),
                name: reference_name.clone(),
            })
        }
        SymbolAtPosition::ModelImportDefinition { name, .. } => match name {
            ModelImportName::Reference(reference_name) => Some(RenameTarget::ImportAlias {
                model_path: current_model_path.clone(),
                name: reference_name.clone(),
            }),
            // Submodel import `name_span` is the source model name (`foo`), not the alias (`bar`).
            ModelImportName::Submodel(_) => None,
        },
        SymbolAtPosition::BuiltinValueReference { .. }
        | SymbolAtPosition::BuiltinFunctionReference { .. }
        | SymbolAtPosition::PythonImport { .. }
        | SymbolAtPosition::PythonFunctionReference { .. } => None,
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

/// Builds a workspace edit that renames `target` to `new_name`.
pub fn workspace_edit_for_rename(
    target: &RenameTarget,
    new_name: &str,
    runtime: &mut Runtime,
    trigger_model_path: &ModelPath,
    also_scan: &[ModelPath],
) -> Result<WorkspaceEdit, String> {
    validate_new_name(target, new_name, runtime)?;

    let occurrences = collect_rename_occurrences(target, runtime, trigger_model_path, also_scan);
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

    for edits in changes.values_mut() {
        sort_edits_descending(edits);
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
            let Some(model) = load_model(runtime, model_path) else {
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
            let Some(model) = load_model(runtime, model_path) else {
                return Err("could not load model".to_string());
            };
            let new_reference = ReferenceName::from(new_name);
            if model.reference_imports().contains_key(&new_reference)
                || model.submodel_imports().contains_key(&new_reference)
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
fn collect_rename_occurrences(
    target: &RenameTarget,
    runtime: &mut Runtime,
    trigger_model_path: &ModelPath,
    also_scan: &[ModelPath],
) -> Vec<RenameOccurrence> {
    match target {
        RenameTarget::Parameter { model_path, name } => {
            let mut paths = IndexSet::new();
            paths.insert(model_path.clone());
            paths.insert(trigger_model_path.clone());
            paths.extend(collect_composition_paths(runtime, trigger_model_path));
            paths.extend(collect_composition_paths(runtime, model_path));
            paths.extend(also_scan.iter().cloned());

            let mut occurrences = Vec::new();
            for path in paths {
                let Some(model) = load_model(runtime, &path) else {
                    continue;
                };
                if path == *model_path {
                    collect_local_parameter_occurrences(model, name, &mut occurrences);
                } else {
                    collect_external_parameter_occurrences(
                        model,
                        model_path,
                        name,
                        &mut occurrences,
                    );
                }
            }
            dedupe_occurrences(occurrences)
        }
        RenameTarget::ImportAlias { model_path, name } => {
            let Some(model) = load_model(runtime, model_path) else {
                return Vec::new();
            };
            let mut occurrences = Vec::new();
            collect_import_alias_occurrences(model, name, &mut occurrences);
            dedupe_occurrences(occurrences)
        }
    }
}

/// Collects all model paths that should be scanned for composition occurrences.
fn collect_composition_paths(runtime: &mut Runtime, path: &ModelPath) -> Vec<ModelPath> {
    runtime.check_model(path).0
}

/// How to match variable occurrences while walking expressions.
enum VariableRenameMode<'a> {
    LocalParameter {
        name: &'a ParameterName,
    },
    ExternalParameter {
        model: ModelTemplateReference<'a>,
        def_path: &'a ModelPath,
        name: &'a ParameterName,
    },
    ImportAlias {
        name: &'a ReferenceName,
    },
}

fn collect_local_parameter_occurrences(
    model: ModelTemplateReference<'_>,
    name: &ParameterName,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    let mode = VariableRenameMode::LocalParameter { name };

    collect_model_occurrences(model, &mode, occurrences);
}

fn collect_external_parameter_occurrences(
    model: ModelTemplateReference<'_>,
    def_path: &ModelPath,
    name: &ParameterName,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    let mode = VariableRenameMode::ExternalParameter {
        model,
        def_path,
        name,
    };

    collect_model_occurrences(model, &mode, occurrences);
}

fn collect_import_alias_occurrences(
    model: ModelTemplateReference<'_>,
    name: &ReferenceName,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    let mode = VariableRenameMode::ImportAlias { name };

    if let Some(reference) = model.reference_imports().get(name) {
        let model_path = model.path();
        push_occurrence(occurrences, model_path.clone(), reference.name_span.clone());
    }

    collect_model_occurrences(model, &mode, occurrences);
}

fn collect_model_occurrences(
    model: ModelTemplateReference<'_>,
    mode: &VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    let model_path = model.path();

    for param in model.parameters().values() {
        if let VariableRenameMode::LocalParameter { name } = mode
            && param.name() == *name
        {
            push_occurrence(occurrences, model_path.clone(), param.name_span().clone());
        }

        collect_parameter_value(model_path.clone(), param.value(), mode, occurrences);
        collect_limits(model_path.clone(), param.limits(), mode, occurrences);
    }

    for test in model.tests().values() {
        collect_expr(model_path.clone(), test.expr(), mode, occurrences);
    }
}

fn collect_parameter_value(
    model_path: ModelPath,
    value: &ir::ParameterValue,
    mode: &VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match value {
        ir::ParameterValue::Simple(expr, _) => {
            collect_expr(model_path, expr, mode, occurrences);
        }
        ir::ParameterValue::Piecewise(exprs, _) => {
            for piecewise in exprs {
                collect_expr(model_path.clone(), piecewise.expr(), mode, occurrences);
                collect_expr(model_path.clone(), piecewise.if_expr(), mode, occurrences);
            }
        }
    }
}

fn collect_limits(
    model_path: ModelPath,
    limits: &ir::Limits,
    mode: &VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match limits {
        ir::Limits::Default => {}
        ir::Limits::Continuous { min, max, .. } => {
            collect_expr(model_path.clone(), min, mode, occurrences);
            collect_expr(model_path, max, mode, occurrences);
        }
        ir::Limits::Discrete { values, .. } => {
            for value in values {
                collect_expr(model_path.clone(), value, mode, occurrences);
            }
        }
    }
}

fn collect_expr(
    model_path: ModelPath,
    expr: &ir::Expr,
    mode: &VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match expr {
        ir::Expr::Variable { variable, .. } => {
            visit_variable(model_path, variable, mode, occurrences);
        }
        ir::Expr::ComparisonOp {
            left,
            right,
            rest_chained,
            ..
        } => {
            collect_expr(model_path.clone(), left, mode, occurrences);
            collect_expr(model_path.clone(), right, mode, occurrences);
            for (_, chained) in rest_chained {
                collect_expr(model_path.clone(), chained, mode, occurrences);
            }
        }
        ir::Expr::BinaryOp { left, right, .. } | ir::Expr::Fallback { left, right, .. } => {
            collect_expr(model_path.clone(), left, mode, occurrences);
            collect_expr(model_path, right, mode, occurrences);
        }
        ir::Expr::UnaryOp { expr, .. } | ir::Expr::UnitCast { expr, .. } => {
            collect_expr(model_path, expr, mode, occurrences);
        }
        ir::Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_expr(model_path.clone(), arg, mode, occurrences);
            }
        }
        ir::Expr::Literal { .. } => {}
    }
}

fn visit_variable(
    model_path: ModelPath,
    variable: &ir::Variable,
    mode: &VariableRenameMode<'_>,
    occurrences: &mut Vec<RenameOccurrence>,
) {
    match mode {
        VariableRenameMode::LocalParameter { name } => {
            if let ir::Variable::Parameter {
                parameter_name,
                parameter_span,
            } = variable
                && parameter_name == *name
            {
                push_occurrence(occurrences, model_path, parameter_span.clone());
            }
        }
        VariableRenameMode::ExternalParameter {
            model,
            def_path,
            name,
        } => {
            let ir::Variable::External {
                reference_name,
                parameter_name,
                parameter_span,
                ..
            } = variable
            else {
                return;
            };
            if parameter_name == *name
                && resolve_reference_model_path(*model, reference_name).as_ref() == Some(def_path)
            {
                push_occurrence(occurrences, model_path, parameter_span.clone());
            }
        }
        VariableRenameMode::ImportAlias { name } => {
            let ir::Variable::External {
                reference_name,
                reference_span,
                ..
            } = variable
            else {
                return;
            };
            if reference_name == *name {
                push_occurrence(occurrences, model_path, reference_span.clone());
            }
        }
    }
}

fn push_occurrence(occurrences: &mut Vec<RenameOccurrence>, model_path: ModelPath, span: Span) {
    occurrences.push(RenameOccurrence { model_path, span });
}

fn dedupe_occurrences(mut occurrences: Vec<RenameOccurrence>) -> Vec<RenameOccurrence> {
    occurrences.sort_by_key(|o| {
        (
            o.model_path.clone(),
            o.span.start().offset,
            o.span.end().offset,
        )
    });
    occurrences.dedup();
    occurrences
}

fn sort_edits_descending(edits: &mut [TextEdit]) {
    edits.sort_by(|left, right| {
        left.range
            .start
            .line
            .cmp(&right.range.start.line)
            .then_with(|| left.range.start.character.cmp(&right.range.start.character))
    });
}

/// Maps a cursor symbol to a prepare-rename response.
pub fn prepare_rename_response(symbol: &SymbolAtPosition) -> Option<PrepareRenameResponse> {
    let range = prepare_rename_range(symbol);
    let placeholder = match symbol {
        SymbolAtPosition::ParameterDefinition { name, .. }
        | SymbolAtPosition::ParameterReference { name, .. }
        | SymbolAtPosition::ExternalParameterReference {
            parameter_name: name,
            ..
        } => name.as_str().to_string(),
        SymbolAtPosition::ModelImportReference { reference_name, .. }
        | SymbolAtPosition::ModelImportDefinition {
            name: ModelImportName::Reference(reference_name),
            ..
        } => reference_name.as_str().to_string(),
        SymbolAtPosition::BuiltinValueReference { .. }
        | SymbolAtPosition::BuiltinFunctionReference { .. }
        | SymbolAtPosition::PythonImport { .. }
        | SymbolAtPosition::PythonFunctionReference { .. }
        | SymbolAtPosition::ModelImportDefinition {
            name: ModelImportName::Submodel(_),
            ..
        } => return None,
    };

    Some(PrepareRenameResponse::RangeWithPlaceholder { range, placeholder })
}

/// Returns the range to highlight during prepare-rename.
#[must_use]
fn prepare_rename_range(symbol: &SymbolAtPosition) -> Range {
    span_to_range(&symbol.span())
}

fn load_model<'runtime>(
    runtime: &'runtime mut Runtime,
    path: &ModelPath,
) -> Option<ModelTemplateReference<'runtime>> {
    runtime.load_and_lower(path).0
}
