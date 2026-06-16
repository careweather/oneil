//! Rename (refactoring) support for Oneil models.

use std::collections::HashMap;

use oneil_runtime::Runtime;
use oneil_shared::symbols::{ParameterName, ReferenceName};
use tower_lsp_server::ls_types::{PrepareRenameResponse, TextEdit, Uri, WorkspaceEdit};

use crate::{
    location::span_to_range,
    occurrences::{SearchTarget, collect_occurrences},
    symbol_lookup::SymbolAtPosition,
};
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
    target: &SearchTarget,
    new_name: &str,
    runtime: &mut Runtime,
) -> Result<WorkspaceEdit, String> {
    validate_new_name(target, new_name, runtime)?;

    let occurrences = collect_occurrences(target, runtime);
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
    target: &SearchTarget,
    new_name: &str,
    runtime: &mut Runtime,
) -> Result<(), String> {
    if !is_valid_identifier(new_name) {
        return Err(format!("'{new_name}' is not a valid identifier"));
    }

    match target {
        SearchTarget::Parameter { model_path, name } => {
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
        SearchTarget::ImportAlias { model_path, name } => {
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
